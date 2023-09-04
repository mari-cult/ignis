use {
    codespan_reporting::{
        diagnostic::{Diagnostic, Label},
        files::SimpleFile,
        term::termcolor::{ColorChoice, StandardStream},
    },
    indexmap::IndexMap,
    ipnetwork::IpNetwork,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, fs, io, net::IpAddr, num::NonZeroU32, ops::Range, path::Path},
    thiserror::Error,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub locale: String,
    #[serde(default)]
    pub module: IndexMap<String, String>,
    #[serde(default)]
    pub mount: IndexMap<String, Mount>,
    pub network: Network,
    #[serde(default)]
    pub package: HashMap<String, Package>,
    pub theme: Theme,
    pub timezone: String,
    pub user: HashMap<String, User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub network: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Mount {
    pub executable: bool,
    #[serde(default)]
    pub group_id: u32,
    pub kind: String,
    #[serde(default)]
    pub user_id: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Network {
    #[serde(default)]
    pub interface: HashMap<String, IpNetwork>,
    #[serde(default)]
    pub route: HashMap<String, IpAddr>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub home: String,
    pub shell: String,
    pub user_id: NonZeroU32,
    pub group_id: NonZeroU32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("io: {error}")]
    Io {
        #[from]
        error: io::Error,
    },
    #[error("syntax error")]
    Syntax {
        toml: String,
        message: String,
        span: Range<usize>,
    },
}

impl SettingsError {
    pub fn diagnostic(&self) -> Option<Diagnostic<()>> {
        if let SettingsError::Syntax {
            toml,
            message,
            span,
        } = self
        {
            let file = SimpleFile::new("settings.toml", toml);
            let diagnostic = Diagnostic::error()
                .with_message("syntax error")
                .with_labels(vec![Label::primary((), span.clone()).with_message(message)]);

            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            let _result =
                codespan_reporting::term::emit(&mut writer.lock(), &config, &file, &diagnostic);

            Some(diagnostic)
        } else {
            None
        }
    }
}

impl Settings {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, SettingsError> {
        let path = path.as_ref();
        let toml = fs::read_to_string(path)?;
        let settings = toml::from_str(&toml).map_err(|error| {
            let message = error.message().to_string();
            let span = error.span().unwrap_or_default();

            SettingsError::Syntax {
                toml,
                message,
                span,
            }
        })?;

        Ok(settings)
    }
}
