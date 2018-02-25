# chore
chore is a directory-dependent task runner.
You can create your own tasks belonging to a directory in which you can only execute them

## Usage
```sh
chore `subcommand` [parameters]
chore new `task` [filename] [--inherit]
chore edit `task`
chore run `task`
cargo rename `task` `new name`
chore rm `task`
chore ls
```

## Options

There are useful options for some command. The details will be added later.
- inherit (boolean): By default, a task is only executed in the created directory. When the option is set on, the task can be executed in its subdirectories

## Tutorial
```sh
# You can create a task named profile like this
chore new profile
# Also you can create a task based on your file
chore new profile /path/to/file

# Next, you can run the task
chore run profile

# Ok, well done! if you want to remove it, try it
chore rm profile
```
Now you are the expert for chore if you followed this guidelines successfully!

## Problem

chore manages all tasks through an central database, [RocksDB](https://github.com/facebook/rocksdb) which is key-value sot store by facebook.
This implementation makes it difficult to copy or move multiple tasks to different directory.
So you may use `chore cp` and `chore mv` (both of them aren't implemented yet) manually to achieve it.

## Caution

chore is a very young project so it isn't featured fully.
In current version, `sh` is the only supported script language and `vi` is the only supported editor.

Don't be disappointed yet. Fortunately, You have a cool solution.
Try to duplicate the editor you use to 'vi' and script-executable to 'sh' just like `copy /path/to/editor /bin/vi` and `copy /path/to/python /bin/sh`.
Just kidding. I don't extremely recommend you to do so. you may know the reason.
