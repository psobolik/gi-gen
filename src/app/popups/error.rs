/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-15
 */

use crate::app::styles;
use crate::util;
use ratatui::layout::Alignment;
use ratatui::prelude::Text;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub(crate) fn render(message: &str, frame: &mut Frame) {
    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(styles::POPUP_ERROR_BLOCK_STYLE);
    let content = Paragraph::new(Text::from(message))
        .style(styles::POPUP_ERROR_MESSAGE_STYLE)
        .alignment(Alignment::Center)
        .block(block);

    let error_len = message.len() as u16;
    let area = util::centered_rect(error_len + 6, 3, frame.size());

    frame.render_widget(Clear, area); // This clears the background underneath the popup
    frame.render_widget(content, area);
}
