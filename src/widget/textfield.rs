use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    Id,
    ButtonFlags,
    CursorType,
    make_color,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
};

impl Context {
    fn draw_textfield_text(&mut self, bounds: Rect, text: Option<&str>, offset_x: f32, text_style: TextStyle) {
        let yoff = bounds.height() * 0.5f32 - self.default_font.height() * 0.5f32;
        let pos = float2(bounds.min.0, bounds.max.1 - yoff).round();
        if let Some(text) = text {
            self.draw_list.add_text_clipped(
                &mut *self.renderer,
                &mut self.default_font,
                text,
                pos,
                text_style.color,
                offset_x,
                bounds.max.0
            );
        } else {
            self.draw_list.add_chars_clipped(
                &mut *self.renderer,
                &mut self.default_font,
                &self.text_edit_state.buffer,
                pos,
                text_style.color,
                offset_x,
                bounds.max.0
            );
        }
    }

    fn draw_textfield_bg(&mut self, bounds: Rect, border: Border, bg: Background) {
        let bmin = bounds.min.round();
        let bmax = bounds.max.round();
        
        match bg {
            Background::Color(c) => {
                self.draw_list.add_rect_filled(
                    bmin, bmax,
                    0f32,
                    c
                );
            }
            _ => {}
        }

        let off = float2(0.5f32, 0.5f32);
        self.draw_list.add_rect_gradient(
            bmin - off, bmax + off,
            0f32, border.thickness,
            border.color & 0x00ffffff, border.color
        );
    }

    fn locate_textfield_char(&mut self, x: f32) -> usize {
        let mut prev_x = 0f32;
        let text = &self.text_edit_state.buffer;
        
        for (idx, &ch) in text.iter().enumerate() {
            let w = self.default_font.char_width(None, ch);

            if x < prev_x + w {
                if x < prev_x + (w * 0.5f32) {
                    return idx;
                } else {
                    return idx + 1;
                }
            }
            
            prev_x += w;
        }

        text.len() - 1
    }
    
    pub fn textfield(&mut self, id: &str, text: &mut String) {
        let id = self.id(id);
        let pad = self.style.textfield.padding.1;

        self.last_widget_state = WidgetState::None;
        let (bounds, state) = self.widget(Some(self.default_font.height() + pad));

        let mut clear_active_id = false;

        let io = &self.io;
        let hovering = io.has_mouse_in_rect(bounds);
        let pressed = hovering && io.is_mouse_pressed(MouseButton::Left);
        let down = io.is_mouse_down(MouseButton::Left);
        let dragging = down && io.has_mouse_click_in_rect(MouseButton::Left, bounds);
        
        // let pressed = self.button_behaviour(bounds, ButtonFlags::PressOnClick);

        if hovering || dragging {
            self.cursor = CursorType::Caret;
        } 
        if pressed {
            if self.active_id != id {
                self.text_edit_state.buffer.clear();
                self.text_edit_state.buffer.extend(text.chars());
                self.text_edit_state.backup = text.clone();
                
                self.text_edit_state.cursor = 0;
                self.text_edit_state.clear_selection();
                self.text_edit_state.reset_cursor();

                self.text_edit_state.id = id;
                self.text_edit_state.scroll_x = 0f32;
            }
            self.set_active_id(id);
        } else if io.mouse_pressed[MouseButton::Left as usize] {
            clear_active_id = true;
        }
        
        let style = &self.style.textfield;
        let state = self.last_widget_state;

        let (text_style, border, background) = if self.active_id == id {
            (style.active_text, style.active_border, style.active)
        } else if hovering {
            (style.hover_text, style.hover_border, style.hover)
        } else {
            (style.normal_text, style.normal_border, style.normal)
        };

        if clear_active_id {
            self.set_active_id(0);
        }

        let (text, offset) = if self.active_id == id {
            (None, self.text_edit_state.scroll_x)
        } else {
            (Some(text.as_str()), 0f32)
        };

        let padding = self.style.textfield.padding.0;
        let clip_bounds = bounds.grow(
            float2(padding, 0f32),
            float2(padding, 0f32)
        );

        let io = &self.io;
        let sx = self.text_edit_state.scroll_x;
        let mx = io.mouse.0 - clip_bounds.min.0 + sx; 
        let dm = io.mouse_delta;
        if pressed {
            let cur = self.locate_textfield_char(mx);
            
            self.text_edit_state.click(cur);
        } else if dragging && (dm.0 != 0f32 || dm.1 != 0f32) {
            let cur = self.locate_textfield_char(mx);

            self.text_edit_state.drag(cur);
        }
        
        self.draw_textfield_bg(bounds, border, background);
        self.draw_textfield_text(clip_bounds, text, offset, text_style);

        if self.active_id == id {
            let cur = self.text_edit_state.cursor;
            let sel_start = self.text_edit_state.selection_start;
            
            let mut cursor_x = self.default_font.chars_width(&self.text_edit_state.buffer[..cur]);

            self.text_edit_state.animate(self.io.delta);

            let yoff = bounds.height() * 0.5f32 - self.default_font.height() * 0.5f32;
            let screen_pos = clip_bounds.min - float2(offset, 0f32);
            let cursor_screen_pos = screen_pos + float2(cursor_x, yoff);
            let cursor_rect = Rect::new(
                cursor_screen_pos,
                cursor_screen_pos + float2(1f32, self.default_font.height())
            );

            // follow cursor
            let sx = self.text_edit_state.scroll_x;
            let w = clip_bounds.width();
            let increment = w * 0.25f32;
            if cursor_x < sx {
                self.text_edit_state.scroll_x = (cursor_x - increment).floor().max(0f32);
            } else if cursor_x - w >= sx {
                self.text_edit_state.scroll_x = (cursor_x - w + increment).floor();
            }
            
            if self.text_edit_state.has_selection() {
                use std::mem::swap;

                let mut selection_x = self.default_font.chars_width(&self.text_edit_state.buffer[..sel_start]);
                if cursor_x > selection_x {
                    swap(&mut cursor_x, &mut selection_x);
                }
                let dist = selection_x - cursor_x;
                
                let cursor_screen_pos = screen_pos + float2(cursor_x, yoff);
                let selection_rect = Rect::new(
                    cursor_screen_pos,
                    cursor_screen_pos + float2(dist, self.default_font.height())
                );
                
                self.draw_list.add_rect_filled(selection_rect.min, selection_rect.max, 0f32, make_color(127, 201, 255, 124));
            }
            
            if self.text_edit_state.is_cursor_visible() {
                self.draw_list.add_rect_filled(cursor_rect.min, cursor_rect.max, 0f32, 0xffffffff);
            }


        }
    }
}

#[derive(Debug)]
pub enum TextEditAction {
    Char(char),
    Key(TextEditKey),
}

#[derive(Debug)]
pub enum TextEditKey {
    Left(bool),
    Right(bool),
    Delete,
    Backspace,
}

pub struct EditState {
    pub id: Id,
    buffer: Vec<char>,
    
    backup: String,

    scroll_x: f32,
    animation: f32,

    cursor: usize,
    selection_start: usize,
    selection_end: usize,
}

impl EditState {
    pub fn new() -> Self {
        EditState {
            id: 0,
            buffer: Vec::new(),
            backup: String::new(),
            
            scroll_x: 0f32,
            animation: 0f32,

            cursor: 0,
            selection_start: 0,
            selection_end: 0,
        }
    }

    pub fn click(&mut self, cur: usize) {
        self.cursor = cur;
        self.selection_start = self.cursor;
        self.selection_end = self.cursor;

        self.reset_cursor();
    }

    pub fn drag(&mut self, cur: usize) {
        if self.selection_start == self.selection_end {
            self.selection_start = self.cursor;
        }

        self.cursor = cur;
        self.selection_end = self.cursor;

        self.reset_cursor();
    }
    
    pub fn reset_cursor(&mut self) {
        self.animation = -0.5f32;
    }

    pub fn animate(&mut self, dt: f32) {
        self.animation += dt;
        if self.animation > 0.5f32 {
            self.reset_cursor();
        }
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.animation < 0f32
    }
    
    pub fn sort_selection(&mut self) {
        use std::mem::swap;
        
        if self.selection_end < self.selection_start {
            swap(&mut self.selection_end, &mut self.selection_start);
        }
    }

    pub fn prepare_selection_at_cursor(&mut self) {
        if !self.has_selection() {
            self.selection_start = self.cursor;
            self.selection_end = self.cursor;
        } else {
            self.cursor = self.selection_end;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = self.cursor;
        self.selection_end = self.cursor;
    }

    pub fn move_to_first(&mut self) {
        if self.has_selection() {
            self.sort_selection();
            
            self.cursor = self.selection_start;
            self.selection_end = self.selection_start;
        }
    }

    pub fn move_to_end(&mut self) {
        if self.has_selection() {
            self.sort_selection();
            self.clamp();
        
            self.cursor = self.selection_end;
            self.selection_start = self.selection_end;
        }
    }

    pub fn select_all(&mut self) {
        self.selection_start = 0;
        self.selection_end = self.buffer.len();
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start != self.selection_end
    }

    pub fn clamp(&mut self) {
        let len = self.buffer.len();
        if self.has_selection() {
            if self.selection_start > len {
                self.selection_start = len;
            }
            if self.selection_end > len {
                self.selection_end = len;
            }
            
            if self.selection_start == self.selection_end {
                self.cursor = self.selection_start
            }
        }
        
        if self.cursor > len {
            self.cursor = len;
        }
    }

    pub fn delete_selection(&mut self) {
        self.clamp();

        if self.selection_start != self.selection_end {
            if self.selection_start < self.selection_end {
                self.buffer.drain(self.selection_start..self.selection_end);
                
                self.selection_end = self.selection_start;
                self.cursor = self.selection_start;
            } else {
                self.buffer.drain(self.selection_end..self.selection_start);
                
                self.selection_start = self.selection_end;
                self.cursor = self.selection_end;
            }
        }
    }

    pub fn delete_chars(&mut self, at: usize, len: usize) {
        self.buffer.drain(at..(at + len));
    }
    
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();
        
        self.buffer.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn key_press(&mut self, key: TextEditKey) {
        match key {
            TextEditKey::Left(shift) => {
                if shift {
                    self.clamp();
                    self.prepare_selection_at_cursor();
                    
                    if self.selection_end > 0 {
                        self.selection_end -= 1;
                    }
                    self.cursor = self.selection_end;
                } else {
                    if self.has_selection() {
                        self.move_to_first();
                    } else {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                }
            }
            TextEditKey::Right(shift) => {
                if shift {
                    if !self.has_selection() {
                        self.prepare_selection_at_cursor();
                    }
                    
                    self.selection_end += 1;
                    self.clamp();
                    self.cursor = self.selection_end;
                } else {
                    if self.has_selection() {
                        self.move_to_end();
                    } else {
                        self.cursor += 1;
                    }
                    
                    self.clamp();
                }
            }
            TextEditKey::Delete => {
                if self.has_selection() {
                    self.delete_selection();
                } else {
                    let n = self.buffer.len();
                    if self.cursor < n {
                        self.delete_chars(self.cursor, 1);
                    }
                }
            }
            TextEditKey::Backspace => {
                if self.has_selection() {
                    self.delete_selection();
                } else {
                    self.clamp();
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.delete_chars(self.cursor, 1);
                    }
                }
            }
        }

        self.reset_cursor();
    }
}
