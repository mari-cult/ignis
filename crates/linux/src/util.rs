macro_rules! cstr {
    ($string:literal) => {{
        const CSTR: &'static ::core::ffi::CStr =
            match ::core::ffi::CStr::from_bytes_with_nul(::core::concat!($string, '\0').as_bytes())
            {
                Ok(cstr) => cstr,
                Err(_error) => panic!("invalid C string"),
            };

        CSTR
    }};
}

pub(crate) use cstr;
