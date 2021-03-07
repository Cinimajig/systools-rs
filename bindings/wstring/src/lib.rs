use ::std::fmt;

#[derive(Default, Debug)]
pub struct WideString {
    inner: Vec<u16>,
}

impl WideString {
    pub fn ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }

    pub fn mut_ptr(&mut self) -> *mut u16 {
        self.inner.as_mut_ptr()
    }

    pub fn with_size(size: usize) -> Self {
        let mut vec = Vec::new();
        vec.resize(size, 0);

        Self { inner: vec }
    }

    pub fn from_str_with_size(text: &str, size: usize) -> Self {
        let mut vec = get_wide_string(text);
        vec.resize(size, 0);

        Self { inner: vec }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            inner: get_wide_string(text),
        }
    }
}

impl fmt::Display for WideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = String::from_utf16_lossy(&self.inner);
        write!(f, "{}", string)
    }
}

fn get_wide_string(text: &str) -> Vec<u16> {
    use ::std::ffi::OsStr;
    use ::std::os::windows::ffi::OsStrExt;

    OsStr::new(text)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
