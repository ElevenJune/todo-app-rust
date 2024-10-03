use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::path::Path;
use std::io::{Write, Read};



#[derive(Serialize, Deserialize, Debug)]
pub struct Todo{
    pub list: Vec<Task>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task{
    pub name:String,
    pub priority:u8
}

impl Task{
    pub fn new(name:String, priority:u8) -> Self {
        Self{
            name:name,
            priority:priority
        }
    }
}

impl Todo{
    const PATH: &str = "./tasks.json";


    pub fn new() -> Self {
        if !Path::new(Self::PATH).exists() {
            return Todo{
                list:vec!()
            };
        }
        let mut file = File::open(Self::PATH).unwrap();
        let mut buff = String::new();
        file.read_to_string(&mut buff).unwrap();

        let foo: Todo = serde_json::from_str(&buff).unwrap();
        //println!("Name: {}", foo.name);
        foo
        /**/
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
        self.save();
    }

    pub fn rename(&mut self, index:usize, name:String){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        println!("{} renamed to {}",self.list[index].name,name);
        self.list[index].name = name;
        let _ = self.save();
    }

    pub fn save(&self) {
        // Create the file
        let file_result = File::create(Self::PATH);
        let mut f = match file_result {
            Ok(file) => file,
            Err(e) => {
                println!("Error creating file: {}", e);
                return; // Exit the function early
            }
        };

        // Serialize the struct
        let serialized = serde_json::to_string(&self);
        match serialized {
            Ok(data) => {
                // Write to the file
                if let Err(e) = f.write_all(data.as_bytes()) {
                    println!("Error writing to file: {}", e);
                }
            }
            Err(e) => {
                println!("Error serializing data: {}", e);
            }
        }
    }
}

