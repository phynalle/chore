use std::error;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::io;
use std::result;

use rocksdb;

use crate::task::TaskError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
    suggest: String,
}

impl Error {
    pub fn new<S: AsRef<str>>(message: S) -> Error {
        Error {
            message: message.as_ref().to_owned(),
            suggest: String::new(),
        }
    }

    pub fn with_suggest<S: AsRef<str>>(message: S, suggest: S) -> Error {
        Error {
            message: message.as_ref().to_owned(),
            suggest: suggest.as_ref().to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_error(f, &self.message, &self.suggest)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(err.description())
    }
}

impl From<TaskError> for Error {
    fn from(err: TaskError) -> Error {
        match err {
            TaskError::NotFound(s) => {
                use colored::*;
                let message = format!("Task '{}' doesn't exist", s.yellow());
                Error::new(message)
            }
            _ => Error::new(err.description()),
        }
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Error {
        Error::new(err.description())
    }
}

fn format_error(f: &mut fmt::Formatter, message: &str, suggest: &str) -> fmt::Result {
    use colored::*;

    let mut details = String::new();
    fmt::write(&mut details, format_args!("{}\n", message))?;
    if !suggest.is_empty() {
        fmt::write(&mut details, format_args!("\n\t{}\n", suggest))?;
    }
    write!(f, "{} {}", "error:".red().bold(), details,)
}
