use std::fmt;
use std::result;
use std::io;
use std::error;
use serde_json;
use serde_json::error::Category;
use rocksdb;
use task::TaskError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    DB(rocksdb::Error),
    Json(serde_json::Error),
    Task(TaskError),
    Message(String),
}

impl Error {
    pub fn new<S: AsRef<str>>(message: S) -> Error {
        Error::Message(message.as_ref().to_owned())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "IO Error: {}", e),
            Error::DB(ref e) => write!(f, "DB Error: {}", e),
            Error::Json(ref e) => write!(f, "Json Error: {}", e),
            Error::Task(ref e) => write!(f, "Task: {}", e),
            Error::Message(ref e) => write!(f, "Error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::DB(ref e) => e.description(),
            Error::Json(ref e) => e.description(),
            Error::Task(ref e) => e.description(),
            Error::Message(ref e) => e.as_str(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::DB(ref e) => Some(e),
            Error::Json(ref e) => Some(e),
            Error::Task(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Error {
        Error::DB(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        match err.classify() {
            Category::Io => Error::Io(err.into()),
            _ => Error::Json(err),
        }
    }
}

impl From<TaskError> for Error {
    fn from(err: TaskError) -> Error {
        Error::Task(err)
    }
}
