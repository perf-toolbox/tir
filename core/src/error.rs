pub enum Error {
    FileError,
    ElfParseError,
    NoProgramSegments,
}

pub type Result<T> = std::result::Result<T, Error>;
