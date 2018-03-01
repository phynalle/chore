# chore
chore is a directory-dependent task runner.
You can create your own tasks belonging to a directory in which you can only execute them

## Getting Started
### Prerequisites
You need to install [Rust Programming Language](https://www.rust-lang.org/) and its package manager [Cargo](https://github.com/rust-lang/cargo)

For easy installation, I recommend you use [rustup](https://www.rustup.rs/)
```sh
curl https://sh.rustup.rs -sSf | sh
```

### Installing
```sh
clone https://github.com/phynalle/chore

cd chore

cargo install
```

## Usage
```sh
chore `subcommand` [parameters]
chore new `task` [filename] [--task `task name`] [--inherit]
chore edit `task`
chore run `task`
cargo rename `task` `new name`
chore rm `task`
chore ls
```

### Options

There are useful options for some command. The details will be added later.
- inherit (boolean): By default, a task is only executed in the created directory. When the option is set on, the task can be executed in its subdirectories

## Tutorial
```sh
# You can create a task named profile like this
chore new profile
# Also you can create a task based on your file
chore new profile /path/to/file
# or on a task in the same directory
chore new profile --task another_profile_task

# Next, you can run the task
chore run profile

# If you want to rename it, just do like this
chore rename profile prof

# Ok, well done! if you want to remove it, try it
chore rm prof
```
Now you are the expert for chore if you followed this guidelines successfully!

## What you should know
chore is a very young project so it isn't featured fully.
Therefore, you may encounter some situation to irritate you.

- I have only developed and tested on macOS, High Sierra. 
So I'm not sure it works on other operating systems.

- You may come across a situation to change your directory name. 
Unfortunately, It is not yet implemented to copy or move multiple tasks in a directory to another one.
So you may give up all the task in it, or you should recreate each task by using --task option.

- chore manages all tasks through an central database, [RocksDB](https://github.com/facebook/rocksdb) which is key-value sot store by facebook.
And it create db file in ~/.tmp/testdb (sorry for careless name).
