/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-15
 */
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::styles;

pub(crate) fn render(frame: &mut Frame) {
    let text = vec![
        Line::from("Use this app to create a .gitignore file for one or more operating systems, programming languages or IDEs, using templates from https://www.toptal.com/developers/gitignore/"),
        Line::default(),
        Line::from("* Select the templates to include in the file."),
        Line::from("  - Use the up and down arrows to highlight a template."),
        Line::from("  - Press the space bar to select the highlighted template."),
        Line::from("  - Type all or part of a template's name to filter the list."),
        Line::default(),
        Line::from("* Press Ctrl+S to write the .gitignore file to disk."),
        Line::from("  - The .gitignore file will be written to the current directory."),
        Line::from("  - If the .gitignore file already exists, you will be given the option of replacing it or appending to it."),
        Line::default(),
        Line::from("* Press Ctrl+Q to close the app without writing the .gitignore file."),
    ];
    let block = Block::new()
        .borders(Borders::ALL)
        .style(styles::POPUP_BLOCK_STYLE);
    let content = Paragraph::new(text)
        .style(styles::POPUP_MESSAGE_STYLE)
        .wrap(Wrap { trim: false });

    let popup_area = calculate_area(frame.size());
    let content_area = popup_area.inner(&Margin::new(2, 1));

    frame.render_widget(Clear, popup_area); // This clears the background underneath the popup
    frame.render_widget(block, popup_area);
    frame.render_widget(content, content_area);
}
fn calculate_area(rect: Rect) -> Rect {
    let vertical_layout = Layout::default()
        .constraints([
            Constraint::Fill(0),
            Constraint::Length(25),
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
