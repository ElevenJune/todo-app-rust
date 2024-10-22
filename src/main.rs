use dialoguer::{theme::ColorfulTheme, Confirm};
use todo::TodoFileError;
mod todo;
mod app;
mod ui;

//Ratatui
use color_eyre::Result;


use todo::Todo;
use app::App;



fn create_empty_list() -> Todo {
    let new = Todo::new();
    if let Err(save_error) = new.save() {
        println!("Failed to save the new list: {}", save_error);
    } else {
        println!("New empty list generated");
    }
    new
}

fn main() -> Result<()> {
    let list: Todo;
    match Todo::load() {
        Ok(todo) => list = todo,
        Err(TodoFileError::IoError(e)) => {
            println!(
                "Could not read {}, a new empty list will be created.\nError : {}",
                Todo::load_path(),
                e
            );
            list = create_empty_list();
        }
        Err(TodoFileError::SerializationError(e)) => {
            let error = format!(
                "Parsing error while reading {}, a new empty list will be created.",
                Todo::load_path()
            );
            //.bold()
            //.red();
            println!("{}", error);
            println!("Error is : {}", e);
            println!("By creating the new list, the previous data will be erased");

            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to continue?")
                .interact()
                .unwrap()
            {
                list = create_empty_list();
            } else {
                println!("Exiting...");
                //return Err(());
                panic!();
            }
        }
    };

    let app: App = App::new(list);

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = app.run(terminal);
    ratatui::restore();
    app_result
}