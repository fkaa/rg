#[macro_use]
extern crate bitflags;

mod math;
mod draw;
mod widget;
mod layout;
mod style;
mod collections;

pub use self::math::*;
pub use self::draw::*;
pub use self::widget::*;
pub use self::layout::*;
pub use self::style::*;
pub use self::collections::*;

use self::math::*;

use std::time::Instant;

pub type Id = u32;

#[derive(Copy, Clone)]
// #[repr(u32)]
pub enum DataType {
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

pub fn hash_id_seed(val: &str, seed: u32) -> Id {
    let mut hash = seed;

    let bytes = val.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        hash = hash.wrapping_mul(16777619) ^ unsafe { *bytes.get_unchecked(i) } as u32;
        
        i += 1;
    }

    hash as Id
}

pub fn hash_id(val: &str) -> Id {
    hash_id_seed(val, 0xbaba_f00d_u32)
}

struct ContextDrawInfo {
    cursor: float2,
}

pub struct Context {
    windows: Vec<Window>,
    window_stack: Vec<usize>,
    current_window: Option<usize>,
    active: Option<usize>,
    style: Style,
    default_font: Font,
    last_widget_state: WidgetState,
    pub io: IoState,

    pub draw_list: DrawList,
    pub renderer: Box<Renderer>,

    frame: u32,

    now: Instant,
    active_id: Id,
    
    text_edit_id: Id,
    text_edit_state: EditState,
    current_panel: usize,
    panel_index: usize,
    panel_stack: Vec<Panel>,
    id_stack: Vec<Id>,
    pub tab_bars: Pool<TabBar>,
    pub current_tab_bar: Vec<PoolIndex>,
    tab_sorter: Vec<TabItemSort>,
    
    pub prev_cursor: CursorType,
    pub cursor: CursorType,
}

impl Context {
    pub fn new(renderer: Box<Renderer>) -> Self {
        // println!("{:#?}", font_kit::sources::fs::FsSource::new().all_families());
        let family = font_kit::sources::fs::FsSource::new()
            .select_family_by_name("DejaVu Sans")
            .unwrap();
        let handle = &family.fonts()[0];
        let default_font = Font::new(String::from("Test"), &handle, 14f32).unwrap();
        
        Context {
            windows: Vec::new(),
            window_stack: Vec::new(),
            current_window: None,
            active: None,
            style: Style::dark_style(),
            default_font,
            last_widget_state: WidgetState::None,
            io: IoState::new(),

            draw_list: DrawList::new(),
            renderer,

            frame: 0,

            now: Instant::now(),
            active_id: 0,

            text_edit_id: 0,
            text_edit_state: EditState::new(),
            current_panel: 0,
            panel_index: 0,
            panel_stack: Vec::new(),
            id_stack: Vec::new(),
            tab_bars: Pool::new(),
            current_tab_bar: Vec::new(),
            tab_sorter: Vec::new(),
            
            prev_cursor: CursorType::Default,
            cursor: CursorType::Default,
        }
    }

    

    pub fn draw_list(&mut self) -> &mut DrawList {
        &mut self.draw_list
    }
    
    pub fn begin_frame(&mut self) {
        let delta = {
            let duration = Instant::now() - self.now;
            let seconds = duration.as_secs() as f32;
            let ms = duration.subsec_millis() as f32 / 1000f32;

            seconds + ms
        };
        self.io.delta = delta;
        self.now = Instant::now();
        
        if self.is_editing_text() {
            for action in self.io.text_edit_actions.drain(..) {
                match action {
                    TextEditAction::Char(ch) => self.text_edit_state.insert_char(ch),
                    TextEditAction::Key(key) => self.text_edit_state.key_press(key),
                }
            }
        }
    }
    
    pub fn end_frame(&mut self) {
        self.frame += 1;

        if self.prev_cursor != self.cursor {
            self.io.cursor = Some(self.cursor);
        }

        self.panel_index = 0;
        self.prev_cursor = self.cursor;
        self.cursor = CursorType::Default;
    }

    fn id(&self, text: &str) -> Id {
        if let Some(id) = self.id_stack.last() {
            hash_id_seed(text, *id)
        } else {
            hash_id(text)
        }
    }

    pub fn set_active_id(&mut self, id: Id) {
        //self.active_id_just_activated = self.active_id != id;
        self.active_id = id;
    }

    pub fn is_editing_text(&self) -> bool {
        self.text_edit_state.id != 0
    }

    pub fn set_next_window_pos(&mut self, pos: float2) {

    }

    pub fn text(&mut self, text: &str) {

    }

    pub fn draw(&mut self) {
        self.draw_list.swap_window_stack_indices(&self.window_stack);
        self.renderer.render(&self.draw_list);
        self.draw_list.clear();
    }

    #[inline(always)]
    fn current_index(&self) -> usize {
        self.active.unwrap()
        //self.window_stack[self.window_stack.len() - 1]
    }
}

#[repr(u32)]
pub enum MouseButton {
    Left = 1,
    Middle = 2,
    Right = 3,
}

pub struct IoState {
    pub display_size: float2,
    pub delta: f32,

    pub mouse_pressed: [bool; 5],
    pub mouse_down: [bool; 5],
    pub mouse_released: [bool; 5],
    pub mouse_clicked_pos: [float2; 5],
    pub mouse: float2,
    pub mouse_delta: float2,
    pub mouse_scroll: float2,
    pub pressed: [bool; 512],
    pub down: [bool; 512],

    //pub characters: Vec<char>,
    pub text_edit_actions: Vec<TextEditAction>,
    pub cursor: Option<CursorType>,
}

impl IoState {
    pub fn new() -> Self {
        IoState {
            display_size: float2(0f32, 0f32),
            delta: 0f32,
            pressed: [false; 512],
            down: [false; 512],
            mouse_pressed: [false; 5],
            mouse_down: [false; 5],
            mouse_released: [false; 5],
            mouse_clicked_pos: [float2(0f32, 0f32); 5],
            mouse: float2(0f32, 0f32),
            mouse_delta: float2(0f32, 0f32),
            mouse_scroll: float2(0f32, 0f32),

            text_edit_actions: Vec::new(),
            cursor: None,
        }
    }

    pub fn clear(&mut self) {
        self.pressed = [false; 512];
        self.mouse_pressed = [false; 5];
        self.mouse_released = [false; 5];
        self.mouse_delta = float2(0f32, 0f32);
        self.mouse_scroll = float2(0f32, 0f32);
        self.text_edit_actions.clear();
    }

    pub fn is_key_down(&self, key: usize) -> bool {
        self.down[key]
    }
    
    #[inline(always)]
    pub fn has_mouse_in_rect(&self, rect: Rect) -> bool {
        let pos = self.mouse;

        rect.contains(pos)
    }

    #[inline(always)]
    pub fn has_mouse_click_in_rect(&self, button: MouseButton, rect: Rect) -> bool {
        let pos = unsafe { *self.mouse_clicked_pos.get_unchecked(button as usize) };

        rect.contains(pos)
    }

    #[inline(always)]
    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        unsafe { *self.mouse_down.get_unchecked(button as usize) }
    }

    #[inline(always)]
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        unsafe { *self.mouse_pressed.get_unchecked(button as usize) }
    }
    
    #[inline(always)]
    pub fn is_mouse_released(&self, button: MouseButton) -> bool {
        unsafe { *self.mouse_released.get_unchecked(button as usize) }
    }
}
