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
use ratatui::layout::Position;
use ratatui::prelude::*;
use regex::Regex;
use tokio::sync::mpsc::UnboundedSender;

use panes::template_list_pane::TemplateListPane;
use popups::about as about_popup;
use popups::error as error_popup;
use popups::help as help_popup;
use popups::save_option::SaveOption as SaveOptionPopup;
use popups::save_option::SaveOptions;
use widgets::filter::Filter;
use widgets::task_action::TaskAction;
use widgets::task_bar::TaskBar;

use crate::gitignore_api;
use crate::tui::event::Event;

mod list_state_wrapper;
mod panes;
mod popups;
mod styles;
mod widgets;

#[derive(Default)]
struct FrameSet {
    available: Rect,
    selected: Rect,
    task_bar: Rect,
    filter: Rect,
}

#[derive(Copy, Clone, Default)]
struct FilterStatus {
    hidden: bool,
    selected: bool,
}

#[derive(Clone, Debug)]
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

    filter_pane: Filter,
    available_pane: TemplateListPane,
    selected_pane: TemplateListPane,
    task_bar: TaskBar,

    save_option_popup: SaveOptionPopup,

    popup_flag: Option<PopupFlag>,
    frame_set: FrameSet,
}

impl App {
    pub(crate) fn with_event_tx(mut self, event_tx: Option<UnboundedSender<Event>>) -> Self {
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

        frame.render_widget(&self.filter_pane, self.frame_set.filter);
        frame.render_widget(&self.task_bar, self.frame_set.task_bar);

        if let Some(popup_flag) = &self.popup_flag {
            match popup_flag {
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
        match gitignore_api::get_template_names().await {
            Ok(templates) => {
                for template in templates {
                    self.templates.insert(template, FilterStatus::default());
                }
                self.set_templates();
            }
            Err(error) => self.set_error_popup_flag(error.to_string().as_str()),
        }
        self.available_pane
            .set_title("Available Templates")
            .set_focus(true);
        self.selected_pane
            .set_title("Selected Templates")
            .set_focus(false);

        self.task_bar.style(styles::TASK_BAR_STYLE).buttons(vec![
            TaskAction::new(Self::HELP_KEY_EVENT, "F1 Help"),
            TaskAction::new(Self::ABOUT_KEY_EVENT, "^A About"),
            TaskAction::new(Self::SAVE_KEY_EVENT, "^S Save"),
            TaskAction::new(Self::QUIT_KEY_EVENT, "^Q Quit"),
        ]);
    }
    fn set_templates(&mut self) {
        self.available_pane
            .set_templates(self.available_templates());
        self.selected_pane.set_templates(self.selected_templates());
    }
    fn available_templates(&self) -> Vec<String> {
        self.templates
            .iter()
            .filter_map(|(template, status)| {
                if !status.hidden && !status.selected {
                    Some(template.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
    fn selected_templates(&self) -> Vec<String> {
        self.templates
            .iter()
            .filter_map(|(template, status)| {
                if status.selected {
                    Some(template.to_string())
                } else {
                    None
                }
            })
            .collect()
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
            SaveOptions::Replace => {
                self.save(Some(true)).await;
            }
            SaveOptions::Append => {
                self.save(Some(false)).await;
            }
            SaveOptions::Cancel => {}
        }
        self.popup_flag = None;
    }
    async fn save(&mut self, overwrite_option_flag: Option<bool>) {
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
                        match overwrite_option_flag {
                            Some(overwrite_flag) => {
                                if overwrite_flag {
                                    open_options.truncate(true).write(true);
                                    message = "Replaced contents of existing .gitignore file.";
                                } else {
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
    // Finds a template by name and toggles its selected flag.
    fn toggle_selection(&mut self, template: String) {
        self.templates
            .entry(template)
            .and_modify(|status| status.selected = !status.selected);
        self.set_templates();
    }
    /// Hide templates that don't match the filter by setting their hidden status flag;
    fn apply_filter(&mut self) {
        // We escape the filter so we can use the input as a regular expression, and we also ignore
        // any filter that can't be used as a regular expression. (Belt and suspenders!) This is
        // really only relevant for "c++".
        let filter = regex::escape(self.filter_pane.text());
        if let Ok(regex) = Regex::new(filter.as_str()) {
            let templates: Vec<String> = self.templates.keys().map(|t| t.to_string()).collect();
            for template in templates {
                self.templates
                    .entry(template.to_string())
                    .and_modify(|status| {
                        status.hidden = !regex.is_match(template.as_str());
                    });
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
impl App {
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Ctrl+C closes the app, regardless of its state
        if Char('c') == key_event.code && key_event.modifiers == KeyModifiers::CONTROL {
            self.quit();
            return;
        }
        // Give a popup the chance to handle the event
        if self.popup_handle_key_event(key_event).await {
            return;
        }
        // Give the active pane the chance to handle the event
        if self.pane_handle_key_event(key_event) {
            return;
        }
        // If nobody else handled it, give the app a shot at the event
        match key_event.code {
            KeyCode::Tab => self.toggle_focus(),
            KeyCode::Esc | KeyCode::Delete => {
                self.filter_pane.clear();
                self.apply_filter();
            }
            KeyCode::Backspace => {
                self.filter_pane.pop();
                self.apply_filter();
            }
            Self::HELP_KEY_CODE => self.set_help_popup_flag(),
            Char(ch) => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    match ch {
                        Self::QUIT_CHAR => self.quit(),
                        Self::ABOUT_CHAR => self.set_about_popup_flag(),
                        Self::SAVE_CHAR => self.save(None).await,
                        _ => { /* ignore other control chars */ }
                    }
                } else {
                    self.filter_pane.push(ch.to_ascii_lowercase());
                    self.apply_filter();
                }
            }
            _ => { /* ignore other keys */ }
        }
    }
    fn pane_handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        if self.available_pane.has_focus() {
            self.available_pane.handle_key_event(key_event)
        } else {
            self.selected_pane.handle_key_event(key_event)
        }
    }
    // If a popup is open, close it and return true else return false.
    async fn popup_handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        // No popup, return false
        let Some(popup_flag) = &self.popup_flag else {
            return false;
        };
        match popup_flag {
            PopupFlag::Error(_) | PopupFlag::Help | PopupFlag::About => {
                self.popup_flag = None;
            }
            PopupFlag::SaveOption => {
                if self.save_handle_key_event(key_event).await {
                    self.popup_flag = None;
                }
            }
        }
        true // If a popup was open, eat the key
    }
    async fn save_handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Esc => {
                self.popup_flag = None;
                true
            }
            Char(ch) => {
                if let Some(save_option) = self.save_option_popup.key_test(ch) {
                    self.handle_save_option(save_option).await;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
impl App {
    async fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        // Allow a popup to handle the event
        if self.popup_handle_mouse_event(mouse_event).await {
            return;
        }
        // Allow the task bar to handle the event
        if self.task_bar_handle_mouse_event(mouse_event).await {
            return;
        }
        // If the mouse event occurs in the focused pane, the pane will have a chance to handle it.
        // Otherwise if the mouse is clicked in the unfocused pane, it will be given the focus.
        let mouse_over_available_pane = self
            .available_pane
            .hit_test(mouse_event.column, mouse_event.row);
        let mouse_over_selected_pane = self
            .selected_pane
            .hit_test(mouse_event.column, mouse_event.row);
        if self.available_pane.has_focus() {
            if mouse_over_available_pane {
                // The mouse is in the pane
                self.available_pane.handle_mouse_event(mouse_event);
            } else {
                // If the event is a left click in the selected pane, toggle the focus
                if mouse_over_selected_pane
                    && mouse_event.kind == MouseEventKind::Up(MouseButton::Left)
                {
                    self.toggle_focus();
                }
            }
        } else if self.selected_pane.has_focus() {
            if mouse_over_selected_pane {
                // The mouse is over in the pane
                self.selected_pane.handle_mouse_event(mouse_event);
            } else {
                // If the event is a left click outside the selected pane, toggle the focus
                if mouse_over_available_pane
                    && mouse_event.kind == MouseEventKind::Up(MouseButton::Left)
                {
                    self.toggle_focus();
                }
            }
        }
    }
    async fn task_bar_handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        // Only handle left mouse up events
        if mouse_event.kind != MouseEventKind::Up(MouseButton::Left) {
            return false;
        }
        // Ignore events that are not on the taskbar
        if !self
            .frame_set
            .task_bar
            .contains(Position::new(mouse_event.column, mouse_event.row))
        {
            return false;
        }
        // Task bar click may be translated into a key press
        if let Some(key_event) = self.task_bar.hit_test(mouse_event.column) {
            self.handle_key_event(key_event).await;
            true
        } else {
            false
        }
    }
    // If a popup is open, close it and return true else return false.
    async fn popup_handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        // Only handle left mouse up events
        if mouse_event.kind != MouseEventKind::Up(MouseButton::Left) {
            return false;
        }
        // No popup, return false
        let Some(popup_flag) = &self.popup_flag else {
            return false;
        };
        match popup_flag {
            PopupFlag::Error(_) | PopupFlag::Help | PopupFlag::About => {
                self.popup_flag = None;
                true
            }
            PopupFlag::SaveOption => self.save_popup_handle_mouse_event(mouse_event).await,
        }
    }
    // If the save option popup is open it may handle the mouse event.
    // The function will return true if it does.
    async fn save_popup_handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        let Some(PopupFlag::SaveOption) = &self.popup_flag else {
            return false;
        };
        let Some(save_option) = self
            .save_option_popup
            .hit_test(mouse_event.column, mouse_event.row)
        else {
            return false;
        };
        self.handle_save_option(save_option).await;
        true
    }
}
