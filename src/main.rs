use clap::{Parser, Subcommand};
mod todo;

use crate::todo::Todo;

/// A basic Todo app in Rust
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "todo")]
#[command(about = "A basic Todo app in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    ///List tasks
    List,
    ///Add task to list
    #[command(arg_required_else_help = true)]
    Add {
        /// Name of the task
        task_name: String,
        ///Priority of the task
        #[arg(default_value_t = 0)]
        priority: u8,
    },
    ///Remove tasks from list by indexes  
    #[command(arg_required_else_help = true)]
    Remove {
        ///List of indexes to remove
        index: Vec<usize>,
    },
    ///Rename a task
    #[command(arg_required_else_help = true)]
    Rename {
        ///Index of the task to rename
        index: usize,
        ///New name of the task
        new_name: String,
    },
    ///Set the priority of a task
    #[command(arg_required_else_help = true)]
    Priority {
        ///Index of the task to change
        index: usize,
        ///New priority of the task
        new_priority: u8,
    },
    ///Clear the list
    Clear,
    ///Check/Uncheck a task
    #[command(arg_required_else_help = true)]
    Done {
        ///Index of the task to update
        index: usize,
    },
}

fn execute_cmd(cmd: &Commands, list:&mut Todo){
    match &cmd {
        Commands::Add{task_name,priority} => {list.add(task_name,*priority); list.list();},
        Commands::Remove{index} => {
            match list.remove_vec(index) {
                Ok(_) => {list.list();},
                Err(_) => println!("An index out of bounds")
            }},
            Commands::Rename{index,new_name} => {list.rename(*index,new_name); list.list();}
            Commands::Clear => list.clear(),
            Commands::Done{index} => {list.done(*index); list.list();},
            Commands::Priority{index,new_priority} => {list.set_priority(*index,*new_priority); list.list();}
            _ => list.list(),
    }
}

fn main() {
    let args = Cli::parse();
    let mut list = Todo::new();
    
    match args.command {
        None => list.list(),
        Some(cmd) => execute_cmd(&cmd,&mut list),
    }
}