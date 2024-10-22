use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    prelude::Span,
    style::{
        palette::tailwind::{AMBER, TEAL},
        Color, Modifier, Style, Stylize,
    },
    symbols::{self},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};

use crate::Todo;
use color_eyre::Result;

use crate::ui;

#[derive(Debug)]
pub struct App {
    list: Todo,
    exit: bool,
    state: ListState,
    pub edit: bool,
    pub edit_name: String,
    pub edit_priority: u8,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
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

    pub fn get_list(&self) -> &Todo{
        &self.list
    }

    pub fn get_state(&mut self) -> &mut ListState{
        &mut self.state
    }

    pub fn get_selected(&self) -> Option<usize>{
        self.state.selected()
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
}

