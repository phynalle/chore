use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::fs::File;

use rand::{thread_rng, Rng};

use db::open_database;
use error::{Error, Result};
use task::{Task, TaskError, TaskSystem};
use tempfile::TempFile;

pub trait Cmd {
    fn run(&self) -> Result<()>;
}

pub struct New {
    pub dir: PathBuf,
    pub task: String,
    pub filename: String,
    pub inherit: bool,
}

impl Cmd for New {
    fn run(&self) -> Result<()> {
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);

        if ts.exists(&self.task)? {
            // Already exist: Ask to overwrite it
            print!(
                "{} already exists. do you want to overwrite it? [y/n]: ",
                self.task
            );

            let (i, o) = (stdin(), stdout());
            o.lock().flush()?;

            let mut arr = [0u8; 1];
            i.lock().read(&mut arr)?;
            if arr[0] != b'y' {
                return Ok(());
            }
        }

        let mut task = Task::current(&self.task);
        let mut file: Box<Read> = if self.filename.is_empty() {
            let mut file = create_tempfile().expect("failed to open temp file");
            file.write_all(task.content())?;
            file.close();

            let mut cmd: Child = Command::new("vi").arg(file.path()).spawn()?;
            let exit = cmd.wait()?;

            if exit.success() {
                file.reopen()?;
                Box::new(file)
            } else {
                return Err(Error::new("failed to write file"));
            }
        } else {
            Box::new(File::open(&self.filename)?)
        };

        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content)?;
        task.set_inherit(self.inherit);
        task.set_content(content);
        ts.save(&task)?;

        Ok(())
    }
}

pub struct Edit {
    pub task: String,
}

impl Cmd for Edit {
    fn run(&self) -> Result<()> {
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);
        let mut task = match ts.open(&self.task) {
            Ok(task) => task,
            Err(TaskError::NotFound) => {
                // Print beautiful Not Found error message
                return Err(From::from(TaskError::NotFound));
            }
            Err(e) => return Err(From::from(e)),
        };

        let mut file = create_tempfile().expect("failed to open temp file");
        file.write_all(task.content())?;
        file.close();

        let mut cmd: Child = Command::new("vi").arg(file.path()).spawn()?;
        let exit = cmd.wait()?;

        if exit.success() {
            file.reopen()?;

            let mut content = Vec::new();
            let _ = file.read_to_end(&mut content)?;
            task.set_content(content);
            ts.save(&task)?;
        } else {
            return Err(Error::new("failed to write file"));
        }

        Ok(())
    }
}

pub struct Run {
    pub dir: PathBuf,
    pub task: String,
    pub args: Vec<String>,
}

impl Cmd for Run {
    fn run(&self) -> Result<()> {
        let mut dir = self.dir.clone();
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);

        let mut is_cwd = true;
        let task = loop {
            let path = {
                let mut dir = dir.clone();
                dir.push(&self.task);
                dir
            };

            match ts.open(&path) {
                Ok(task) => {
                    if is_cwd || task.inherit() {
                        break task;
                    }
                }
                Err(e) => match e {
                    TaskError::NotFound => (),
                    _ => return Err(e.into()),
                },
            }

            if !dir.pop() {
                return Err(Error::Task(TaskError::NotFound));
            }

            is_cwd = false;
        };

        let mut file = create_tempfile().expect("failed to open temp file");
        file.write_all(task.content())?;
        file.close();

        let mut cmd: Child = Command::new("sh")
            .arg(file.path())
            .args(&self.args)
            .spawn()?;

        let _ = cmd.wait()?;
        Ok(())
    }
}

pub struct Show {
    pub task: String,
}

impl Cmd for Show {
    fn run(&self) -> Result<()> {
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);
        let task: Task = ts.open(&self.task)?;

        println!("Inherit: {}", task.inherit());
        println!(
            "Source: \n{}",
            String::from_utf8_lossy(task.content()).to_owned()
        );
        Ok(())
    }
}

pub struct Remove {
    pub task: String,
}

impl Cmd for Remove {
    fn run(&self) -> Result<()> {
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);
        ts.remove(&self.task).map_err(|e| e.into())
    }
}

pub struct List {
    pub dir: PathBuf,
}

impl Cmd for List {
    fn run(&self) -> Result<()> {
        let db = open_database()?;
        let ts = TaskSystem::new(db);

        let mut dir = self.dir.clone();
        let mut is_cwd = true;

        loop {
            for task in ts.scan(&dir, true)? {
                if is_cwd || task.inherit() {
                    println!(
                        "[{}]: location: {}, inherit: {}",
                        task.name(),
                        task.path(),
                        task.inherit()
                    );
                }
            }

            if !dir.pop() {
                break;
            }
            is_cwd = false;
        }
        Ok(())
    }
}

fn create_tempfile() -> Result<TempFile> {
    let length = 8;
    let charset = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();

    let mut rng = thread_rng();
    let mut filename = ".chore".to_string();
    (0..length).for_each(|_| filename.push(*rng.choose(charset).unwrap() as char));
    filename.push_str(".sh");

    TempFile::create(filename)
}
