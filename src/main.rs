use std::str::FromStr;

use clap::{Parser, Subcommand};
//use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use todo::TodoFileError;
mod todo;

//Ratatui
use color_eyre::{owo_colors::OwoColorize, Result};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    prelude::Span,
    style::{
        palette::tailwind::{AMBER, TEAL},
        Color, Modifier, Style, Styled, Stylize,
    },
    symbols::{self, border},
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
                list = create_empty_list();
            } else {
                println!("Exiting...");
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

const TODO_HEADER_STYLE: Style = Style::new().fg(TEAL.c100).bg(TEAL.c800);
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
const EDIT_ROW_COLOR: Color = AMBER.c700;
const EDIT_VALUE_COLOR: Color = AMBER.c500;
const HIGH_PRIORITY_ROW: Style = Style::new().fg(TEAL.c100).add_modifier(Modifier::BOLD);
const EDIT_STYLE: Style = Style::new().bg(EDIT_ROW_COLOR).add_modifier(Modifier::BOLD).fg(AMBER.c100);
const EDIT_VALUE_STYLE: Style = Style::new().bg(EDIT_VALUE_COLOR).add_modifier(Modifier::BOLD).fg(AMBER.c100);
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = TEAL.c200;
const TEXT_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);

#[derive(Debug)]
pub struct App {
    list: Todo,
    exit: bool,
    state: ListState,
    edit: bool,
    edit_name: String,
    edit_priority: u8,
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

    pub fn new(todo: Todo) -> Self {
        App {
            list: todo,
            exit: false,
            state: ListState::default(),
            edit: false,
            edit_name: String::new(),
            edit_priority: 0,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.edit {
            match key.code {
                KeyCode::Char('+') => self.change_priority(true),
                KeyCode::Char('-') => self.change_priority(false),
                KeyCode::Char(c) => self.add_text(c),
                KeyCode::Backspace => self.erase_text(),
                KeyCode::Enter => self.toggle_edit_mode(false),
                KeyCode::Esc => self.toggle_edit_mode(true),
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                let _ = self.list.save();
                self.exit = true
            }
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('a') => self.add_task(),
            KeyCode::Char('r') => self.remove_task(),
            KeyCode::Enter => self.toggle_edit_mode(false),
            KeyCode::Char('l') | KeyCode::Right => {
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
        if let Some(index) = self.list.items().len().checked_sub(1) {
            self.state.select(Some(index));
        }
    }

    fn add_task(&mut self) {
        self.list.add(&"New".to_string(), 0);
        self.select_last();
        self.toggle_edit_mode(false);
    }

    fn remove_task(&mut self) {
        if let Some(i) = self.state.selected() {
            let _ = self.list.remove(&vec![i]);
        }
    }

    fn toggle_edit_mode(&mut self, cancel:bool) {
        let current_task;
        let current_task_index: usize;
        match self.state.selected() {
            Some(i) => {
                current_task = self.list.task(i);
                current_task_index = i;
            }
            None => return,
        }
        if cancel {
            self.edit = false;
            return;
        } else {
            self.edit = !self.edit;
        }
        
        if self.edit {
            //start editing
            self.edit_name = current_task.name.clone();
            self.edit_priority = current_task.priority;
        } else {
            //edit finished
            self.list
                .rename(current_task_index, &self.edit_name.clone());
            self.list
                .set_priority(current_task_index, self.edit_priority);
        }
    }

    /// Changes the status of the selected list item
    fn toggle_status(&mut self) {
        if let Some(i) = self.state.selected() {
            self.list.done(i);
        }
    }

    fn add_text(&mut self, text: char) -> () {
        self.edit_name.push(text);
    }

    fn change_priority(&mut self, increment: bool) -> () {
        if increment {
            if self.edit_priority==10 {return;}
            self.edit_priority += 1;
        } else {
            if self.edit_priority<=0 {return;}
            self.edit_priority -= 1;
        }
    }

    fn erase_text(&mut self) {
        self.edit_name.pop();
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Todo List Application")
            .bold()
            .centered()
            .bg(TEAL.c500)
            .render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let text = if self.edit {
            "[Edit Mode]\nSave with Enter, Cancel with Esc\n-/+ to change priority, type to change name"
        } else {
            "Use ↓↑ to move, ← to unselect, → to change status\n'a' to add a task. 'r' to remove a task"
        };
        Paragraph::new(text)
        .centered()
        .bg(AMBER.c100)
        .fg(EDIT_ROW_COLOR)
        .bold()
        .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Task List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .list
            .list
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                let displayed_name = todo_item.name.clone();
                let mut item = ListItem::from(displayed_name).bg(color);
                if todo_item.done {
                    item = item.add_modifier(Modifier::CROSSED_OUT);
                }
                else if todo_item.priority>5 {
                    item = item.style(HIGH_PRIORITY_ROW);
                }
                /*match self.state.selected(){
                    Some(index) => if i==index {item = item.bg()}
                    None => {}
                }*/
                item
            })
            .collect();

        let mut selected_style = SELECTED_STYLE;
        let mut symbol = " => ";
        if self.edit {

            symbol = "===>";
            selected_style=EDIT_STYLE;//.add_modifier(Modifier::REVERSED);
        };

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol(symbol)
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let mut text: Vec<Line<'_>> = vec![];
        let border_style = if self.edit { EDIT_STYLE } else { TODO_HEADER_STYLE };

        match self.state.selected() {
            Some(i) => {
                let task = self.list.task(i);
                let style = if self.edit { EDIT_VALUE_STYLE } else { TEXT_STYLE };

                let mut name_line = vec!["Name : ".red()];

                let mut priority_line = vec!["Priority : ".red()];

                let state_line = vec![
                    "State : ".red(),
                    Span::styled(format!("{}", task.done), TEXT_STYLE),
                ];

                if self.edit {
                    name_line.push(Span::styled(&self.edit_name, style));
                    priority_line.push(Span::styled(format!("{}", self.edit_priority), style));
                    name_line.push("_".fg(EDIT_VALUE_COLOR).add_modifier(Modifier::BOLD));
                    priority_line.push(" (-/+)".fg(EDIT_VALUE_COLOR).bold());
                    //bg = SELECTED_STYLE;
                } else {
                    name_line.push(Span::styled(&task.name, style));
                    priority_line.push(Span::styled(format!("{}", task.priority), TEXT_STYLE));
                }

                text.push(Line::from(name_line));
                text.push(Line::from(priority_line));
                text.push(Line::from(state_line));
            }
            None => {
                text.push(Line::styled("Select a task", Style::new().gray().italic()));
            }
        }

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Task Information").centered())
            .borders(Borders::all())
            .border_set(symbols::border::EMPTY)
            .border_style(border_style)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let footer_length = if self.edit {3} else {2};
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(footer_length),
        ])
        .areas(area);

        let info_weight = if self.edit {2} else {1};
        let [list_area, item_area] =
            Layout::horizontal([Constraint::Fill(3-info_weight), Constraint::Fill(info_weight)]).areas(main_area);

        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
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
        let line = Line::styled(
            format!(
                "Name:{}\nPriority:{}\nDone:{}",
                value.name, value.priority, value.done
            ),
            TEXT_FG_COLOR,
        );
        ListItem::new(line)
    }
}
