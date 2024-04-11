/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-12
 */

use ratatui::prelude::*;

pub fn is_in_rect(x: u16, y: u16, rect: ratatui::layout::Rect) -> bool {
    x >= rect.left() && x < rect.right() && y >= rect.top() && y < rect.bottom()
}
pub fn centered_rect(width: u16, height: u16, rect: Rect) -> Rect {
    let vert_margin = (rect.height - height) / 2;
    let horiz_margin = (rect.width - width) / 2;
    let vert_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vert_margin),
            Constraint::Length(height),
            Constraint::Length(vert_margin),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horiz_margin),
            Constraint::Length(width),
            Constraint::Length(horiz_margin),
        ])
        .split(vert_layout[1])[1]
}
