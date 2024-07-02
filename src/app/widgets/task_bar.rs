/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-16
 */
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

use crate::app;
use app::widgets::key_button::KeyButton;
use app::widgets::task_action::TaskAction;

pub struct TaskBar {
    buttons: Vec<KeyButton>,
    style: Style,
    button_style: Style,
    button_space: u16,
}

impl Default for TaskBar {
    fn default() -> Self {
        Self {
            buttons: vec![],
            style: Style::default(),
            button_style: Style::default(),
            button_space: 1,
        }
    }
}
#[allow(dead_code)]
impl TaskBar {
    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }
    pub fn button_style(&mut self, style: Style) -> &mut Self {
        self.button_style = style;
        self
    }
    pub fn button_space(&mut self, space: u16) -> &mut Self {
        self.button_space = space;
        self
    }
    pub fn buttons(&mut self, actions: Vec<TaskAction>) -> &mut Self {
        let mut position = self.button_space;
        for action in actions {
            let key_button = KeyButton::new(action, self.button_style, position);
            position += self.button_space + key_button.len() as u16;
            self.buttons.push(key_button);
        }
        self
    }
    pub fn hit_test(&self, column: u16) -> Option<KeyEvent> {
        let position: u16 = column + self.button_space;
        for button in &self.buttons {
            if button.hit_test(position) {
                return Some(*button.key_event());
            }
        }
        None
    }
}
impl WidgetRef for TaskBar {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let mut area = area;
        let mut offset = self.button_space;
        for button in &self.buttons {
            area.x = offset;
            button.render(area, buf);
            offset += (button.len() as u16) + self.button_space;
        }
    }
}
