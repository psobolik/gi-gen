/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-11
 */

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::*;
use regex::Regex;
use tokio::sync::mpsc::UnboundedSender;

use panes::filter::Filter as FilterPane;
use panes::task_bar::TaskBar as TaskBarPane;
use panes::template_list::TemplateList as TemplateListPane;
use popups::about as about_popup;
use popups::error as error_popup;
use popups::help as help_popup;
use popups::save_option::SaveOption as SaveOptionPopup;
use popups::save_option::SaveOptions;

use crate::gitignore_api;
use crate::tui::event::Event;
use crate::util;

mod list_state_wrapper;
mod panes;
mod popups;
mod styles;

#[derive(Default)]
struct FrameSet {
    available: Rect,
    selected: Rect,
    task_bar: Rect,
    filter: Rect,
}

#[derive(Default)]
struct FilterStatus {
    hidden: bool,
    selected: bool,
}

enum PopupFlag {
    Error(String),
    Help,
    About,
    SaveOption,
}

#[derive(Default)]
pub(crate) struct App {
    should_quit: bool,

    templates: HashMap<String, FilterStatus>,

    filter_pane: FilterPane,
    available_pane: TemplateListPane,
    selected_pane: TemplateListPane,
    task_bar: TaskBarPane,

    save_option_popup: SaveOptionPopup,

    save_option_flag: Option<bool>,
    popup_flag: Option<PopupFlag>,
    frame_set: FrameSet,
}

impl App {
    pub(crate) fn set_event_tx(&mut self, event_tx: Option<UnboundedSender<Event>>) -> &mut Self {
        self.available_pane.set_event_tx(&event_tx);
        self.selected_pane.set_event_tx(&event_tx);
        self
    }
    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub(crate) async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Init => self.handle_init_event().await,
            Event::Key(key_event) => self.handle_key_event(key_event).await,
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event).await,
            Event::Resize(width, height) => self.handle_resize_event(width, height),
            Event::TemplateSelect(template) => self.toggle_selection(template),
            _ => { /* ignore */ }
        }
    }
    pub(crate) fn render(&mut self, frame: &mut Frame<'_>) {
        self.frame_set = Self::calculate_frames(frame.size());

        self.available_pane.render(self.frame_set.available, frame);
        self.selected_pane.render(self.frame_set.selected, frame);
        self.filter_pane.render(self.frame_set.filter, frame);
        self.task_bar.render(self.frame_set.task_bar, frame);

        if let Some(popup_message) = &self.popup_flag {
            match popup_message {
                PopupFlag::Error(message) => error_popup::render(message, frame),
                PopupFlag::Help => help_popup::render(frame),
                PopupFlag::About => about_popup::render(frame),
                PopupFlag::SaveOption => self.save_option_popup.render(frame),
            }
        }
    }
}

impl App {
    const ABOUT_CHAR: char = 'a';
    const SAVE_CHAR: char = 's';
    const QUIT_CHAR: char = 'q';
    const NEXT_CHAR: char = 'n';
    const PREV_CHAR: char = 'p';
    const HELP_KEY_CODE: KeyCode = KeyCode::F(1);
    const HELP_KEY_EVENT: KeyEvent = KeyEvent::new(Self::HELP_KEY_CODE, KeyModifiers::NONE);
    const ABOUT_KEY_EVENT: KeyEvent = KeyEvent::new(Char(Self::ABOUT_CHAR), KeyModifiers::CONTROL);
    const SAVE_KEY_EVENT: KeyEvent = KeyEvent::new(Char(Self::SAVE_CHAR), KeyModifiers::CONTROL);
    const QUIT_KEY_EVENT: KeyEvent = KeyEvent::new(Char(Self::QUIT_CHAR), KeyModifiers::CONTROL);

    async fn handle_init_event(&mut self) {
        self.available_pane.set_title("Available Templates");
        self.selected_pane.set_title("Selected Templates");
        match gitignore_api::get_template_names().await {
            Ok(templates) => {
                for template in templates {
                    self.templates.insert(template, FilterStatus::default());
                }
                self.set_templates();
            }
            Err(error) => eprintln!("Error: {}", error),
        }
        self.available_pane.set_focus(true);
        self.selected_pane.set_focus(false);

        self.task_bar.push_task(Self::HELP_KEY_EVENT, "F1 Help");
        self.task_bar.push_task(Self::ABOUT_KEY_EVENT, "^A About");
        self.task_bar.push_task(Self::SAVE_KEY_EVENT, "^S Save");
        self.task_bar.push_task(Self::QUIT_KEY_EVENT, "^Q Quit");
    }
    fn set_templates(&mut self) {
        self.available_pane
            .set_templates(self.available_templates());
        self.selected_pane.set_templates(self.selected_templates());
    }
    fn available_templates(&self) -> Vec<String> {
        self.templates
            .iter()
            .filter(|(_template, status)| !status.hidden && !status.selected)
            .map(|(template, _status)| template.to_string())
            .collect()
    }
    fn selected_templates(&self) -> Vec<String> {
        self.templates
            .iter()
            .filter(|(_template, status)| status.selected)
            .map(|(template, _status)| template.to_string())
            .collect()
    }
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Ctrl+C closes the app, regardless of its state
        if Char('c') == key_event.code && key_event.modifiers == KeyModifiers::CONTROL {
            self.quit();
            return;
        }
        // If the save options popup is showing, it handles all keys
        if let Some(PopupFlag::SaveOption) = self.popup_flag {
            match key_event.code {
                KeyCode::Esc => self.cancel_save(),
                Char(ch) => {
                    match self.save_option_popup.key_test(ch) {
                        None => { /* ignore unrecognized keys */ }
                        Some(save_option) => self.handle_save_option(save_option).await,
                    }
                }
                _ => { /* ignore other keys */ }
            }
        } else if !self.maybe_clear_message() {
            // If there's another popup, any key event will clear it and stop processing the event
            match key_event.code {
                KeyCode::Tab => self.toggle_focus(),
                KeyCode::Enter
                | KeyCode::Home
                | KeyCode::PageUp
                | KeyCode::Up
                | KeyCode::End
                | KeyCode::PageDown
                | KeyCode::Down
                | Char(' ') => self.pane_handle_key_event(key_event),
                KeyCode::Esc | KeyCode::Delete => {
                    self.filter_pane.clear();
                    self.apply_filter();
                }
                KeyCode::Backspace => {
                    self.filter_pane.delete_last();
                    self.apply_filter();
                }
                Self::HELP_KEY_CODE => self.set_help_popup_flag(),
                Char(ch) => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        match ch {
                            Self::QUIT_CHAR => self.quit(),
                            Self::ABOUT_CHAR => self.set_about_popup_flag(),
                            Self::SAVE_CHAR => self.save().await,
                            Self::PREV_CHAR => self.pane_handle_key_event(KeyEvent::new(
                                KeyCode::Up,
                                KeyModifiers::NONE,
                            )),
                            Self::NEXT_CHAR => self.pane_handle_key_event(KeyEvent::new(
                                KeyCode::Down,
                                KeyModifiers::NONE,
                            )),
                            _ => { /* ignore other control chars */ }
                        }
                    } else {
                        self.filter_pane.append(ch.to_ascii_lowercase());
                        self.apply_filter();
                    }
                }
                _ => { /* ignore other keys */ }
            }
        }
    }
    fn pane_handle_key_event(&mut self, key_event: KeyEvent) {
        if self.available_pane.has_focus() {
            self.available_pane.handle_key_event(key_event);
        } else {
            self.selected_pane.handle_key_event(key_event);
        }
    }
    async fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        // If the save option popup is active, it handles all mouse events
        if let Some(PopupFlag::SaveOption) = self.popup_flag {
            match mouse_event.kind {
                MouseEventKind::Up(mouse_button) => {
                    if mouse_button == MouseButton::Left {
                        match self
                            .save_option_popup
                            .hit_test(mouse_event.column, mouse_event.row)
                        {
                            None => { /* ignore random clicks */ }
                            Some(save_option) => self.handle_save_option(save_option).await,
                        }
                    }
                }
                _ => { /* ignore other events */ }
            }
        } else {
            match mouse_event.kind {
                MouseEventKind::Up(mouse_button) => {
                    if mouse_button == MouseButton::Left && !self.maybe_clear_message() {
                        // If there's another popup, a mouse up event might close it and stop processing the event.
                        if self
                            .available_pane
                            .hit_test(mouse_event.column, mouse_event.row)
                            && !self.available_pane.has_focus()
                            || self
                                .selected_pane
                                .hit_test(mouse_event.column, mouse_event.row)
                                && !self.selected_pane.has_focus()
                        {
                            self.toggle_focus();
                        } else {
                            self.pane_handle_mouse_event(mouse_event).await;
                        }
                    }
                }
                _ => self.pane_handle_mouse_event(mouse_event).await,
            }
        }
    }
    async fn pane_handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        if util::is_in_rect(mouse_event.column, mouse_event.row, self.frame_set.task_bar) {
            match mouse_event.kind {
                MouseEventKind::Up(mouse_button) => {
                    if mouse_button == MouseButton::Left {
                        // Task bar click may be translated into a key press
                        if let Some(key_event) = self.task_bar.key_test(mouse_event.column) {
                            self.handle_key_event(key_event).await;
                        }
                    }
                }
                _ => { /* ignore other events */ }
            }
        } else if self.available_pane.has_focus() {
            self.available_pane.handle_mouse_event(mouse_event);
        } else if self.selected_pane.has_focus() {
            self.selected_pane.handle_mouse_event(mouse_event);
        }
    }
    fn handle_resize_event(&mut self, width: u16, height: u16) {
        self.frame_set = Self::calculate_frames(Rect::new(0, 0, width, height));
        self.available_pane
            .handle_resize_event(self.frame_set.available);
        self.selected_pane
            .handle_resize_event(self.frame_set.selected);
    }
    fn quit(&mut self) {
        self.should_quit = true;
    }
    async fn handle_save_option(&mut self, save_option: SaveOptions) {
        match save_option {
            SaveOptions::Replace => self.overwrite_save().await,
            SaveOptions::Append => self.append_save().await,
            SaveOptions::Cancel => self.cancel_save(),
        }
    }
    async fn overwrite_save(&mut self) {
        // Cancel the save option popup and call save again, this time with the overwrite flag set
        self.save_option_flag = Some(true);
        self.popup_flag = None;
        self.save().await;
    }
    async fn append_save(&mut self) {
        // Cancel the save option popup and call save again, this time with the overwrite flag clear
        self.save_option_flag = Some(false);
        self.popup_flag = None;
        self.save().await;
    }
    fn cancel_save(&mut self) {
        // Just cancel the save option popup
        self.popup_flag = None;
    }
    async fn save(&mut self) {
        let selected_templates = self.selected_templates();
        if selected_templates.is_empty() {
            self.set_error_popup_flag("Select one or more templates and try again.");
        } else {
            match gitignore_api::get_template(&selected_templates).await {
                Ok(result) => {
                    let output_file = Path::new("./.gitignore");
                    let mut open_options = OpenOptions::new();
                    let message: &str;
                    if output_file.exists() {
                        match self.save_option_flag {
                            Some(overwrite_flag) => {
                                if overwrite_flag {
                                    self.save_option_flag = None;
                                    open_options.truncate(true).write(true);
                                    message = "Replaced contents of existing .gitignore file.";
                                } else {
                                    self.save_option_flag = None;
                                    open_options.append(true);
                                    message = "Appended templates to existing .gitignore file.";
                                }
                            }
                            None => {
                                self.set_save_option_popup_flag();
                                return;
                            }
                        }
                    } else {
                        open_options.create(true).write(true);
                        message = "Created new .gitingore file.";
                    }
                    match open_options.open(output_file) {
                        Ok(mut file) => {
                            if let Err(error) = file.write(result.as_bytes()) {
                                self.set_error_popup_flag(error.to_string().as_str())
                            } else {
                                self.quit();
                                print!("[{}] {}", env!("CARGO_PKG_NAME"), message);
                            }
                        }
                        Err(error) => self.set_error_popup_flag(error.to_string().as_str()),
                    }
                }
                Err(error) => {
                    self.set_error_popup_flag(
                        format!(
                            r#"Problem getting .gitignore for "{}": {}"#,
                            selected_templates.join(" "),
                            error
                        )
                        .as_str(),
                    );
                }
            };
        }
    }
    fn set_error_popup_flag(&mut self, message: &str) {
        self.popup_flag = Some(PopupFlag::Error(message.to_string()));
    }
    fn set_help_popup_flag(&mut self) {
        self.popup_flag = Some(PopupFlag::Help);
    }
    fn set_about_popup_flag(&mut self) {
        self.popup_flag = Some(PopupFlag::About);
    }
    fn set_save_option_popup_flag(&mut self) {
        self.popup_flag = Some(PopupFlag::SaveOption);
    }
    // Clears the error, help or about popup if they are showing, and returns
    // true to indicate that the event was handled. Does nothing if the
    // save option popup is showing or if no popup is showing, returning false
    // to indicate that the event was not handled.
    fn maybe_clear_message(&mut self) -> bool {
        if let Some(popup_flag) = &self.popup_flag {
            return match popup_flag {
                PopupFlag::Error(_) | PopupFlag::Help | PopupFlag::About => {
                    self.popup_flag = None;
                    true
                }
                PopupFlag::SaveOption => false,
            };
        }
        false
    }
    // Finds a template by name and toggles its selected flag.
    fn toggle_selection(&mut self, template: String) {
        let filter_status = self.templates.entry(template).or_default();
        filter_status.selected = !filter_status.selected;
        self.set_templates();
    }
    fn apply_filter(&mut self) {
        // We escape the filter wo we can use the input as a regular expression, and we also ignore
        // any filter that can't be used as a regular expression. (Belt and suspenders!) This is
        // really only relevant for "c++".
        let filter = regex::escape(self.filter_pane.text());
        if let Ok(regex) = Regex::new(filter.as_str()) {
            let templates: Vec<String> = self.templates.keys().map(|t| t.to_string()).collect();
            for template in templates {
                let filter_status = self.templates.entry(template.to_string()).or_default();
                filter_status.hidden = !regex.is_match(template.as_str());
            }
            self.set_templates();
        }
    }
    fn toggle_focus(&mut self) {
        if self.available_pane.has_focus() {
            self.focus_selected();
        } else {
            self.focus_available();
        }
    }
    fn focus_available(&mut self) {
        if !self.available_pane.has_focus() {
            self.available_pane.set_focus(true);
            self.selected_pane.set_focus(false);
        }
    }
    fn focus_selected(&mut self) {
        if !self.selected_pane.has_focus() {
            self.available_pane.set_focus(false);
            self.selected_pane.set_focus(true);
        }
    }
}

impl App {
    fn calculate_frames(rect: Rect) -> FrameSet {
        let root = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(rect);
        let top = root[0];
        let middle = root[1];
        let bottom = root[2];

        let main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(middle);
        let left = main[0];
        let right = main[1];

        FrameSet {
            filter: top,
            available: left,
            selected: right,
            task_bar: bottom,
        }
    }
}
