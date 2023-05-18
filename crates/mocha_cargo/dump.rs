#![feature(prelude_import)]
#![deny(warnings)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use {
    camino::{Utf8Path, Utf8PathBuf},
    cargo_metadata::Message, mocha_target::Target,
    mocha_utils::{Category, Command, Rule},
    serde::Deserialize,
    std::{
        collections::BTreeSet, fmt, fs, future::Future, io::{self, Cursor},
        os::unix::fs::PermissionsExt, pin::Pin, process::Stdio, ptr,
        task::{Context, Poll},
    },
    tokio::{
        io::{AsyncBufReadExt, BufReader, Lines},
        process::ChildStdout,
    },
};
mod error {
    use std::io::{Error, ErrorKind};
    pub fn must_be_an_exe() -> Error {
        Error::new(ErrorKind::PermissionDenied, "cargo must be an executable")
    }
    pub fn missing_stdio() -> Error {
        Error::new(ErrorKind::NotFound, "missing stdio handle")
    }
    pub fn invalid_message() -> Error {
        Error::new(ErrorKind::InvalidData, "invalid message from cargo")
    }
    pub fn invalid_plan(error: serde_json::Error) -> Error {
        Error::new(ErrorKind::InvalidData, error)
    }
    pub fn already_exited() -> Error {
        Error::new(ErrorKind::NotFound, "already exited")
    }
}
/// A `cargo` invocation context.
pub struct Cargo {
    cargo_path: Utf8PathBuf,
}
/// A `cargo build` invocation builder.
pub struct Build {
    cargo_path: Utf8PathBuf,
    workspace_path: Utf8PathBuf,
    features: BTreeSet<String>,
    target: Target,
}
enum ChildState {
    Plan {
        output: Pin<Box<dyn Future<Output = io::Result<mocha_utils::Output>>>>,
        child: mocha_utils::Child,
        stdout: Lines<BufReader<ChildStdout>>,
    },
    Build {
        child: mocha_utils::Child,
        stdout: Lines<BufReader<ChildStdout>>,
        completed: usize,
        total: usize,
    },
    Error,
    Done,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
enum ChildStateProjection<'__pin>
where
    ChildState: '__pin,
{
    Plan {
        output: ::pin_project_lite::__private::Pin<
            &'__pin mut (Pin<Box<dyn Future<Output = io::Result<mocha_utils::Output>>>>),
        >,
        child: &'__pin mut (mocha_utils::Child),
        stdout: &'__pin mut (Lines<BufReader<ChildStdout>>),
    },
    Build {
        child: &'__pin mut (mocha_utils::Child),
        stdout: ::pin_project_lite::__private::Pin<
            &'__pin mut (Lines<BufReader<ChildStdout>>),
        >,
        completed: &'__pin mut (usize),
        total: &'__pin mut (usize),
    },
    Error,
    Done,
}
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl ChildState {
        fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> ChildStateProjection<'__pin> {
            unsafe {
                match self.get_unchecked_mut() {
                    Self::Plan { output, child, stdout } => {
                        ChildStateProjection::Plan {
                            output: ::pin_project_lite::__private::Pin::new_unchecked(
                                output,
                            ),
                            child: child,
                            stdout: stdout,
                        }
                    }
                    Self::Build { child, stdout, completed, total } => {
                        ChildStateProjection::Build {
                            child: child,
                            stdout: ::pin_project_lite::__private::Pin::new_unchecked(
                                stdout,
                            ),
                            completed: completed,
                            total: total,
                        }
                    }
                    Self::Error => ChildStateProjection::Error,
                    Self::Done => ChildStateProjection::Done,
                }
            }
        }
    }
    #[allow(non_snake_case)]
    struct __Origin<'__pin> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        Plan: (
            Pin<Box<dyn Future<Output = io::Result<mocha_utils::Output>>>>,
            ::pin_project_lite::__private::AlwaysUnpin<mocha_utils::Child>,
            ::pin_project_lite::__private::AlwaysUnpin<Lines<BufReader<ChildStdout>>>,
        ),
        Build: (
            ::pin_project_lite::__private::AlwaysUnpin<mocha_utils::Child>,
            Lines<BufReader<ChildStdout>>,
            ::pin_project_lite::__private::AlwaysUnpin<usize>,
            ::pin_project_lite::__private::AlwaysUnpin<usize>,
        ),
        Error: (),
        Done: (),
    }
    impl<'__pin> ::pin_project_lite::__private::Unpin for ChildState
    where
        __Origin<'__pin>: ::pin_project_lite::__private::Unpin,
    {}
    trait MustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project_lite::__private::Drop> MustNotImplDrop for T {}
    impl MustNotImplDrop for ChildState {}
};
/// A `cargo build` child process.
pub struct Child {
    state: ChildState,
}
struct BuildPlan {
    invocations: Vec<serde_json::Value>,
}
#[automatically_derived]
impl ::core::fmt::Debug for BuildPlan {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "BuildPlan",
            "invocations",
            &&self.invocations,
        )
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for BuildPlan {
        fn deserialize<__D>(
            __deserializer: __D,
        ) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "field identifier",
                    )
                }
                fn visit_u64<__E>(
                    self,
                    __value: u64,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "invocations" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"invocations" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(
                        __deserializer,
                        __FieldVisitor,
                    )
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<BuildPlan>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = BuildPlan;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "struct BuildPlan",
                    )
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match match _serde::de::SeqAccess::next_element::<
                        Vec<serde_json::Value>,
                    >(&mut __seq) {
                        _serde::__private::Ok(__val) => __val,
                        _serde::__private::Err(__err) => {
                            return _serde::__private::Err(__err);
                        }
                    } {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(
                                _serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct BuildPlan with 1 element",
                                ),
                            );
                        }
                    };
                    _serde::__private::Ok(BuildPlan { invocations: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<
                        Vec<serde_json::Value>,
                    > = _serde::__private::None;
                    while let _serde::__private::Some(__key)
                        = match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "invocations",
                                        ),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    match _serde::de::MapAccess::next_value::<
                                        Vec<serde_json::Value>,
                                    >(&mut __map) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map) {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => {
                            match _serde::__private::de::missing_field("invocations") {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        }
                    };
                    _serde::__private::Ok(BuildPlan { invocations: __field0 })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["invocations"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "BuildPlan",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<BuildPlan>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
impl Cargo {
    /// Create a new context.
    ///
    /// Ensures that `cargo_path` points to a valid executable.
    pub fn new<P>(cargo_path: P) -> io::Result<Self>
    where
        P: AsRef<Utf8Path>,
    {
        let cargo_path = cargo_path.as_ref().canonicalize_utf8()?;
        let metadata = fs::metadata(&cargo_path)?;
        if !metadata.file_type().is_file() {
            return Err(error::must_be_an_exe());
        }
        if metadata.permissions().mode() & 0o100 != 0o100 {
            return Err(error::must_be_an_exe());
        }
        Ok(Self { cargo_path })
    }
    /// Create an invocation builder.
    pub fn build<P>(&self, workspace_path: P) -> Build
    where
        P: Into<Utf8PathBuf>,
    {
        Build {
            cargo_path: self.cargo_path.clone(),
            workspace_path: workspace_path.into(),
            features: BTreeSet::new(),
            target: Target::HOST,
        }
    }
}
impl Build {
    /// Append a cargo feature.
    pub fn feature<S>(mut self, feature: S) -> Self
    where
        S: Into<String>,
    {
        self.features.insert(feature.into());
        self
    }
    /// Append multiple cargo features.
    pub fn features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for feature in features {
            self = self.feature(feature);
        }
        self
    }
    /// Set the target to build for.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
    /// Start the build.
    pub fn spawn(self) -> io::Result<Child> {
        let Self { cargo_path, workspace_path, features, target } = self;
        let features = features.into_iter().collect::<Vec<_>>().join(",");
        let triple = target.rust_triple();
        let output = Command::new(&cargo_path)
            .current_dir(&workspace_path)
            .arg("+nightly")
            .arg("build")
            .arg("--build-plan")
            .arg("--no-default-features")
            .arg("--release")
            .arg("-Zunstable-options")
            .arg({
                let res = ::alloc::fmt::format(format_args!("--features={0}", features));
                res
            })
            .arg({
                let res = ::alloc::fmt::format(format_args!("--target={0}", triple));
                res
            })
            .execution_policy((Category::SetUsers, Rule::Kill))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        let output = Box::pin(output);
        let mut child = Command::new(&cargo_path)
            .current_dir(&workspace_path)
            .arg("+nightly")
            .arg("zigbuild")
            .arg("--message-format=json-render-diagnostics")
            .arg("--no-default-features")
            .arg("--release")
            .arg({
                let res = ::alloc::fmt::format(format_args!("--features={0}", features));
                res
            })
            .arg({
                let res = ::alloc::fmt::format(format_args!("--target={0}", triple));
                res
            })
            .execution_policy((Category::SetUsers, Rule::Kill))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdout = child.stdout().ok_or_else(error::missing_stdio)?;
        let stdout = BufReader::new(stdout).lines();
        Ok(Child {
            state: ChildState::Plan {
                output,
                child,
                stdout,
            },
        })
    }
}
impl Future for ChildState {
    type Output = io::Result<Option<(usize, usize)>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            ChildStateProjection::Plan { output, child, stdout } => {
                let poll = output
                    .poll(cx)
                    .map(|maybe_output| {
                        maybe_output.and_then(|output| parse_total(&output.stdout))
                    });
                match poll {
                    Poll::Ready(result) => {
                        let state = ChildState::Build {
                            child,
                            stdout,
                            completed: 0,
                            total,
                        };
                        (state, Poll::Pending)
                    }
                    Poll::Pending => {
                        let state = ChildState::Plan {
                            output,
                            child,
                            stdout,
                        };
                        (state, Poll::Pending)
                    }
                }
            }
        }
    }
}
impl Child {
    pub async fn process(&mut self) -> io::Result<Option<(usize, usize)>> {
        Pin::new(&mut self.state).await
    }
}
impl fmt::Debug for ChildState {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildState::Plan { child, stdout, .. } => {
                fmt.debug_struct("Plan")
                    .field("output", &"dyn Future")
                    .field("child", child)
                    .field("stdout", stdout)
                    .finish()
            }
            ChildState::Build { child, stdout, completed, total } => {
                fmt.debug_struct("Build")
                    .field("child", child)
                    .field("stdout", stdout)
                    .field("completed", completed)
                    .field("total", total)
                    .finish()
            }
            ChildState::Error => fmt.debug_struct("Error").finish(),
            ChildState::Done => fmt.debug_struct("Done").finish(),
        }
    }
}
fn parse_total(bytes: &[u8]) -> io::Result<usize> {
    let build_plan: BuildPlan = serde_json::from_slice(bytes)
        .map_err(error::invalid_plan)?;
    Ok(build_plan.invocations.len())
}
fn parse_message(bytes: &[u8]) -> io::Result<Message> {
    let message = Message::parse_stream(Cursor::new(bytes))
        .next()
        .ok_or_else(error::invalid_message)??;
    Ok(message)
}
