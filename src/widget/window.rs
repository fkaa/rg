use crate::{
    Context, MouseButton, Id, hash_id,
    WidgetState,
    Background,
    CursorType,
    PanelFlags,
    
    TextAlignment,
    math::{
        float2,
        Rect,
    },

    layout,
};

use super::{
    Panel, PanelType,
};

struct WindowPerFrameData {
    cursor_start: float2,
    cursor: float2,
    cursor_prev_line: float2,
    cursor_max_pos: float2,
    indent: f32,
    current_text_base_offset: f32,
}

impl WindowPerFrameData {
    pub fn new() -> Self {
        WindowPerFrameData {
            cursor_start: float2(0f32, 0f32),
            cursor: float2(0f32, 0f32),
            cursor_prev_line: float2(0f32, 0f32),
            cursor_max_pos: float2(0f32, 0f32),
            indent: 0f32,
            current_text_base_offset: 0f32,
        }
    }
}

bitflags! {
    pub struct WindowFlags: u32 {
        const None = 0;
        
        const Border = 1 << 0;
        const Movable = 1 << 1;
        const Scalable = 1 << 2;
        const Closable = 1 << 3;
        const Minimizable = 1 << 4;
        const NoScrollbar = 1 << 5;
        const Title = 1 << 6;
        const ScrollAutoHide = 1 << 7;
        const Background = 1 << 8;
        const ScaleLeft = 1 << 9;
        const NoInput = 1 << 10;
        
        const Private = 1 << 11;
        const Dynamic = 1 << 12;
        const ReadOnly = 1 << 13;
        const NotInteractive = Self::ReadOnly.bits | Self::NoInput.bits;
        const Hidden = 1 << 14;
        const Closed = 1 << 15;
        const Minimized = 1 << 16;
        const RemoveReadOnly = 1 << 17;
        const NoBg = 1 << 18;
    }
}

pub struct Window {
    pub name: String,
    pub id: Id,
    pub bounds: Rect,
    pub flags: WindowFlags,
    pub scrollbar: float2,
    data: WindowPerFrameData,
}

impl Window {
    pub fn new(label: String, hash: Id, bounds: Rect, flags: WindowFlags) -> Self {
        Window {
            name: label,
            id: hash,

            bounds,
            flags,

            scrollbar: float2(0f32, 0f32),
            data: WindowPerFrameData::new(),
        }
    }

    pub fn has_header(&self) -> bool {
        (self.flags.contains(WindowFlags::Closable) || self.flags.contains(WindowFlags::Minimizable)
         || self.flags.contains(WindowFlags::Title))
            && !self.flags.contains(WindowFlags::Hidden)
    }

    pub fn is_clipped(&self, bounding_box: Rect) -> bool {
        true // self.clip_rect.outside(bounding_box)
    }
}

impl Context {
    fn get_window_stack_pos(&self, idx: usize) -> Option<usize> {
        let len = self.window_stack.len();
        let mut i = 0;
        
        while i < len {
            let wnd = unsafe { *self.window_stack.get_unchecked(i) };
            if idx == wnd {
                return Some(i);
            }
            
            i += 1;
        }

        None
    }

    fn move_stack_pos_to_front(&mut self, idx: usize) {
        let wnd = self.window_stack.remove(idx);

        self.window_stack.push(wnd);
    }
    
    fn move_window_to_front(&mut self, idx: usize) {
        if let Some(wnd) = self.get_window_stack_pos(idx) {
            self.move_stack_pos_to_front(wnd);
        }
    }
    
    fn create_window(&mut self, name: &str, hash: Id, bounds: Rect, flags: WindowFlags) -> usize {
        let mut wnd = Window::new(name.into(), hash, bounds, flags);

        let idx = self.windows.len();

        self.windows.push(wnd);

        idx
    }

    fn find_window_hash(&self, hash: Id) -> Option<usize> {
        let len = self.windows.len();
        let mut i = 0;
        
        while i < len {
            let wnd = unsafe { self.windows.get_unchecked(i) };
            if wnd.id == hash {
                return Some(i);
            }

            i += 1;
        }

        None
    }
    
    fn find_window(&self, name: &str) -> Option<usize> {
        let id = hash_id(name);

        self.find_window_hash(id)
    }

    pub fn begin(&mut self, title: &str, flags: WindowFlags) -> bool {
        let offset = float2(40f32, 40f32) * self.window_stack.len() as f32;
        self.begin_titled(title, Rect::new(float2(10f32, 10f32) + offset, float2(410f32, 510f32) + offset), flags)
    }

    pub fn begin_root(&mut self, title: &str, flags: WindowFlags) -> bool {
        self.begin_titled(
            title,
            Rect::new(float2(0f32, 0f32), self.io.display_size),
            WindowFlags::Background | flags
        )
    }
    
    pub fn begin_titled(&mut self, title: &str, bounds: Rect, flags: WindowFlags) -> bool {
        let hash = hash_id(title);
        
        let idx = if let Some(idx) = self.find_window_hash(hash) {
            let wnd = &mut self.windows[idx];
            wnd.flags |= flags;
            if !(wnd.flags.contains(WindowFlags::Movable) || wnd.flags.contains(WindowFlags::Scalable)) {
                wnd.bounds = bounds;
            }

            idx
        } else {
            let idx = self.create_window(title, hash, bounds, flags | WindowFlags::ReadOnly);

            self.window_stack.push(idx);
            
            idx
        };

        let stack_pos = self.get_window_stack_pos(idx).unwrap();
        let wnd = &mut self.windows[idx];
        let io = &self.io;

        if wnd.flags.contains(WindowFlags::Background) {
            wnd.bounds = Rect::new(float2(0f32, 0f32), self.io.display_size);
        }

        if wnd.flags.contains(WindowFlags::Hidden) {
            return false;
        }

        if !wnd.flags.contains(WindowFlags::NoInput) {
            let bounds = wnd.bounds;

            let bg = wnd.flags.contains(WindowFlags::Background);
            let mouse_click = io.is_mouse_pressed(MouseButton::Left);
            let mouse_down = io.is_mouse_down(MouseButton::Left);
            let mouse_inside = mouse_click && io.has_mouse_click_in_rect(MouseButton::Left, wnd.bounds);
            let mouse_hover = io.has_mouse_in_rect(wnd.bounds);

            // borrowck {
            drop(wnd);
            
            let mut window_clicked = None;
            if mouse_inside {
                let len = self.window_stack.len();
                let mut i = 0;
                let mut p = 0;

                while i < len {
                    // if i == idx { continue; }
                    let pos = self.window_stack[i];
                    let wnd = &self.windows[i];
                    let bounds = wnd.bounds;

                    if !wnd.flags.contains(WindowFlags::Background) {
                        if bounds.contains(io.mouse) && pos >= p {
                            window_clicked = Some(i);
                            p = pos;
                        }
                    }

                    i += 1;
                }
            }

            let wnd = &mut self.windows[idx];
            // borrowck }

            if let Some(wnd_idx) = window_clicked {
                wnd.flags.insert(WindowFlags::ReadOnly);
                self.windows[wnd_idx].flags.remove(WindowFlags::ReadOnly);
                //self.active = Some(wnd_idx);
                
                self.move_window_to_front(wnd_idx);
            } else if mouse_inside {
                wnd.flags.remove(WindowFlags::ReadOnly);
                //self.active = Some(idx);

                if !bg {
                    self.move_stack_pos_to_front(stack_pos);
                }
            } else if mouse_click {
                wnd.flags.insert(WindowFlags::ReadOnly);
            }
        }
        self.active = Some(idx);
        let wnd = &mut self.windows[idx];
        /*wnd.layout.offset = wnd.scrollbar;
        let panel_padding = wnd.layout.get_padding(&self.style.window);
        wnd.layout.bounds = wnd.bounds.grow(
            float2(panel_padding.0, panel_padding.1),
            float2(panel_padding.0, panel_padding.1)
        );
        wnd.layout.cursor = wnd.layout.bounds.min;*/
        
        // draw
        let state = WidgetState::None;

        let header = &self.style.window.header;
        let window = &self.style.window;
        let (border, background, header_background, text) = if state.contains(WidgetState::Active) {
            (window.active_border, window.active, header.active, header.active_text)
        } else if state.contains(WidgetState::Hovering) {
            (window.hover_border, window.hover, header.hover, header.hover_text)

        } else {
            (window.normal_border, window.normal, header.normal, header.normal_text)
        };

        let mut body = wnd.bounds;
        
        if wnd.has_header() {
            let style = &self.style.window.header;
            let mut header = wnd.bounds;
            header.max.1 = header.min.1;
            header.max.1 += 14f32 + 2f32 * style.padding.1;
            header.max.1 += 2f32 * style.label_padding.1;

            let h = header.height();
            body.min.1 += h;

            match header_background {
                Background::Color(col) => {
                    self.draw_list.add_rect_filled(header.min, header.max, 0f32, col);
                },
                _ => {}
            }

            if wnd.flags.contains(WindowFlags::Title) {
                self.draw_list.add_text_wrapped(
                    &mut *self.renderer,
                    &mut self.default_font,
                    title,
                    header.min,
                    text.align,
                    header.width(),
                    text.color
                );
            }
        }

        if !wnd.flags.contains(WindowFlags::NoBg) {
            match background {
                Background::Color(col) => {
                    self.draw_list.add_rect_filled(body.min, body.max, 0f32, col);
                },
                _ => {}
            }

            if border.thickness != 0f32 {
                let border_bounds = wnd.bounds.pad(-border.thickness * 0.5f32);
                self.draw_list.add_rect(border_bounds.min, border_bounds.max, border.rounding, border.thickness, border.color);
            }
        }

        self.begin_panel_ex(title, Some(body), None, PanelFlags::None)
    }

    pub fn end(&mut self) {
        self.end_panel();

        if let Some(idx) = self.active {
            self.draw_list.push_layer(idx as u32);

            let io = &mut self.io;
            let wnd = &mut self.windows[idx];
            // drag, resize handling
            if wnd.flags.contains(WindowFlags::Movable) && !wnd.flags.contains(WindowFlags::ReadOnly) {
                let border_size = 3f32;
                let handle_size = 5f32;
                let handle_area = float2(handle_size, handle_size);

                let hori_resize_area = float2(-border_size, handle_size);
                let vertical_resize_area = float2(handle_size, -border_size);

                let handle_resize_area = float2(-handle_size, -handle_size);

                let mut header = wnd.bounds;
                header.max.1 = header.min.1;

                let style = &self.style.window.header;
                if wnd.has_header() {
                    header.max.1 += 14f32 + 2f32 * style.padding.1;
                    header.max.1 += 2f32 * style.label_padding.1;
                } else {
                    let pane = &mut self.panel_stack[self.current_panel];
                    header.max.1 += pane.get_padding(&self.style.window).1;
                }
                
                let left = Rect::new(
                    wnd.bounds.min,
                    float2(wnd.bounds.min.0, wnd.bounds.max.1)
                ).grow(hori_resize_area, hori_resize_area);
                let right = Rect::new(
                    float2(wnd.bounds.max.0, wnd.bounds.min.1),
                    wnd.bounds.max,
                ).grow(hori_resize_area, hori_resize_area);

                let top = Rect::new(
                    wnd.bounds.min,
                    float2(wnd.bounds.max.0, wnd.bounds.min.1)
                ).grow(vertical_resize_area, vertical_resize_area);
                let bottom = Rect::new(
                    float2(wnd.bounds.min.0, wnd.bounds.max.1),
                    wnd.bounds.max,
                ).grow(vertical_resize_area, vertical_resize_area);

                let tl = wnd.bounds.min;
                let tl = Rect::new(
                    tl, tl
                ).grow(handle_resize_area, handle_resize_area);
                let tr = float2(wnd.bounds.max.0, wnd.bounds.min.1);
                let tr = Rect::new(
                    tr, tr,
                ).grow(handle_resize_area, handle_resize_area);

                let bl = float2(wnd.bounds.min.0, wnd.bounds.max.1);
                let bl = Rect::new(
                    bl, bl
                ).grow(handle_resize_area, handle_resize_area);
                let br = wnd.bounds.max;
                let br = Rect::new(
                    br, br
                ).grow(handle_resize_area, handle_resize_area);
                let mouse_down = io.is_mouse_down(MouseButton::Left);

                let in_header = io.has_mouse_in_rect(header);
                
                if !io.has_mouse_in_rect(header) {
                    if io.has_mouse_in_rect(left) {
                        self.cursor = CursorType::ResizeHorizontal;
                    } else if io.has_mouse_in_rect(right) {
                        self.cursor = CursorType::ResizeHorizontal;
                    } else if io.has_mouse_in_rect(bottom) {
                        self.cursor = CursorType::ResizeVertical;
                    } else if io.has_mouse_in_rect(top) {
                        self.cursor = CursorType::ResizeVertical;
                    } else if io.has_mouse_in_rect(tl) {
                        self.cursor = CursorType::ResizeNW;
                    } else if io.has_mouse_in_rect(tr) {
                        self.cursor = CursorType::ResizeNE;
                    } else if io.has_mouse_in_rect(bl) {
                        self.cursor = CursorType::ResizeNE;
                    } else if io.has_mouse_in_rect(br) {
                        self.cursor = CursorType::ResizeNW;
                    }
                }

                if mouse_down {
                    if io.has_mouse_click_in_rect(MouseButton::Left, header) {
                        wnd.bounds.min += io.mouse_delta;
                        wnd.bounds.max += io.mouse_delta;
                        io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, right) {
                        wnd.bounds.max.0 += io.mouse_delta.0;
                        io.mouse_clicked_pos[MouseButton::Left as usize].0 += io.mouse_delta.0;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, left) {
                        wnd.bounds.min.0 += io.mouse_delta.0;
                        io.mouse_clicked_pos[MouseButton::Left as usize].0 += io.mouse_delta.0;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, bottom) {
                        wnd.bounds.max.1 += io.mouse_delta.1;
                        io.mouse_clicked_pos[MouseButton::Left as usize].1 += io.mouse_delta.1;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, top) {
                        wnd.bounds.min.1 += io.mouse_delta.1;
                        io.mouse_clicked_pos[MouseButton::Left as usize].1 += io.mouse_delta.1;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, tl) {
                        wnd.bounds.min += io.mouse_delta;
                        io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, tr) {
                        wnd.bounds.max.0 += io.mouse_delta.0;
                        wnd.bounds.min.1 += io.mouse_delta.1;
                        io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, bl) {
                        wnd.bounds.min.0 += io.mouse_delta.0;
                        wnd.bounds.max.1 += io.mouse_delta.1;
                        io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                    } else if io.has_mouse_click_in_rect(MouseButton::Left, br) {
                        wnd.bounds.max += io.mouse_delta;
                        io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                    }
                }

                if io.is_mouse_down(MouseButton::Right) {
                    self.draw_list.add_rect(left.min, left.max, 0f32, 1f32, 0x8800ffff);
                    self.draw_list.add_rect(right.min, right.max, 0f32, 1f32, 0x8800ffff);
                    self.draw_list.add_rect(top.min, top.max, 0f32, 1f32, 0x8800ffff);
                    self.draw_list.add_rect(bottom.min, bottom.max, 0f32, 1f32, 0x8800ffff);


                    self.draw_list.add_rect(tl.min, tl.max, 0f32, 1f32, 0x8800ff00);
                    self.draw_list.add_rect(tr.min, tr.max, 0f32, 1f32, 0x8800ff00);
                    self.draw_list.add_rect(bl.min, bl.max, 0f32, 1f32, 0x8800ff00);
                    self.draw_list.add_rect(br.min, br.max, 0f32, 1f32, 0x8800ff00);


                    self.draw_list.add_rect(header.min, header.max, 0f32, 1f32, 0x88ff00ff);
                }
            }
            
            self.active = None;
        }
    }
}
