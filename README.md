
# Basic to-do app in Rust

Basic to-do app to learn Rust and use crates

![Example screenshot](./todo-example.gif)


## Authors

- [@elevenJune](https://github.com/ElevenJune)



## Usage

__todo [COMMAND]__

### Commands

| Command   | Description                                    |
|-----------|------------------------------------------------|
| list      | List tasks                                    |
| add       | Add task to list                              |
| remove    | Remove tasks from list by indexes             |
| rename    | Rename a task                                 |
| priority  | Set the priority of a task                    |
| clear     | Clear the list                                |
| done      | Check/Uncheck a task                          |
| help      | Print this message or the help of the given subcommand(s) |

### Options

| Option             | Description      |
|--------------------|------------------|
| -h, --help         | Print help       |


## Usage/Examples

`todo add task` adds a new task with priority 0

`todo add high_priority_task 10` adds a new task with priority 10

`todo` (does the same as `todo list`) lists the tasks

`todo rename 0 normal_task` renames the first task

`todo priority 0 2` changes the priority of the first task to 2

`todo remove 0`

## Crates used
- serde_json : serialization to json
- colored : colored terminal output
- clap : command line argument parser
- thiserror : custom errors / unified erros
- dialoguer : confirmation message

