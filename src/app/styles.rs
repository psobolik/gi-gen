/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-13
 */

use ratatui::prelude::{Color, Modifier, Style};

pub(super) const LIST_HIGHLIGHT_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Gray);
pub(super) const FOCUSED_BLOCK_STYLE: Style = Style::new()
    .fg(Color::LightBlue)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD);
pub(super) const FOCUSED_TITLE_STYLE: Style = Style::new().fg(Color::Gray).bg(Color::Blue);
pub(super) const DEFAULT_BLOCK_STYLE: Style = Style::new().fg(Color::DarkGray).bg(Color::Black);
pub(super) const DEFAULT_TITLE_STYLE: Style = Style::new().fg(Color::Gray).bg(Color::Black);
pub(super) const TASK_BAR_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);
pub(crate) const POPUP_ERROR_MESSAGE_STYLE: Style = Style::new().fg(Color::White).bg(Color::Black);
pub(crate) const POPUP_ERROR_BLOCK_STYLE: Style = Style::new().fg(Color::LightRed).bg(Color::Black);
pub(crate) const POPUP_MESSAGE_STYLE: Style = Style::new().fg(Color::White).bg(Color::Black);
pub(crate) const POPUP_BLOCK_STYLE: Style = Style::new().fg(Color::LightYellow).bg(Color::Black);
