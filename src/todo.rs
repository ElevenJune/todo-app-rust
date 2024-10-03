use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{self, Write, Read};
use colored::*;


#[derive(Serialize, Deserialize, Debug)]
pub struct Todo{
    pub list: Vec<Task>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task{
    pub name:String,
    pub priority:u8,
    pub done:bool,
}

impl Task{
    pub fn new(name:&String, priority:u8) -> Self {
        Self{
            name:name.clone(),
            priority:priority,
            done:false
        }
    }
}

impl Todo{
    const PATH: &str = "./tasks.json";


    pub fn new() -> Self {
        match Self::read_from_file(Self::PATH){
            Ok(todo)=>return todo,
            Err(e)=>{
                println!("Could no read tasks.json, a new empty list will be created. Error: {}",e);
                let new_todo = Todo{
                    list:vec!()
                };

                if let Err(save_error) = new_todo.save() {
                    println!("Failed to save the new list: {}", save_error);
                } else {
                    println!("New empty list generated");
                }

                new_todo
            }
        }
    }

    pub fn add(&mut self, name:&String, priority:u8){
        println!("{} added",name);
        self.list.push(Task::new(name,if priority> 10 {10} else {priority}));
        self.order_by_priority();
        let _ = self.save();
    }

    pub fn done(&mut self, index:usize){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        self.list[index].done = !self.list[index].done;
        match self.save() {
            Ok(_) => println!("{} state changed",self.list[index].name),
            Err(e) => println!("Error while changing state : {}",e)
        }
    }

    pub fn remove_vec(&mut self, index:&Vec<usize>) -> Result<(),()> {
        let mut indexes = index.clone();
        indexes.sort();
        indexes.dedup();
        for i in indexes.iter().rev() {
            if *i >= self.list.len() {
                return Err(());
            }
            self.list.remove(*i);
        }
        self.order_by_priority();
        let _ = self.save();
        Ok(())
    }

    pub fn list(&self){
        for i in 0..self.list.len() {
            let task: &Task = &self.list[i];
            let mut displayed_name = task.name.normal();
            if task.done{
                displayed_name = task.name.strikethrough();
            }
            displayed_name = match task.priority{
                0..=2 => displayed_name,
                3..=6 => displayed_name.red(),
                _ => displayed_name.red().bold(),
            };
            println!("{} - {} [{}]",i,displayed_name,task.priority);
        }
    }

    pub fn clear(&mut self){
        self.list = vec!();
        match self.save() {
            Ok(_) => println!("List cleared"),
            Err(e) => println!("Error while clearing list : {}",e)
        }
    }

    pub fn rename(&mut self, index:usize, name:&String){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        let old_name = self.list[index].name.clone();
        self.list[index].name = name.clone();
        match self.save() {
            Ok(_) => println!("{} renamed to {}",old_name,self.list[index].name),
            Err(e) => println!("Error while renaming : {}",e)
        }
    }

    pub fn set_priority(&mut self, index:usize, priority:u8){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        self.list[index].priority = priority;
        match self.save() {
            Ok(_) => {
                println!("{} priority changed to {}",
                self.list[index].name.clone(),
                self.list[index].priority);
                self.order_by_priority();
            }
            Err(e) => println!("Error while changing priority : {}",e)
        }
    }

    fn order_by_priority(&mut self){
        self.list.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    fn save(&self) -> Result<(), std::io::Error> {

        // Create/open the file
        let mut f = File::create(Self::PATH)?;

        // Serialize the struct
        let serialized = serde_json::to_string(&self)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Write to file
        f.write_all(serialized.as_bytes())?;

        Ok(())
    }

    fn read_from_file(path: &str) -> Result<Todo, io::Error> {
        let mut file = File::open(path)?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        let todo: Todo = serde_json::from_str(&buff)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON: {}", e)))?;
        Ok(todo)
    }
}

