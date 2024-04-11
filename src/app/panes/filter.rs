/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-12
 */

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Default)]
pub(crate) struct Filter {
    text: String,
}

impl Filter {
    pub(crate) fn text(&self) -> &str {
        self.text.as_str()
    }
    pub(crate) fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    pub(crate) fn clear(&mut self) {
        self.set_text(String::default().as_str());
    }
    pub(crate) fn append(&mut self, ch: char) -> &str {
        let text = self.text.to_owned() + &ch.to_string();
        self.text = text;
        self.text()
    }
    pub(crate) fn delete_last(&mut self) -> &str {
        let len = self.text.len();
        if len > 0 {
            self.text.remove(len - 1);
        }
        self.text()
    }
    pub(crate) fn render(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let text = format!(" Filter: [{}]", self.text);
        let paragraph = Paragraph::new(text.as_str());
        frame.render_widget(paragraph, area);
    }
}
