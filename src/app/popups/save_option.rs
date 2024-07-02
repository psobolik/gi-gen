/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-14
 */

use ratatui::layout::{Constraint, Direction, Layout, Margin, Position, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::styles;
use crate::util;

#[derive(Default)]
struct FrameSet {
    popup: Rect,
    prompt: Rect,
    replace: Rect,
    append: Rect,
    cancel: Rect,
}

pub(crate) enum SaveOptions {
    Replace,
    Append,
    Cancel,
}

#[derive(Default)]
pub(crate) struct SaveOption {
    frame_set: FrameSet,
}

impl SaveOption {
    const REPLACE_TEXT: &'static str = "Replace";
    const APPEND_TEXT: &'static str = "Append";
    const CANCEL_TEXT: &'static str = "Cancel";
    const REPLACE_CHAR: char = 'r';
    const APPEND_CHAR: char = 'a';
    const CANCEL_CHAR: char = 'c';

    pub(crate) fn key_test(&self, ch: char) -> Option<SaveOptions> {
        match ch {
            Self::REPLACE_CHAR => Some(SaveOptions::Replace),
            Self::APPEND_CHAR => Some(SaveOptions::Append),
            Self::CANCEL_CHAR => Some(SaveOptions::Cancel),
            _ => None,
        }
    }
    pub(crate) fn hit_test(&self, column: u16, row: u16) -> Option<SaveOptions> {
        let position = Position::new(column, row);
        if self.frame_set.replace.contains(position) {
            Some(SaveOptions::Replace)
        } else if self.frame_set.append.contains(position) {
            Some(SaveOptions::Append)
        } else if self.frame_set.cancel.contains(position)
            || !self.frame_set.popup.contains(position)
        {
            Some(SaveOptions::Cancel)
        } else {
            None
        }
    }
    pub(in crate::app) fn render(&mut self, frame: &mut Frame<'_>) {
        let block = Block::new()
            .borders(Borders::ALL)
            .style(styles::POPUP_BLOCK_STYLE);
        let prompt = Paragraph::new("A .gitignore file already exists in the current directory.")
            .style(styles::POPUP_MESSAGE_STYLE)
            .wrap(Wrap { trim: true });
        let replace_prompt = Self::format_prompt(Self::REPLACE_TEXT);
        let append_prompt = Self::format_prompt(Self::APPEND_TEXT);
        let cancel_prompt = Self::format_prompt(Self::CANCEL_TEXT);

        self.frame_set = Self::calculate_frames(
            frame.size(),
            Self::REPLACE_TEXT.len(),
            Self::APPEND_TEXT.len(),
            Self::CANCEL_TEXT.len(),
        );

        frame.render_widget(Clear, self.frame_set.popup); // This clears the background underneath the popup
        frame.render_widget(block, self.frame_set.popup);
        frame.render_widget(prompt, self.frame_set.prompt);
        frame.render_widget(replace_prompt, self.frame_set.replace);
        frame.render_widget(append_prompt, self.frame_set.append);
        frame.render_widget(cancel_prompt, self.frame_set.cancel);
    }
    fn format_prompt(prompt: &str) -> Paragraph {
        let prompt = prompt.as_bytes();
        let first = std::str::from_utf8(&prompt[0..1]).expect("Bad input");
        let rest = std::str::from_utf8(&prompt[1..]).expect("Bad input");

        Paragraph::new(vec![Line::from(vec![
            Span::raw("["),
            Span::styled(first, Style::new().reversed()),
            Span::raw(rest),
            Span::raw("]"),
        ])])
        .style(styles::POPUP_MESSAGE_STYLE)
    }
    fn calculate_frames(
        rect: Rect,
        replace_width: usize,
        append_width: usize,
        cancel_width: usize,
    ) -> FrameSet {
        let vertical_layout = Layout::default()
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
            ])
            .split(rect);
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(0),
                Constraint::Length(30),
                Constraint::Fill(0),
            ])
            .split(vertical_layout[1]);

        let popup = horizontal_layout[1];
        let inner_layout = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(popup);
        let prompt = inner_layout[0].inner(&Margin::new(2, 1));
        let button_layout = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner_layout[1]);
        let replace = Self::calculate_button_rect(button_layout[0], replace_width);
        let append = Self::calculate_button_rect(button_layout[1], append_width);
        let cancel = Self::calculate_button_rect(button_layout[2], cancel_width);

        let popup = util::centered_rect(popup.width, popup.height, rect);
        FrameSet {
            popup,
            prompt,
            replace,
            append,
            cancel,
        }
    }
    fn calculate_button_rect(rect: Rect, width: usize) -> Rect {
        util::centered_rect((width + 2) as u16, rect.height, rect)
    }
}
