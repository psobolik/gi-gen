/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-12
 */

use ratatui::widgets::ListState;

#[derive(Default, Debug)]
pub struct ListStateWrapper {
    pub list_state: ListState,
    pub lower_bound: usize,
    pub upper_bound: usize,
    pub wrap: bool,
    pub size: usize,
}

#[allow(dead_code)]
impl ListStateWrapper {
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn set_size(&mut self, length: usize) -> &mut Self {
        self.size = length;
        self.upper_bound = if length > 0 { length - 1 } else { 0 };
        self
    }
    pub fn wrap(&self) -> bool {
        self.wrap
    }
    pub fn set_wrap(&mut self, wrap: bool) -> &mut Self {
        self.wrap = wrap;
        self
    }
    pub fn selected(&self) -> Option<usize> {
        if self.size > 0 {
            self.list_state.selected()
        } else {
            None
        }
    }
    pub fn is_in_bounds(&self, index: isize) -> bool {
        self.size > 0 && index >= self.lower_bound as isize && index <= self.upper_bound as isize
    }
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }
    pub fn is_first_selected(&self) -> bool {
        match self.list_state.selected() {
            Some(selected) => selected == self.lower_bound,
            None => false,
        }
    }
    pub fn select_first(&mut self) {
        if !self.is_first_selected() {
            self.set_selected(Some(self.lower_bound));
        }
    }
    pub fn is_last_selected(&self) -> bool {
        match self.list_state.selected() {
            Some(selected) => selected == self.upper_bound,
            None => false,
        }
    }
    pub fn select_last(&mut self) {
        if !self.is_last_selected() {
            self.set_selected(Some(self.upper_bound));
        }
    }
    pub fn advance_selected(&mut self, distance: usize) {
        let new_selected = self.selected().unwrap_or(self.upper_bound) + distance;
        if new_selected < self.upper_bound {
            self.set_selected(Some(new_selected));
        } else if self.wrap && self.is_last_selected() {
            self.select_first();
        } else {
            self.select_last()
        }
    }
    pub fn recede_selected(&mut self, distance: usize) {
        let selected = self.selected().unwrap_or(self.lower_bound);
        if selected < distance {
            if self.wrap && self.is_first_selected() {
                self.select_last();
            } else {
                self.select_first();
            }
        } else {
            self.set_selected(Some(selected - distance));
        }
    }
    pub fn offset(&self) -> usize {
        self.list_state.offset()
    }
    pub fn set_offset(&mut self, offset: usize) {
        *self.list_state.offset_mut() = offset;
    }
    pub fn at_offset_first(&self) -> bool {
        self.list_state.offset() == self.lower_bound
    }
    pub fn set_offset_first(&mut self) {
        self.set_offset(self.lower_bound)
    }
    pub fn at_offset_last(&mut self) -> bool {
        self.offset() == self.upper_bound
    }
    pub fn set_offset_last(&mut self) {
        self.set_offset(self.upper_bound)
    }
    pub fn advance_offset(&mut self, distance: usize) {
        let offset = self.offset() + distance;
        if offset < self.upper_bound {
            self.set_offset(offset)
        } else {
            self.set_offset_last()
        }
    }
    pub fn recede_offset(&mut self, distance: usize) {
        let offset = self.offset();
        if offset < distance {
            self.set_offset_first()
        } else {
            self.set_offset(offset - distance);
        }
    }
}
