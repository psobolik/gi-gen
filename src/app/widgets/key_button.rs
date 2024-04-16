/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-16
 */

use std::fmt::{Display, Formatter};

use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::WidgetRef;

use widgets::task_action::TaskAction;

use crate::app::widgets;

#[derive(Clone)]
pub struct KeyButton {
    action: TaskAction,
    style: Style,
    position: u16,
}
impl KeyButton {
    pub fn new(action: TaskAction) -> Self {
        let style = Style::default();
        Self { action, style, position: u16::default() }
    }
    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }
    pub fn position(&mut self, position: u16) -> &mut Self {
        self.position = position;
        self
    }
    pub fn len(&self) -> usize {
        self.to_string().len()
    }
    pub fn key_event(&self) -> &KeyEvent {
        self.action.key_event()
    }
    pub fn hit_test(&self, column: u16) -> bool {
        let start = self.position;
        let end = start + self.len() as u16;
        column > start && column <= end
    }
}
impl WidgetRef for KeyButton {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, self.to_string(), self.style);
    }
}
impl Display for KeyButton {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.action.text())
    }
}
