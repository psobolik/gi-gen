/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-14
 */

use crossterm::event::KeyCode::Char;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Padding};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::list_state_wrapper::ListStateWrapper;
use crate::app::styles;
use crate::tui::event::Event;

#[derive(Default)]
pub(crate) struct TemplateList {
    has_focus: bool,
    area: Rect,
    title: String,

    event_tx: Option<UnboundedSender<Event>>,

    templates: Vec<String>,
    list_state: ListStateWrapper,
}

impl TemplateList {
    pub(crate) fn set_event_tx(&mut self, event_tx: &Option<UnboundedSender<Event>>) {
        self.event_tx.clone_from(event_tx);
    }
    pub(crate) fn title(&self) -> String {
        self.title.to_string()
    }
    pub(crate) fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }
    pub(crate) fn has_focus(&self) -> bool {
        self.has_focus
    }
    pub(crate) fn set_focus(&mut self, focus: bool) {
        self.has_focus = focus;
    }
    pub(crate) fn hit_test(&self, x: u16, y: u16) -> bool {
        self.area.contains(Position::new(x, y))
    }
    pub(crate) fn set_templates(&mut self, templates: Vec<String>) -> &mut Self {
        self.templates = templates;
        self.templates.sort();
        self.list_state.set_size(self.templates.len());
        match self.list_state.selected() {
            Some(selected) => {
                if selected > self.list_state.upper_bound {
                    self.list_state.select_last();
                } else {
                    self.list_state.set_selected(Some(selected))
                }
            }
            None => self.list_state.select_first(),
        };
        self
    }

    pub(crate) fn is_selected(&self, index: usize) -> bool {
        match self.list_state.selected() {
            Some(selected) => selected == index,
            None => false,
        }
    }
    pub(crate) fn index_from_row(&self, row: u16) -> Option<usize> {
        let index =
            row as isize - self.area.y as isize + self.list_state.list_state.offset() as isize - 1;
        if self.list_state.is_in_bounds(index) {
            Some(index as usize)
        } else {
            None
        }
    }
    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => self.send_template_select_event(),

            KeyCode::Home => self.list_state.select_first(),
            KeyCode::PageUp => self.list_state.recede_selected(self.page_size()),
            KeyCode::Up => self.list_state.recede_selected(1),

            KeyCode::End => self.list_state.select_last(),
            KeyCode::PageDown => self.list_state.advance_selected(self.page_size()),
            KeyCode::Down => self.list_state.advance_selected(1),

            key_code => {
                if let Char(ch) = key_code {
                    if ch == ' ' {
                        self.send_template_select_event();
                    }
                }
            }
        }
    }
    pub(crate) fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Up(mouse_button) => {
                if mouse_button == MouseButton::Left {
                    if let Some(index) = self.index_from_row(mouse_event.row) {
                        if self.is_selected(index) {
                            let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
                            self.handle_key_event(key_event);
                        } else {
                            self.set_selected(index);
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
                self.handle_key_event(key_event);
            }
            MouseEventKind::ScrollDown => {
                let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
                self.handle_key_event(key_event);
            }
            _ => {}
        }
    }
    pub(crate) fn handle_resize_event(&mut self, area: Rect) {
        self.area = area;
    }
    pub(crate) fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        self.area = area;

        let block = self.component_block();
        let list = List::new(self.list_items())
            .block(block)
            .highlight_style(styles::LIST_HIGHLIGHT_STYLE);
        frame.render_stateful_widget(list, area, &mut self.list_state.list_state)
    }
}
impl TemplateList {
    fn send_template_select_event(&self) {
        if let Some(selected) = self.list_state.selected() {
            // Signal that the template was selected
            let template = self.templates[selected].to_string();
            self.event_tx
                .as_ref()
                .unwrap()
                .send(Event::TemplateSelect(template))
                .expect("Panic sending template select event");
        }
    }
    fn set_selected(&mut self, index: usize) -> bool {
        if Some(index) == self.list_state.selected() {
            false // Don't bother if the item is already selected
        } else {
            self.list_state.set_selected(Some(index));
            true
        }
    }
    fn list_items<'a>(&mut self) -> Vec<ListItem<'a>> {
        self.templates
            .iter()
            .map(|t| ListItem::new(t.to_string()))
            .collect()
    }
    fn page_size(&self) -> usize {
        (self.area.height - 2) as usize
    }
    fn component_block<'a>(&self) -> Block<'a> {
        if self.has_focus {
            Self::focused_block()
        } else {
            Self::default_block()
        }
        .title(self.title())
    }
}
impl TemplateList {
    fn focused_block<'a>() -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(styles::FOCUSED_BLOCK_STYLE)
            .padding(Padding::horizontal(1))
            .title_style(styles::FOCUSED_TITLE_STYLE)
    }
    fn default_block<'a>() -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(styles::DEFAULT_BLOCK_STYLE)
            .padding(Padding::horizontal(1))
            .title_style(styles::DEFAULT_TITLE_STYLE)
    }
}
