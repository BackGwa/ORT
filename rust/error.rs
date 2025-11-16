use std::fmt;

#[derive(Debug)]
pub struct OrtError {
    pub line: usize,
    pub code: String,
    pub message: String,
}

impl OrtError {
    pub fn new(line: usize, code: String, message: String) -> Self {
        Self { line, code, message }
    }
}

impl fmt::Display for OrtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::Colorize;
        writeln!(f, "{} | {}",
            format!("{:3}", self.line).blue(),
            self.code.white()
        )?;
        write!(f, "{} : {}",
            "Exception".red(),
            self.message.white()
        )
    }
}

impl std::error::Error for OrtError {}

pub type OrtResult<T> = Result<T, OrtError>;
