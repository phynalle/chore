// use clap::{App, Arg, SubCommand, AppSettings};
use clap::App;

pub struct Chore;

impl Chore {
    pub fn initialize() -> App<'static, 'static> {
        clap_app!(choreful =>
             (version: crate_version!())
             (author: crate_authors!())
             (@setting DeriveDisplayOrder)
             (@setting SubcommandRequiredElseHelp)
             (@subcommand new =>
                 (about: "Create new task")
                 (@arg task: +required)
                 (@arg inherit: -i --inherit)
                 (@arg filename: )
                 (@arg src_task: -t --task +takes_value)
                 (@arg editor: -e --editor +takes_value)
             )
             (@subcommand edit =>
                 (about: "Edit a task")
                 (@arg task: +required)
             )
             (@subcommand run =>
                 (about: "Run a task")
                 (@arg task: +required)
                 (@arg args: +multiple)
             )
             (@subcommand show =>
                 (about: "Print the details of the task")
                 (@arg task: +required)
             )
             (@subcommand rename =>
                 (about: "Rename a task")
                 (@arg task: +required)
                 (@arg new_name: +required)
             )
             (@subcommand rm =>
                 (about: "Remove a task")
                 (@arg task: +required)
             )
             (@subcommand ls =>
                 (about: "Print tasks belong to current directory")
             )
        )
    }
}
