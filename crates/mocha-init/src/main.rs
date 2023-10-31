#![allow(internal_features)]
//#![deny(warnings)]
#![feature(allow_internal_unstable)]

use {
    self::{
        service::Service,
        settings::Settings,
        signal::{Signal, Signals},
    },
    futures_util::stream::TryStreamExt,
    mocha_os::{device, module, mount, process, virtual_console::VirtualConsole},
    std::{
        ffi::{CString, NulError},
        fs, future, io,
        net::IpAddr,
        num::NonZeroU8,
        path::Path,
        sync::Arc,
        task::{Context, Poll},
    },
    thiserror::Error,
    tokio::{net::UnixListener, process::Command},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {error}")]
    Io {
        #[from]
        error: io::Error,
    },
    #[error("netlink: {error}")]
    Netlink {
        #[from]
        error: rtnetlink::Error,
    },
    #[error("settings: {error}")]
    Settings {
        #[from]
        error: settings::SettingsError,
    },
    #[error("c string: {error}")]
    CString {
        #[from]
        error: NulError,
    },
}

mod backoff;
mod diagnostic;
mod service;
mod settings;
mod signal;

pub struct Init {
    settings: Settings,
    console: Arc<VirtualConsole>,
    signals: Signals,
    listener: UnixListener,
    login: Service,
}

impl Init {
    pub async fn new() -> Result<Self, Error> {
        info!("hello world")?;

        mount::setup_standard()?;

        let settings = Settings::open("/system/settings.toml").unwrap();
        let console = Arc::new(VirtualConsole::open(NonZeroU8::new(1).unwrap())?);
        let signals = Signals::new()?;
        let listener = bind_unix_listener("/tmp/init.sock")?;
        let login_console = Arc::clone(&console);

        setup_mounts(&settings)?;
        setup_modules(&settings)?;
        setup_network(&settings)?;

        let login = setup_login(&settings, login_console)?;

        Ok(Self {
            settings,
            console,
            signals,
            listener,
            login,
        })
    }

    pub async fn process(&mut self) -> io::Result<()> {
        future::poll_fn(|context| self.poll_process(context)).await
    }

    pub fn poll_process(&mut self, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        let _ = self.login.poll(context);

        match self.signals.poll_next(context) {
            Poll::Ready(signal) => return Poll::Ready(self.process_signal(signal)),
            _ => {}
        }

        Poll::Pending
    }

    fn process_signal(&mut self, signal: Signal) -> io::Result<()> {
        match signal {
            Signal::Child => {
                let _result = process::wait_all(|status| {
                    let _result = info!("{status:?}");
                });

                Ok(())
            }
            Signal::UserDefined1 => {
                info!("power off")?;

                Err(device::power_off())
            }
            Signal::UserDefined2 => {
                info!("restart")?;

                Err(device::restart())
            }
        }
    }
}

fn setup_mounts(settings: &Settings) -> Result<(), Error> {
    for (path, mount) in settings.mount.iter() {
        info!("mount: setup {path}")?;

        if let Err(error) = single_mount(path, mount) {
            info!("mount: {path}: {error}")?;
        }
    }

    Ok(())
}

fn single_mount(path: &str, mount: &settings::Mount) -> Result<(), Error> {
    let settings::Mount {
        executable,
        group_id,
        kind,
        user_id,
    } = mount;

    {
        let path = CString::new(path.to_string())?;
        let kind = CString::new(kind.clone())?;
        let data = mount::EMPTY;
        let mut flags = mount::DONT_UPDATE_ACCESS_TIME;

        flags.set(mount::NOT_EXECUTABLE, !executable);

        mount::ensure_mount(&kind, &path, &kind, flags, data)?;
    }

    std::os::unix::fs::chown(&path, Some(*user_id), Some(*group_id))?;

    info!("mount {path} as {kind}")?;

    Ok(())
}

fn setup_modules(settings: &Settings) -> Result<(), Error> {
    for (path, args) in settings.module.iter() {
        info!("module: setup {path} with args {args}")?;

        if let Err(error) = module::load_module(path, args.clone(), Default::default()) {
            info!("module: {path}: {error}")?;
        }
    }

    Ok(())
}

fn setup_network(settings: &Settings) -> Result<(), Error> {
    let network = settings.network.clone();

    tokio::spawn(async move {
        let (connection, handle, _) = rtnetlink::new_connection()?;

        tokio::spawn(connection);

        for (name, range) in network.interface {
            let mut links = handle.link().get().match_name(name.clone()).execute();
            let Some(link) = links.try_next().await? else {
                info!("couldn't find interface: {name}")?;

                continue;
            };

            info!("interface {name}: add {range}")?;

            handle
                .address()
                .add(link.header.index, range.ip(), range.prefix())
                .execute()
                .await?;

            info!("interface {name}: up")?;

            handle.link().set(link.header.index).up().execute().await?;
        }

        for (name, address) in network.route {
            if name == "default" {
                info!("default gateway: {address}")?;

                match address {
                    IpAddr::V4(gateway) => {
                        handle.route().add().v4().gateway(gateway).execute().await?
                    }
                    IpAddr::V6(gateway) => {
                        handle.route().add().v6().gateway(gateway).execute().await?
                    }
                }
            } else {
                info!("not sure what to do with route: {name}")?;
            }
        }

        Ok::<_, Error>(())
    });

    Ok(())
}

fn setup_login(settings: &Settings, login_console: Arc<VirtualConsole>) -> io::Result<Service> {
    let (username, user) = settings.user.iter().next().unwrap();
    let username = username.clone();
    let shell = user.shell.clone();
    let home = user.home.clone();

    let login = service::spawn(move || {
        let (stdin, stdout, stderr) = login_console.clone_for_stdio()?;
        let mut command = Command::new(&shell);

        command
            .env_clear()
            .current_dir(&home)
            .env("HOME", &home)
            .env("PATH", "/bin")
            .env("TERM", "linux")
            .kill_on_drop(true)
            .stderr(stderr)
            .stdin(stdin)
            .stdout(stdout)
            .gid(1)
            .uid(1);

        info!("spawn login for {username}")?;

        Ok(command)
    })?;

    Ok(login)
}

fn bind_unix_listener<P: AsRef<Path>>(path: P) -> io::Result<UnixListener> {
    let path = path.as_ref();

    UnixListener::bind(path).or_else(|_error| {
        fs::remove_file(path)?;

        UnixListener::bind(path)
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let mut init = Init::new().await?;

    loop {
        init.process().await?;
    }
}
