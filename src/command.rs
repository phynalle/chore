use std::io::{stdin, stdout, Cursor, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::fs::File;
use std::collections::HashMap;

use rand::{thread_rng, Rng};

use db::open_database;
use error::{Error, Result};
use task::{Task, TaskError, TaskSystem};
use tempfile::TempFile;

use colored::*;

fn validate_task_name(task: &str) -> Result<()> {
    let suggest = if task == "." || task == ".." || task.ends_with('/') {
        "A task is like a file rather than a directory".to_string()
    } else if task.contains('/') {
        format!(
            "If you want to create a task in another directory, you should go there and try again.
\tOr could you drop '{}' in your task?",
            "/".green()
        )
    } else {
        return Ok(());
    };

    Err(Error::with_suggest(
        format!("'{}' is invalid name for a task", task.yellow()),
        suggest,
    ))
}

fn try_overwrite(task: &str) -> bool {
    let (i, o) = (stdin(), stdout());

    print!(
        "Task '{}' already exists. Do you want to overwrite it? [y/n]: ",
        task.yellow(),
    );
    if o.lock().flush().is_err() {
        return false;
    }

    let mut input = String::new();
    match i.read_line(&mut input) {
        Ok(_) => input.starts_with('y'),
        _ => false,
    }
}

pub trait Cmd {
    fn run(&self) -> Result<()>;
}

pub struct New {
    pub dir: PathBuf,
    pub task: String,
    pub inherit: bool,
    pub filename: String,
    pub src_task: String,
    pub ext: String,
    pub editor: Vec<String>,
}

impl Cmd for New {
    fn run(&self) -> Result<()> {
        validate_task_name(&self.task)?;

        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);

        if ts.exists(&self.task)? && !try_overwrite(&self.task) {
            return Ok(());
        }

        let mut task = Task::current(&self.task);
        let mut file: Box<Read> = if !self.filename.is_empty() {
            Box::new(File::open(&self.filename)?)
        } else if !self.src_task.is_empty() {
            let mut task = ts.open(&self.src_task)?;
            Box::new(Cursor::new(task.take()))
        } else {
            let mut file = create_tempfile(&self.ext).expect("failed to open temp file");
            file.write_all(task.content())?;
            file.close();

            let mut cmd: Child = Command::new(&self.editor[0])
                .args(&self.editor[1..])
                .arg(file.path())
                .spawn()?;
            let exit = cmd.wait()?;

            if exit.success() {
                file.reopen()?;
                Box::new(file)
            } else {
                return Err(Error::new("failed to write file"));
            }
        };

        let mut content = Vec::new();
        let _ = file.read_to_end(&mut content)?;
        task.set_extension(&self.ext);
        task.set_inherit(self.inherit);
        task.set_content(content);
        ts.save(&task)?;

        print_done("New task is created successfully!");
        Ok(())
    }
}

pub struct Edit {
    pub task: String,
}

impl Cmd for Edit {
    fn run(&self) -> Result<()> {
        validate_task_name(&self.task)?;

        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);
        let mut task = match ts.open(&self.task) {
            Ok(task) => task,
            Err(e) => return Err(e.into()),
        };

        let mut file = create_tempfile(task.extension()).expect("failed to open temp file");
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

        print_done("The task is edited successfully!");
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
        validate_task_name(&self.task)?;

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
                    TaskError::NotFound(_) => (),
                    _ => return Err(e.into()),
                },
            }

            if !dir.pop() {
                return Err(TaskError::NotFound(self.task.clone()).into());
            }

            is_cwd = false;
        };

        let mut file = create_tempfile(task.extension()).expect("failed to open temp file");
        file.write_all(task.content())?;
        file.close();

        Command::new("chmod")
            .arg("+x")
            .arg(file.path())
            .spawn()?
            .wait()?;

        let mut cmd = Command::new(file.path());
        let mut child: Child = cmd.arg(file.path()).args(&self.args).spawn()?;
        let _ = child.wait()?;
        Ok(())
    }
}

pub struct Show {
    pub task: String,
}

impl Cmd for Show {
    fn run(&self) -> Result<()> {
        validate_task_name(&self.task)?;
        let db = open_database().expect("unabled to open db");
        let ts = TaskSystem::new(db);
        let task: Task = ts.open(&self.task)?;

        println!("{}", "[options]".green().bold());
        println!("inherit: {}", task.inherit().to_string().red());
        println!("{}", "[content]".green().bold());
        println!("{}", String::from_utf8_lossy(task.content()).to_owned());
        Ok(())
    }
}

pub struct Remove {
    pub task: String,
}

impl Cmd for Remove {
    fn run(&self) -> Result<()> {
        validate_task_name(&self.task)?;

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

        let mut task_names = HashMap::new();
        let mut dir: PathBuf = self.dir.clone();
        let mut is_cwd = true;

        loop {
            let mut print_dir = false;
            for task in ts.scan(&dir, true)? {
                if task_names.get(task.name()).is_some() {
                    // Only first found task is visible
                    continue;
                }

                if is_cwd || task.inherit() {
                    task_names.insert(task.name().to_owned(), true);

                    if !print_dir {
                        let mut message = format!("[{}", dir.to_string_lossy().green());
                        if is_cwd {
                            message.push_str(&format!(" {}", "(current)".red()));
                        }
                        message.push(']');
                        println!("{}", message);
                        print_dir = true;
                    }
                    println!("  {}", task.name(),);
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

pub struct Rename {
    pub from: String,
    pub to: String,
}

impl Cmd for Rename {
    fn run(&self) -> Result<()> {
        validate_task_name(&self.from)?;
        validate_task_name(&self.to)?;

        let db = open_database()?;
        let ts = TaskSystem::new(db);

        let from_task = ts.open(&self.from)?;
        if ts.exists(&self.to)? && !try_overwrite(&self.to) {
            return Ok(());
        }

        let mut to_task = Task::current(&self.to);
        to_task.copy_from(&from_task);

        let mut batch = ts.batch();
        batch.save(&to_task)?;
        batch.remove_task(from_task)?;
        batch.commit()?;
        Ok(())
    }
}

fn create_tempfile(ext: &str) -> Result<TempFile> {
    let length = 8;
    let charset = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes();

    let mut rng = thread_rng();
    let mut file_name = ".chore".to_string();

    for _ in 0..length {
        file_name.push(*rng.choose(charset).unwrap() as char);
    }

    if !ext.is_empty() {
        if !ext.starts_with('.') {
            file_name.push('.');
        }
        file_name.push_str(ext);
    }

    TempFile::create(file_name)
}

fn print_done(message: &str) {
    println!("    {} {}", "Done".green().bold(), message)
}
