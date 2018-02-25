#[macro_use]
extern crate clap;
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
        "new" => Box::new(command::New {
            dir: env::current_dir().unwrap(),
            task: submatches.value_of("task").unwrap().to_owned(),
            filename: submatches
                .value_of("filename")
                .map(|v| v.to_owned())
                .unwrap_or_default(),
            inherit: submatches.is_present("inherit"),
        }),
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
