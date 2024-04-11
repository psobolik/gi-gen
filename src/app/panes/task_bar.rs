/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-14
 */
use std::fmt::Display;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::Frame;

use crate::app::styles;

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
    actions: Vec<TaskAction>,
}

impl TaskBar {
    const SEPARATOR: &'static str = " | ";

    pub(crate) fn push_task(&mut self, key_event: KeyEvent, text: &str) {
        self.actions
            .push(TaskAction::new(key_event, text.to_string()));
    }
    pub(crate) fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .style(styles::TASK_BAR_STYLE);
        let paragraph = Paragraph::new(self.to_string()).block(block);
        frame.render_widget(paragraph, area);
    }
    pub(crate) fn key_test(&self, column: u16) -> Option<KeyEvent> {
        let mut start: u16 = 1;
        let mut end: u16;
        for action in &self.actions {
            end = start + action.text.len() as u16;
            if column >= start && column < end {
                return Some(action.key_event);
            }
            start = end + Self::SEPARATOR.len() as u16;
        }
        None
    }
}
impl Display for TaskBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tasks: Vec<String> = self.actions.iter().map(|t| t.text.to_string()).collect();
        write!(f, "{}", tasks.join(Self::SEPARATOR))
    }
}
