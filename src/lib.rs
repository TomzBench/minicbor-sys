extern crate cddl_gen;
use std::path::PathBuf;
use std::{error, fmt, fs, result};

pub use cddl_gen::{Language, Options};

#[derive(Debug)]
pub enum Error {
    CouldNotOpenCddl(PathBuf),
    Render(cddl_gen::RenderError),
}

impl From<cddl_gen::RenderError> for Error {
    fn from(value: cddl_gen::RenderError) -> Self {
        Error::Render(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CouldNotOpenCddl(path) => write!(f, "Could not open CDDL {}", path.display()),
            Error::Render(e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {}

pub fn generate(path: &str, options: Options) -> result::Result<String, Error> {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR")).join(path);
    cddl_gen::render_lib(
        &fs::read_to_string(path.clone()).map_err(|_| Error::CouldNotOpenCddl(path))?,
        &options,
    )
    .map_err(Error::from)
}
