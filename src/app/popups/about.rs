/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-15
 */

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span, Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::styles;

pub(crate) fn render(frame: &mut Frame) {
    let text = vec![
        Line::from("+-------------+".light_blue()),
        Line::from(vec![
            Span::styled("| ", Style::new().light_blue()),
            Span::raw("g i - g e n"),
            Span::styled(" |", Style::new().light_blue()),
        ]),
        Line::from("+-------------+".light_blue()),
        Line::from(vec![Span::raw("v"), Span::raw(env!("CARGO_PKG_VERSION"))]),
        Line::from("Copyright © 2024 Paul Sobolik"),
        Line::default(),
        Line::from("API and templates provided by".italic()),
        Line::from("https://www.toptal.com/developers/gitignore/".italic()),
    ];
    let block = Block::new()
        .borders(Borders::ALL)
        .style(styles::POPUP_BLOCK_STYLE);
    let content = Paragraph::new(text)
        .style(styles::POPUP_MESSAGE_STYLE)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    let area = calculate_area(frame.size());

    frame.render_widget(Clear, area); // This clears the background underneath the popup
    frame.render_widget(content, area);
}
fn calculate_area(rect: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .constraints([
            Constraint::Fill(0),
            Constraint::Length(10),
            Constraint::Fill(0),
        ])
        .split(rect);
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(50),
            Constraint::Fill(1),
        ])
        .split(vertical_layout[1]);

    horizontal_layout[1]
}
