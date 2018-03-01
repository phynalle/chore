use std::env;
use std::error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::result;
use rocksdb::{self, DB};
use serde_json;

type Result<T> = result::Result<T, TaskError>;

pub struct TaskSystem {
    db: DB,
}

impl TaskSystem {
    pub fn new(db: DB) -> TaskSystem {
        TaskSystem { db }
    }

    fn normalize<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        use path::normalize;
        normalize(&path).ok_or(TaskError::InvalidPath)
    }

    fn key<P: AsRef<Path>>(path: P) -> Result<String> {
        let abs_path = TaskSystem::normalize(path)?;
        Ok(format!("task.{}", abs_path.to_string_lossy()))
    }

    pub fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        match self.open(path) {
            Ok(_) => Ok(true),
            Err(TaskError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<Task> {
        let abs_path = TaskSystem::normalize(&path)?;
        let key = TaskSystem::key(&abs_path)?;
        match self.db.get(key.as_bytes())? {
            Some(v) => Task::from_slice(&abs_path, &v),
            None => Err(TaskError::NotFound(
                path.as_ref().to_string_lossy().to_string(),
            )),
        }
    }

    pub fn save(&self, task: &Task) -> Result<()> {
        let mut batch = self.batch();
        batch.save(task)?;
        batch.commit()
    }

    pub fn remove<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut batch = self.batch();
        batch.remove(path)?;
        batch.commit()
    }

    pub fn remove_task(&self, task: Task) -> Result<()> {
        self.remove(task.path)
    }

    // scan is a expensive method so it should be used carefully.
    pub fn scan<P: AsRef<Path>>(&self, path: P, current_only: bool) -> Result<ScanIterator> {
        let mut prefix = TaskSystem::key(path)?;
        if !prefix.ends_with('/') {
            prefix.push('/');
        }

        Ok(ScanIterator {
            inner: self.db.prefix_iterator(prefix.as_bytes()),
            prefix,
            current_only,
        })
    }

    pub fn batch<'a>(&'a self) -> WriteBatch<'a> {
        WriteBatch {
            ts: self,
            batch: rocksdb::WriteBatch::default(),
        }
    }
}

pub struct WriteBatch<'a> {
    ts: &'a TaskSystem,
    batch: rocksdb::WriteBatch,
}

impl<'a> WriteBatch<'a> {
    pub fn save(&mut self, task: &Task) -> Result<()> {
        let key = format!("task.{}", task.path).into_bytes();
        let value = serde_json::to_vec(&task.inner)?;
        self.batch.put(&key, &value).map_err(|e| e.into())
    }

    pub fn remove<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let key = TaskSystem::key(path)?;
        self.batch.delete(key.as_bytes()).map_err(|e| e.into())
    }

    pub fn remove_task(&mut self, task: Task) -> Result<()> {
        self.remove(task.path)
    }

    pub fn commit(self) -> Result<()> {
        self.ts.db.write(self.batch).map_err(|e| e.into())
    }
}

pub struct ScanIterator {
    inner: rocksdb::DBIterator,
    prefix: String,
    current_only: bool,
}

impl Iterator for ScanIterator {
    type Item = Task;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (key, val) = self.inner.next()?;
            let path = String::from_utf8(key.into_vec()).unwrap();
            if !path.starts_with(&self.prefix) {
                break None;
            }

            let relative_path = &path[self.prefix.len()..];
            if self.current_only && relative_path.contains('/') {
                continue;
            }

            let absolute_path = &path["task.".len()..];
            if let Ok(task) = Task::from_slice(absolute_path, &val) {
                break Some(task);
            }
        }
    }
}

pub struct Task {
    name: String,
    path: String,
    inner: Inner,
}

impl Task {
    pub fn current(name: &str) -> Task {
        let mut path = env::current_dir().unwrap();
        path.push(name);

        Task {
            name: name.to_owned(),
            path: path.into_os_string().into_string().unwrap(),
            inner: Inner::default(),
        }
    }

    fn from_slice<P: AsRef<Path>>(abs_path: P, v: &[u8]) -> Result<Task> {
        assert!(abs_path.as_ref().is_absolute());

        let inner: Inner = serde_json::from_slice(&v)?;
        let name = format!(
            "{}",
            abs_path
                .as_ref()
                .file_name()
                .ok_or(TaskError::InvalidPath)?
                .to_string_lossy()
        );
        let path = format!("{}", abs_path.as_ref().to_string_lossy());
        Ok(Task { inner, name, path })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn inherit(&self) -> bool {
        self.inner.inherit
    }

    pub fn set_inherit(&mut self, inherit: bool) {
        self.inner.inherit = inherit;
    }

    pub fn content(&self) -> &[u8] {
        &self.inner.content
    }

    pub fn take(self) -> Vec<u8> {
        self.inner.content
    }

    pub fn set_content(&mut self, content: Vec<u8>) {
        self.inner.content = content;
    }

    pub fn set_extension(&mut self, ext: &str) {
        self.inner.extension = ext.to_owned();
    }

    pub fn extension(&self) -> &str {
        &self.inner.extension
    }

    pub fn copy_from(&mut self, task: &Task) {
        self.inner = task.inner.clone();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Inner {
    inherit: bool,
    #[serde(default = "String::default")]
    extension: String,
    content: Vec<u8>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            content: Vec::new(),
            extension: String::new(),
            inherit: false,
        }
    }
}

#[derive(Debug)]
pub enum TaskError {
    InvalidPath,
    NotFound(String),
    BrokenData,
    DBOperationFailed(rocksdb::Error),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        match *self {
            // TaskError::InvalidPath(ref s) => write!(f, "{}: {}", s, self.description()),
            TaskError::NotFound(ref s) => write!(f, "{}: {}", s, self.description()),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl error::Error for TaskError {
    fn description(&self) -> &str {
        match *self {
            TaskError::InvalidPath => "Invalid path",
            TaskError::NotFound(_) => "No available task",
            TaskError::BrokenData => "Broken data",
            TaskError::DBOperationFailed(ref e) => e.description(),
        }
    }
}

impl From<rocksdb::Error> for TaskError {
    fn from(err: rocksdb::Error) -> TaskError {
        TaskError::DBOperationFailed(err)
    }
}

impl From<serde_json::Error> for TaskError {
    fn from(_: serde_json::Error) -> TaskError {
        TaskError::BrokenData
    }
}
