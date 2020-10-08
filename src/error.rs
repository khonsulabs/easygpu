use std::io;

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum Error {
    #[error("a suitable graphics adapter was not found")]
    NoAdaptersFound,
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}
