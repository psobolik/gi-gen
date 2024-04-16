/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-14
 */
use std::fmt::{Display, Formatter};

use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

struct TaskAction {
    key_event: KeyEvent,
    text: String,
}

impl TaskAction {
    pub fn new(key_event: KeyEvent, text: String) -> Self {
        Self { key_event, text }
    }
}

#[derive(Default)]
pub(crate) struct TaskBar {
    buttons: Vec<TaskButton>,
    style: Style,
}

impl TaskBar {
    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }
    pub(crate) fn add_task(&mut self, key_event: KeyEvent, text: &str) -> &mut Self {
        self.buttons
            .push(TaskButton::new(key_event, text, self.style));
        self
    }
    pub(crate) fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let block = Block::new().style(self.style);
        frame.render_widget(block, area);

        let mut area = Rect::new(area.x + 1, area.y, 1, 1);
        for button in &self.buttons {
            area.width = button.len() as u16;
            frame.render_widget(button, area);
            area.x += area.width + 1;
        }
    }
    pub(crate) fn hit_test(&self, column: u16) -> Option<KeyEvent> {
        let mut start: u16 = 1;
        let mut end: u16;
        for button in &self.buttons {
            end = start + button.len() as u16;
            if column >= start && column < end {
                return Some(button.key_event());
            }
            start = end + 1;
        }
        None
    }
}
pub struct TaskButton {
    action: TaskAction,
    style: Style,
}

impl TaskButton {
    pub fn new(key_event: KeyEvent, text: &str, style: Style) -> Self {
        Self {
            action: TaskAction::new(key_event, text.to_string()),
            style,
        }
    }
    pub fn len(&self) -> usize {
        self.to_string().len()
    }
    pub fn key_event(&self) -> KeyEvent {
        self.action.key_event
    }
}

impl WidgetRef for TaskButton {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, self.to_string(), self.style);
    }
}

impl Display for TaskButton {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.action.text)
    }
}
