Basic Todo list app in Rust

Usage: todo [COMMAND]

Commands:
  list    
  add     Add task to list
  remove  Remove tasks from list by indexes
  rename  Rename a task
  clear   Clear the list
  done    Check/Uncheck a task
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help


Crates used:
- serde_json : serialization to json
- colorized : colored terminal output
- clap : command line argument parser
