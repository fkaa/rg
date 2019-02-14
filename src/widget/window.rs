use crate::{
    Context, MouseButton, Id, hash_id,
        WidgetState,
    Background,
    TextAlignment,
    math::{
        float2,
        Rect,
    },

    layout,
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
    }
}

pub struct Window {
    pub name: String,
    pub id: Id,
    
    pub bounds: Rect,
    pub layout: layout::Panel,

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
            layout: layout::Panel::new(),

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

        if wnd.flags.contains(WindowFlags::Hidden) {
            return false;
        }

        if !wnd.flags.contains(WindowFlags::NoInput) {
            let bounds = wnd.bounds;

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
                    let bounds = self.windows[i].bounds;

                    if bounds.contains(io.mouse) && pos >= p {
                        window_clicked = Some(i);
                        p = pos;
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

                self.move_stack_pos_to_front(stack_pos);
            } else if mouse_click {
                wnd.flags.insert(WindowFlags::ReadOnly);
            }
        }
        self.active = Some(idx);
        let wnd = &mut self.windows[idx];
        wnd.layout.offset = wnd.scrollbar;

        self.panel_begin(title, layout::PanelType::Window)
    }

    pub fn end(&mut self) {
        self.panel_end();

        if let Some(idx) = self.active {
            self.draw_list.push_layer(idx as u32);
            self.active = None;
        }
    }
}
