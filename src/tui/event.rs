/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-11
 */

use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Clone, Debug)]
pub enum Event {
    Init,
    TemplateSelect(String),
    // Quit,
    Error,
    // Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}
