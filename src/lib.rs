mod math;
mod draw;

pub use self::math::*;
pub use self::draw::*;

use self::math::*;

pub type Id = u32;

pub fn hash_id(val: &str) -> Id {
    crc::crc32::checksum_castagnoli(val.as_bytes()) as Id
}

struct WindowPerFrameData {
    cursor: float2,
    cursor_prev_line: float2,
    cursor_max_pos: float2,
    indent: f32,
    current_text_base_offset: f32,
}

impl WindowPerFrameData {
    pub fn new() -> Self {
        WindowPerFrameData {
            cursor: float2(0f32, 0f32),
            cursor_prev_line: float2(0f32, 0f32),
            cursor_max_pos: float2(0f32, 0f32),
            indent: 0f32,
            current_text_base_offset: 0f32,
        }
    }
}

struct Window {
    name: String,
    id: Id,
    
    pos: float2,
    size: float2,

    content_region: Rect,
    clip_rect: Rect,

    data: WindowPerFrameData,
    storage: WindowStorage,
}

impl Window {
    pub fn new(label: String) -> Self {
        Window {
            name: label,
            id: 0,

            pos: float2(0f32, 0f32),
            size: float2(0f32, 0f32),

            content_region: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),
            clip_rect: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),

            data: WindowPerFrameData::new(),
            storage: WindowStorage::new(),
        }
    }

    pub fn is_clipped(&self, bounding_box: Rect) -> bool {
        self.clip_rect.outside(bounding_box)
    }
}

union StorageValue {
    pub integer: i32,
    pub float: f32,
    pub ptr: *mut (),
}

struct WindowStorage {
    pairs: Vec<(Id, StorageValue)>,
}

impl WindowStorage {
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
        }
    }
}

struct Style {
    window_padding: float2,
    frame_padding: float2
}
 
impl Style {
    pub fn new() -> Self {
        Style {
            window_padding: float2(2f32, 2f32),
            frame_padding: float2(2f32, 2f32)
        }
    }
}

struct ContextDrawInfo {
    cursor: float2,
}

struct IoState {
    display_size: float2,
    delta: f32,

    mouse_pos: float2,
    mouse_down: [bool; 5],
    mouse_wheel: f32,

    mouse_pos_prev: float2,
    mouse_pos_delta: float2,
    mouse_clicked: [bool; 5],
    mouse_clicked_pos: [float2; 5],

}

pub struct Context {
    windows: Vec<Window>,
    window_stack: Vec<usize>,
    current_window: Option<usize>,
    style: Style,
    default_font: Font,

    draw_list: DrawList,
    renderer: Box<Renderer>,
}

impl Context {
    pub fn new(renderer: Box<Renderer>) -> Self {
        // println!("{:#?}", font_kit::sources::fs::FsSource::new().all_families());
        let family = font_kit::sources::fs::FsSource::new()
            .select_family_by_name("Calibri")
            .unwrap();
        let handle = &family.fonts()[0];
        let default_font = Font::new(String::from("Test"), &handle, 24f32).unwrap();
        
        Context {
            windows: Vec::new(),
            window_stack: Vec::new(),
            current_window: None,
            style: Style::new(),
            default_font,

            draw_list: DrawList::new(),
            renderer,
        }
    }
    
    pub fn begin(&mut self, name: &str) -> bool {
        let (first_use, window_idx) = if let Some(wnd) = self.find_window(name) {
            (false, wnd)
        } else {
            (true, self.create_window(name))
        };

        self.window_stack.push(window_idx);
        self.current_window = Some(window_idx);

        true
    }

    pub fn end(&mut self) {
        self.window_stack.pop();
        self.current_window = self.window_stack.last().cloned();
    }

    pub fn button(&mut self, label: &str) -> bool {
        true
    }

    pub fn text(&mut self, text: &str) {
        let wnd = self.current_window();

        let wrap_width = 200f32;
        
        /*let text_pos = float2(wnd.data.cursor.0, wnd.data.cursor.1 + wnd.data.current_text_base_offset);
        let text_size = self.default_font.calculate_text_size(text, Some(wrap_width));

        let text_bounds = Rect::new(text_pos, text_pos + text_size);
        
        self.item_size(text_size);
        if !self.item_add(text_bounds) {
            return;
        }
        
        self.draw_list.add_text_wrapped(
            &mut *self.renderer,
            &mut self.default_font,
            text,
            text_size,
            0xffffffff
        );*/
        //let s = format!("{:#?}", self.default_font.metrics);
        //self.draw_list.add_text(&mut *self.renderer, &mut self.default_font, &s, float2(100f32, 100f32), 0xffffffff);
    }

    pub fn draw(&mut self) {
        self.renderer.render(&self.draw_list);
        self.draw_list.clear();
    }
    
    fn item_size(&mut self, size: float2) {
        let window = self.current_window_mut();

        let line_height = 14f32;
        
        window.data.cursor_prev_line = float2(window.data.cursor.0 + size.0, window.data.cursor.1);
        window.data.cursor = float2(
            window.pos.0 + window.data.indent,
            window.data.cursor.1 + line_height,
        );
        window.data.cursor_max_pos = float2(
            window.data.cursor_max_pos.0.max(window.data.cursor_prev_line.0),
            window.data.cursor_max_pos.1.max(window.data.cursor.1),
        );
    }

    fn item_add(&mut self, bb: Rect, id: Option<Id>) -> bool {
        let window = self.current_window();
        
        window.is_clipped(bb)
    }

    fn create_window(&mut self, name: &str) -> usize {
        let wnd = Window::new(name.into());
        let idx = self.windows.len();

        self.windows.push(wnd);

        idx
    }

    fn find_window(&self, name: &str) -> Option<usize> {
        let id = hash_id(name);

        self.windows.iter().position(|ref wnd| wnd.id == id)
    }

    fn current_window(&self) -> &Window {
        &self.windows[self.window_stack[self.window_stack.len() - 1]]
    }

    fn current_window_mut(&mut self) -> &mut Window {
        &mut self.windows[self.window_stack[self.window_stack.len() - 1]]
    }
}
