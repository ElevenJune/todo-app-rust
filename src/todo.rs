use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{self, Write, Read};



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
    pub fn new(name:String, priority:u8) -> Self {
        Self{
            name:name,
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
                println!("Could no read tasks.json, initializing a new list. Error: {}",e);
                return Todo{
                    list:vec!()
                };
            }
        }
    }

    pub fn add(&mut self, name:String, priority:u8){
        println!("{} added",name);
        self.list.push(Task::new(name,priority));
        let _ = self.save();
    }

    pub fn remove_vec(&mut self, index:Vec<usize>) -> Result<(),()> {
        let mut indexes = index.clone();
        indexes.sort();
        indexes.dedup();
        for i in indexes.iter().rev() {
            if *i >= self.list.len() {
                return Err(());
            }
            self.list.remove(*i);
        }
        let _ = self.save();
        Ok(())
    }

    pub fn list(&self){
        for i in 0..self.list.len() {
            println!("{} - {}",i,self.list[i].name);
        }
    }

    pub fn clear(&mut self){
        self.list = vec!();
        match self.save() {
            Ok(_) => println!("List cleared"),
            Err(e) => println!("Error while clearing list : {}",e)
        }
    }

    pub fn rename(&mut self, index:usize, name:String){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        let old_name = self.list[index].name.clone();
        self.list[index].name = name;
        match self.save() {
            Ok(_) => println!("{} renamed to {}",old_name,self.list[index].name),
            Err(e) => println!("Error while renaming : {}",e)
        }
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

