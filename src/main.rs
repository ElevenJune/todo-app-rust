use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm};
mod todo;

use crate::todo::Todo;

/// A basic Todo app in Rust
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "todo")]
#[command(about = "A basic Todo app in Rust", long_about = None)]
#[command(about, version, after_help = 
    "The data are stored in $TODO_PATH\n\
    You can modify it by calling 'export TODO_PATH=...' (Linux)"
)]
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
    /*#[command(arg_required_else_help = true)]
    SetPath {
        ///path where to save and load
        path: String,
    },*/
}

fn execute_cmd(cmd: &Commands, list:&mut Todo){
    match cmd {
        Commands::Add{task_name,priority} => {list.add(task_name,*priority); list.list();},
        Commands::Remove{index} => {
            match list.remove(index) {
                Ok(_) => {list.list();},
                Err(_) => println!("An index is out of bounds")
            }},
        Commands::Rename{index,new_name} => {list.rename(*index,new_name); list.list();}
        Commands::Clear => list.clear(),
        //Commands::SetPath { path } => {Todo::set_path(path); let _ = list.save();}
        Commands::Done{index} => {list.done(*index); list.list();},
        Commands::Priority{index,new_priority} => {list.set_priority(*index,*new_priority); list.list();}
        _ => list.list(),
    }
}

fn main() {    
    let mut list:Todo;
    match Todo::load(){
        Some(todo) => list=todo,
        None => {
            println!("Could not read {}, a new empty list will be created.",Todo::load_path());
            println!("By creating the new list, the previous data will be erased");

            if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to continue?")
            .interact()
            .unwrap()
            {
                let new = Todo::new();
                if let Err(save_error) = new.save() {
                    println!("Failed to save the new list: {}", save_error);
                } else {
                    println!("New empty list generated");
                }
                list=new;
        } else {
            println!("Exiting...");
            return;
        }

        }
    };
    list.enable_autosave();

    let args = Cli::parse();
    match args.command {
        None => list.list(),
        Some(cmd) => {
            execute_cmd(&cmd,&mut list)
        }
    }
}

//Improvements
//1) Better error handling
//2) handle path to save file as argument
//3) function to disable auto-save