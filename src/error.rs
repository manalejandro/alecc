use thiserror::Error;

#[derive(Error, Debug)]
pub enum AleccError {
    #[error("Lexical error at line {line}, column {column}: {message}")]
    LexError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[allow(dead_code)]
    #[error("Semantic error: {message}")]
    SemanticError { message: String },

    #[error("Code generation error: {message}")]
    CodegenError { message: String },

    #[error("Linker error: {message}")]
    LinkerError { message: String },

    #[error("Target not supported: {target}")]
    UnsupportedTarget { target: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    #[allow(dead_code)]
    #[error("Internal compiler error: {message}")]
    InternalError { message: String },
}

pub type Result<T> = std::result::Result<T, AleccError>;
