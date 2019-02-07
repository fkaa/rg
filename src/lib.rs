#[macro_use]
extern crate bitflags;

mod math;
mod draw;
mod widget;
mod layout;

pub use self::math::*;
pub use self::draw::*;
pub use self::widget::*;
pub use self::layout::*;

use self::math::*;

pub type Id = u32;

#[repr(u32)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
}

pub fn hash_id(val: &str) -> Id {
    let mut hash = 0xbaba_f00d_u32;

    let bytes = val.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        hash = (hash << 5) + unsafe { *bytes.get_unchecked(i) } as u32;
        
        i += 1;
    }

    hash as Id
}

struct ContextDrawInfo {
    cursor: float2,
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
            mouse_scroll: float2(0f32, 0f32)
        }
    }

    pub fn clear(&mut self) {
        self.pressed = [false; 512];
        self.mouse_pressed = [false; 5];
        self.mouse_delta = float2(0f32, 0f32);
        self.mouse_scroll = float2(0f32, 0f32);
    }

    #[inline(always)]
    pub fn has_mouse_in_rect(&self, button: MouseButton, rect: Rect) -> bool {
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


pub struct Style {
    window: WindowStyle,
    button: ButtonStyle,
}
 
impl Style {
    pub fn new() -> Self {
        Style {
            window: WindowStyle::new(),
            button: ButtonStyle::new(),
        }
    }
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
    
}

impl Context {
    pub fn new(renderer: Box<Renderer>) -> Self {
        // println!("{:#?}", font_kit::sources::fs::FsSource::new().all_families());
        let family = font_kit::sources::fs::FsSource::new()
            .select_family_by_name("Comic Sans MS")
            .unwrap();
        let handle = &family.fonts()[0];
        let default_font = Font::new(String::from("Test"), &handle, 14f32).unwrap();
        
        Context {
            windows: Vec::new(),
            window_stack: Vec::new(),
            current_window: None,
            active: None,
            style: Style::new(),
            default_font,
            last_widget_state: WidgetState::None,
            io: IoState::new(),

            draw_list: DrawList::new(),
            renderer,

            frame: 0,
        }
    }

    pub fn draw_list(&mut self) -> &mut DrawList {
        &mut self.draw_list
    }
    
    pub fn begin_frame(&mut self) {
        
    }
    
    pub fn end_frame(&mut self) {
        self.frame += 1;
    }

    pub fn set_next_window_pos(&mut self, pos: float2) {

    }

    pub fn text(&mut self, text: &str) {

    }

    pub fn draw(&mut self) {
        self.renderer.render(&self.draw_list);
        self.draw_list.clear();
    }
    
    fn item_size(&mut self, size: float2) {

    }

    fn item_add(&mut self, bb: Rect, id: Option<Id>) -> bool {
        let window = self.current_window();
        
        !window.is_clipped(bb)
    }



    #[inline(always)]
    fn current_index(&self) -> usize {
        self.window_stack[self.window_stack.len() - 1]
    }

    fn current_window(&self) -> &Window {
        &self.windows[self.window_stack[self.window_stack.len() - 1]]
    }

    fn current_window_mut(&mut self) -> &mut Window {
        &mut self.windows[self.window_stack[self.window_stack.len() - 1]]
    }
}
