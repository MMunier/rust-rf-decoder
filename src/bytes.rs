use std::{fmt::Write, slice::EscapeAscii};

pub struct Bytes<T>(pub T)
where
    T: AsRef<[u8]>;

impl<T> std::fmt::Display for Bytes<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("b'")?;
        let val = self.0.as_ref().escape_ascii().to_string();
        f.write_str(&val)?;
        f.write_char('\'')
    }
}
