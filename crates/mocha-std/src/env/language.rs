use std::{env, ffi::OsStr, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Language {
    #[default]
    English,
    Japanese,
}

impl Language {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let language = match bytes {
            b"ja_JP" => Language::Japanese,
            b"en_US" => Language::English,
            _ => return None,
        };

        Some(language)
    }

    #[inline]
    fn from_os_str(string: &OsStr) -> Option<Self> {
        let bytes = string.as_encoded_bytes();

        Self::from_bytes(bytes)
    }
}

pub fn language() -> Language {
    static LANGUAGE: OnceLock<Language> = OnceLock::new();

    *LANGUAGE.get_or_init(|| {
        env::var_os("LANG")
            .and_then(|language| Language::from_os_str(&language))
            .unwrap_or_default()
    })
}
