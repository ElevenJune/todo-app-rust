use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Write, Read};
use std::env;
use colored::*;
use thiserror::Error;

//==== Task

#[derive(Serialize, Deserialize, Debug)]
///Represents a task with a name, a priority and a state
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

    ///Returns a colored string representing the task
    pub fn to_formated_string(&self) -> ColoredString{
        let mut displayed_name = self.name.normal();
        if self.done {
            return displayed_name.strikethrough();
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
    pub list: Vec<Task>
}

#[derive(Debug, Error)]
pub enum TodoFileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl Todo{
    pub const DEFAULT_PATH: &str = "./tasks.json";
    pub const PATH_VAR: &str = "TODO_PATH";


    pub fn new() -> Self {
        Todo{
            list:vec!()
        }
    }

    pub fn items(&self) -> &Vec<Task>{
        &self.list
    }

    pub fn task(&self, i:usize)->&Task{
        &self.list[i]
    }

    pub fn load() -> Result<Self,TodoFileError>{
        let path = Self::load_path();
        Self::read_from_file(path.as_str())
    }

    pub fn add(&mut self, name:&String, priority:u8){
        self.list.push(Task::new(name,if priority> 10 {10} else {priority}));
        self.sort_list();
    }

    pub fn done(&mut self, index:usize){
        if index >= self.list.len() {
            return;
        }
        self.list[index].done = !self.list[index].done;
        self.sort_list();
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
        self.sort_list();
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
    }

    pub fn rename(&mut self, index:usize, name:&String){
        if index >= self.list.len() {
            return;
        }
        self.list[index].name = name.clone();
    }

    pub fn set_priority(&mut self, index:usize, priority:u8){
        if index >= self.list.len() {
            return;
        }
        self.list[index].priority = priority;
        self.sort_list();
    }

    pub fn save_to(&self, path:String) -> Result<(), TodoFileError> {
        //let path = Self::load_path();
        // Create/open the file
        let mut f = File::create(path)?;

        // Serialize the struct
        let serialized = serde_json::to_string(&self)?;

        // Write to file
        f.write_all(serialized.as_bytes())?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), TodoFileError> {
        let path = Self::load_path();
        self.save_to(path)
    }

    pub fn load_path() -> String {
        match env::var(Self::PATH_VAR) {
            Ok(val) => val,
            Err(_e) => Self::DEFAULT_PATH.to_string()
        }
    }

// ---- Private
    fn sort_list(&mut self){
        self.list.sort_by(|a, b| {
            if a.done != b.done {
                a.done.cmp(&b.done)
            }else{
                b.priority.cmp(&a.priority)
            }
        });
    }

    fn read_from_file(path: &str) -> Result<Todo, TodoFileError> {
        let mut file = File::open(path)?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        let todo: Todo = serde_json::from_str(&buff)?;
            /*.map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                format!("Failed to parse JSON: {}", e)))?;*/
        Ok(todo)
    }

    /*pub fn set_path(path: &str) {
        env::set_var(Self::PATH_VAR, path);
    }*/
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
        todo.add(&"Task1".to_string(),2);
        let save_path = "./test.json";
        match todo.save_to(save_path.to_string()){
            Err(_e)=> assert!(false),
            _ => {}
        };
        let todo_read = Todo::read_from_file(&save_path).unwrap();
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
    fn sort_list(){
        let mut todo = Todo::new();
        todo.add(&"Task1".to_string(),2);
        todo.add(&"Task2".to_string(),5);
        todo.add(&"Task3".to_string(),1);
        todo.sort_list();
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

    /*#[test]
    fn set_path_var(){
        let previous = Todo::load_path();
        let new = "./test.json";
        Todo::set_path(new);
        assert_eq!(Todo::load_path(),new);
        Todo::set_path(&previous);
        println!("Restored to {}",previous);
    }*/
}