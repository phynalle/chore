#[macro_use]
extern crate clap;
extern crate colored;
extern crate rand;
extern crate rocksdb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;

mod app;
mod command;
mod db;
mod error;
mod task;
mod tempfile;
mod path;

use command::Cmd;

fn main() {
    let matches = app::Chore::initialize().get_matches();
    let subcmd = matches.subcommand_name().unwrap();
    let submatches = matches.subcommand_matches(subcmd).unwrap();

    let cmd: Box<Cmd> = match subcmd {
        "new" => {
            let task = submatches.value_of("task").unwrap().to_owned();
            let ext = extract_extension(&task).unwrap_or_else(|| "sh".to_owned());
            Box::new(command::New {
                dir: env::current_dir().unwrap(),
                task,
                ext,
                inherit: submatches.is_present("inherit"),
                filename: submatches
                    .value_of("filename")
                    .map(|v| v.to_owned())
                    .unwrap_or_default(),
                src_task: submatches
                    .value_of("src_task")
                    .map(|v| v.to_owned())
                    .unwrap_or_default(),
                editor: submatches
                    .value_of("editor")
                    .unwrap_or("vi")
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>(),
            })
        }
        "edit" => Box::new(command::Edit {
            task: submatches.value_of("task").unwrap().to_owned(),
        }),
        "run" => Box::new(command::Run {
            dir: env::current_dir().unwrap(),
            task: submatches.value_of("task").unwrap().to_owned(),
            args: submatches
                .values_of("args")
                .map(|values| values.map(|s| s.to_owned()).collect())
                .unwrap_or_else(|| Vec::new()),
        }),
        "show" => Box::new(command::Show {
            task: submatches.value_of("task").unwrap().to_owned(),
        }),
        "rename" => Box::new(command::Rename {
            from: submatches.value_of("task").unwrap().to_owned(),
            to: submatches.value_of("new_name").unwrap().to_owned(),
        }),
        "rm" => Box::new(command::Remove {
            task: submatches.value_of("task").unwrap().to_owned(),
        }),
        "ls" => Box::new(command::List {
            dir: env::current_dir().unwrap(),
        }),
        _ => return,
    };

    if let Err(e) = cmd.run() {
        println!("{}", e);
    }
}

fn extract_extension(task_name: &str) -> Option<String> {
    use std::path::Path;
    Path::new(task_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
}
