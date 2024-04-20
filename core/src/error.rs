pub enum Error {
    FileError,
    ElfParseError,
    NoProgramSegments,
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
