/// The errors
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    Tokenizer,
    Parse,
    Engine,
}

pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub line_number: usize,
}

impl ErrorKind {
    /// Returns a string representation of the error.
    pub fn as_str(&self) -> &str {
        match self {
            ErrorKind::Tokenizer => "TokenizerError",
            ErrorKind::Parse => "ParseError",
            ErrorKind::Engine => "EngineError",
        }
    }
}

impl Error {
    pub fn new(kind: ErrorKind, message: String, line_number: usize) -> Self {
        Self {
            kind,
            message,
            line_number,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: \"{}\" at line: {}",
            self.kind.as_str(),
            self.message,
            self.line_number
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;
