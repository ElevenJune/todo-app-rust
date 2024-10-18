use clap::{Parser, Subcommand};
//use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use todo::TodoFileError;
mod todo;

//Ratatui
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};

use crate::todo::Todo;

/// A basic Todo app in Rust
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "todo")]
#[command(about = "A basic Todo app in Rust", long_about = None)]
#[command(
    about,
    version,
    after_help = "The data are stored in $TODO_PATH\n\
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

fn execute_cmd(cmd: &Commands, list: &mut Todo) {
    match cmd {
        Commands::Add {
            task_name,
            priority,
        } => {
            list.add(task_name, *priority);
            list.list();
        }
        Commands::Remove { index } => match list.remove(index) {
            Ok(_) => {
                list.list();
            }
            Err(_) => println!("An index is out of bounds"),
        },
        Commands::Rename { index, new_name } => {
            list.rename(*index, new_name);
            list.list();
        }
        Commands::Clear => list.clear(),
        Commands::Done { index } => {
            list.done(*index);
            list.list();
        }
        Commands::Priority {
            index,
            new_priority,
        } => {
            list.set_priority(*index, *new_priority);
            list.list();
        }
        _ => list.list(),
    }
}

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
    let mut app : App = App::new();
    match Todo::load() {
        Ok(todo) => app.list = todo,
        Err(TodoFileError::IoError(e)) => {
            println!(
                "Could not read {}, a new empty list will be created.\nError : {}",
                Todo::load_path(),
                e
            );
            app.list = create_empty_list();
        }
        Err(TodoFileError::SerializationError(e)) => {
            let error = format!(
                "Parsing error while reading {}, a new empty list will be created.",
                Todo::load_path()
            )
            .bold()
            .red();
            println!("{}", error);
            println!("Error is : {}", e);
            println!("By creating the new list, the previous data will be erased");

            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to continue?")
                .interact()
                .unwrap()
            {
                app.list = create_empty_list();
            } else {
                println!("Exiting...");
                //return Err();
            }
        }
    };
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = app.run(terminal);
    ratatui::restore();
    app_result

    /*let args = Cli::parse();
    match args.command {
        None => list.list(),
        Some(cmd) => {
            execute_cmd(&cmd, &mut list);
            match list.save(){
                Ok(())=>{},
                Err(e)=>{
                    let error = format!("Error while saving, the action has been cancelled.\n{}",e);
                    println!("{}",error.bold().red());
                }
            }
        },
    }*/
}

const TODO_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;

#[derive(Debug)]
pub struct App {
    list: Todo,
    exit: bool,
    state: ListState,
}

impl App {
    /// runs the application's main loop until the user quits
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    pub fn new() -> Self{
        App { list: Todo::new(), exit: false, state:ListState::default() }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.toggle_status();
            }
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }

    /// Changes the status of the selected list item
    fn toggle_status(&mut self) {
        if let Some(i) = self.state.selected() {
            self.list.list[i].done = match self.list.list[i].done {
                true => false,
                false => true,
            }
        }
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ratatui List Example")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to change status, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("TODO List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .list.list
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                ListItem::from(todo_item.name.clone()).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.state.selected() {
            match self.list.task(i).done {
                true => format!("✓ DONE: {}", self.list.task(i).name),
                false => format!("☐ TODO: {}", self.list.task(i).name),
            }
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("TODO Info").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

impl From<&todo::Task> for ListItem<'_> {
    fn from(value: &todo::Task) -> Self {
        let line = match value.done {
            true => Line::styled(format!(" ☐ {}", value.name), TEXT_FG_COLOR),
            false => {
                Line::styled(format!(" ✓ {}", value.name), COMPLETED_TEXT_FG_COLOR)
            }
        };
        ListItem::new(line)
    }
}