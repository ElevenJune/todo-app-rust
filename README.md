
# Basic to-do app in Rust

Basic to-do app to learn Rust and use crates


## Authors

- [@elevenJune](https://github.com/ElevenJune)


## Features

__Usage: todo [COMMAND]__

Command list :

| Name   | Tab   | Description                                      |
|--------|-------|--------------------------------------------------|
| list   |       | Lists all tasks in the to-do list.              |
| add    | Add   | Adds a task to the list.                         |
| remove | Remove| Removes tasks from the list by their indexes.    |
| rename | Rename| Renames a specified task.                        |
| clear  | Clear | Clears all tasks from the list.                  |
| done   | Done  | Checks or unchecks a task as completed.         |
| help   | Help  | Prints this message or the help for specific commands. |


## Usage/Examples

- todo add test
- todo add high_priority_task 10
- todo //will call todo list

## Crates used
- serde_json : serialization to json
- colorized : colored terminal output
- clap : command line argument parser

