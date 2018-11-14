#![feature(placement_in_syntax)]
#![feature(collection_placement)]
#![feature(jokler)]

extern crate bitflags;
extern crate crc;
extern crate rusttype;

mod math;
mod draw;

pub use math::*;
pub use draw::*;

pub type Id = u32;

pub fn hash_id(val: &str) -> Id {
    crc::crc32::checksum_castagnoli(val.as_bytes()) as Id
}

struct Window<'a> {
    name: &'a str,
    id: Id,
    pos: float2,
    size: float2,

}

struct Style {
    window_padding: float2,
    frame_padding: float2
}

struct ContextDrawInfo {
    cursor: float2
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

struct Context<'a> {
    windows: Vec<Window<'a>>,
    window_stack: Vec<usize>,
    current_window: Option<usize>,
    style: Style,
    draw_info: ContextDrawInfo
}

impl<'a> Context<'a> {
    pub fn begin(&mut self, name: &'a str) -> bool {
        let window_idx = if let Some(wnd) = self.find_window(name) {
            wnd
        } else {
            self.create_window(name)
        };

        self.window_stack.push(window_idx);
        self.current_window = Some(window_idx);

        true
    }

    pub fn end(&mut self) {
        self.window_stack.pop();
        self.current_window = self.window_stack.last().cloned();
    }

    pub fn button(&mut self, label: &'a str) -> bool {



        true
    }

    fn item_size(&mut self, bb: float4, padding: f32) {
        // TODO: advance draw state
    }

    fn item_add(&mut self, bb: float4, id: Option<Id>) -> bool {
        // TODO: clipping

        false
    }

    fn create_window(&mut self, name: &'a str) -> usize {
        let wnd = Window {
            name,
            id: hash_id(name),
            pos: float2(0f32, 0f32),
            size: float2(100f32, 100f32)
        };
        let idx = self.windows.len();

        self.windows.push(wnd);

        idx
    }

    fn find_window(&self, name: &str) -> Option<usize> {
        let id = hash_id(name);

        self.windows.iter().position(|ref wnd| wnd.id == id)
    }
}

fn test(cxt: &mut Context) {
    if cxt.begin("test") {

        if cxt.button("press me!") {
            println!("{}", "pressed button");
        }

        cxt.end();
    }
}
