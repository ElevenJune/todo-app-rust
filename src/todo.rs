use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{self, Write, Read};
use colored::*;

//==== Task

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

    pub fn to_formated_string(&self) -> ColoredString{
        let mut displayed_name = self.name.normal();
        if self.done {
            displayed_name = displayed_name.strikethrough();
        }
        displayed_name = match self.priority{
            0..=2 => displayed_name,
            3..=6 => displayed_name.red(),
            _ => displayed_name.red().bold(),
        };
        displayed_name
    }
}

//==== Todo

#[derive(Serialize, Deserialize, Debug)]
pub struct Todo{
    pub list: Vec<Task>,
    autosave : bool
}

impl Todo{
    pub const PATH: &str = "./tasks.json";


    pub fn new() -> Self {
        Todo{
            list:vec!(),
            autosave:false
        }
    }

    pub fn load() -> Option<Self>{
        match Self::read_from_file(Self::PATH){
            Ok(todo)=>return Some(todo),
            Err(_e)=>return None
        }
    }

    pub fn enable_autosave(&mut self){
        self.autosave = true;
    }

    pub fn add(&mut self, name:&String, priority:u8){
        println!("{} added",name);
        self.list.push(Task::new(name,if priority> 10 {10} else {priority}));
        self.order_by_priority();
        if !self.autosave {return;}
        let _ = self.save();
    }

    pub fn done(&mut self, index:usize){
        if index >= self.list.len() {
            println!("Index out of bounds");
            return;
        }
        self.list[index].done = !self.list[index].done;
        if !self.autosave {return;}
        match self.save() {
            Ok(_) => println!("{} state changed",self.list[index].name),
            Err(e) => println!("Error while changing state : {}",e)
        }
    }

    pub fn remove(&mut self, index:&Vec<usize>) -> Result<(),()> {
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
        if !self.autosave {return Ok(());}
        let _ = self.save();
        Ok(())
    }

    pub fn list(&self){
        if self.list.len() == 0 {
            println!("[Empty list]");
            return;
        }
        for i in 0..self.list.len() {
            let task: &Task = &self.list[i];
            println!("{} - {} [{}]",i,task.to_formated_string(),task.priority);
        }
    }

    pub fn clear(&mut self){
        self.list = vec!();
        if !self.autosave {return;}
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
        if !self.autosave {return;}
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
        if !self.autosave {return;}
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

    pub fn save(&self) -> Result<(), std::io::Error> {

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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn new_task_feilds_are_correct(){
        let task = Task::new(&"Test".to_string(),5);
        assert_eq!(task.name,"Test");
        assert_eq!(task.priority,5);
        assert_eq!(task.done,false);
    }

    #[test]
    fn task_to_str_is_correct(){
        assert_eq!(Task::new(&"Small".to_string(),1).to_formated_string(),
        "Small".normal());
        assert_eq!(Task::new(&"Normal".to_string(),5).to_formated_string(),
        "Normal".red());
        assert_eq!(Task::new(&"Highest".to_string(),9).to_formated_string(),
        "Highest".red().bold());
    }
    
    #[test]
    fn empty_list_len_is_zero(){
        let todo = Todo::new();
        assert_eq!(todo.list.len(), 0);
    }

    #[test]
    fn add_task_increases_length() {
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        assert_eq!(todo.list.len(), 1);
    }

    #[test]
    fn added_task_correct_info() {
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        assert_eq!(todo.list[0].name,"Task1".to_string());
        assert_eq!(todo.list[0].priority,2);
        assert_eq!(todo.list[0].done,false);
    }

    #[test]
    fn rename_priority_and_done_updates(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.rename(0,&"Task2".to_string());
        assert_eq!(todo.list[0].name,"Task2".to_string());
        todo.set_priority(0,5);
        assert_eq!(todo.list[0].priority,5);
        todo.done(0);
        assert_eq!(todo.list[0].done,true);
    }

    #[test]
    fn serialize_and_deserialize_ok(){
        let mut todo = Todo::new();
        todo.enable_autosave();
        todo.add(&"Task1".to_string(),2);
        let todo_read = Todo::read_from_file(Todo::PATH).unwrap();
        assert_eq!(todo.list[0].name,todo_read.list[0].name);
        assert_eq!(todo.list[0].priority,todo_read.list[0].priority);
        assert_eq!(todo.list[0].done,todo_read.list[0].done);
    }

    #[test]
    fn remove_task(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.add(&"Task2".to_string(),2);
        assert_eq!(todo.remove(&vec!(0)),Ok(()));
        assert_eq!(todo.list.len(),1);
        assert_eq!(todo.list[0].name,"Task2".to_string());
    }

    #[test]
    fn remove_task_out_of_bounds(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.add(&"Task2".to_string(),2);
        assert_eq!(todo.remove(&vec!(0,2)),Err(()));
    }

    #[test]
    fn clear_list(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.add(&"Task2".to_string(),2);
        todo.clear();
        assert_eq!(todo.list.len(),0);
    }

    #[test]
    fn order_by_priority(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.add(&"Task2".to_string(),5);
        todo.add(&"Task3".to_string(),1);
        todo.order_by_priority();
        assert_eq!(todo.list[0].name,"Task2".to_string());
        assert_eq!(todo.list[1].name,"Task1".to_string());
        assert_eq!(todo.list[2].name,"Task3".to_string());
    }

    #[test]
    fn set_priority_out_of_bounds(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.set_priority(1,5);
        assert_eq!(todo.list[0].priority,2);
    }
}