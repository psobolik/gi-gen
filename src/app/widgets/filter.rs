/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-16
 */

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Default)]
pub struct Filter {
    text: String,
    style: Style,
}

#[allow(dead_code)]
impl Filter {
    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    pub fn clear(&mut self) {
        self.set_text(String::default().as_str());
    }
    pub fn push(&mut self, ch: char) -> &str {
        let text = self.text.to_owned() + &ch.to_string();
        self.text = text;
        self.text()
    }
    pub fn pop(&mut self) -> &str {
        let len = self.text.len();
        if len > 0 {
            self.text.remove(len - 1);
        }
        self.text()
    }
}
impl WidgetRef for Filter {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let text = format!(" Filter: [{}]", self.text);
        buf.set_string(area.x, area.y, text, self.style);
    }
}
