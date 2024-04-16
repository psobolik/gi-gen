/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-16
 */

use crossterm::event::KeyEvent;

#[derive(Clone)]
pub struct TaskAction {
    key_event: KeyEvent,
    text: String,
}

impl TaskAction {
    pub fn new(key_event: KeyEvent, text: &str) -> Self {
        Self {
            key_event,
            text: text.to_string(),
        }
    }
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
    pub fn key_event(&self) -> &KeyEvent {
        &self.key_event
    }
}
