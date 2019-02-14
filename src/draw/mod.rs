mod text;

use std::ops::Range;
pub use self::text::*;

use crate::math::*;

pub type TextureHandle = *mut ();

pub struct Image {
    handle: TextureHandle,
    size: [u16; 2],
    region: [u16; 4],
}

pub struct DynamicImage {
    handle: TextureHandle,
    size: [u16; 2],
    regions: [u16; 8],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: u32
}

impl Vertex {
    pub fn new(pos: float2, uv: float2, color: u32) -> Self {
        Vertex {
            pos: [pos.0, pos.1],
            uv: [uv.0, uv.1],
            color
        }
    }
}

pub trait Renderer {
    fn resize(&mut self, width: f32, height: f32);
    fn render(&mut self, list: &DrawList);
    
    fn create_texture_a8(&mut self, width: u32, height: u32) -> (TextureHandle, TextureHandle);
    fn upload_a8(&mut self, handle: TextureHandle, x: u32, y: u32, width: u32, height: u32, data: &[u8], stride: u32);
}

pub struct DrawCommand {
    pub index_offset: u32,
    pub index_count: u32,
    clip_rect: float4,
    pub texture_id: TextureHandle,
}

impl DrawCommand {
    pub fn new() -> Self {
        DrawCommand {
            index_offset: 0,
            index_count: 0,
            clip_rect: float4(0f32, 0f32, 800f32, 800f32),
            texture_id: ::std::ptr::null_mut() as _
        }
    }
}

#[derive(Clone)]
pub struct DrawLayer {
    command_range: Range<usize>,
    position: u32,
}

impl DrawLayer  {
    pub fn new(command_range: Range<usize>, position: u32) -> Self {
        DrawLayer {
            command_range,
            position,
        }
    }
}

pub struct DrawList {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub index_offset: u32,
    path: Vec<float2>,

    commands: Vec<DrawCommand>,
    command_start: usize,
    layers: Vec<DrawLayer>,
    layer_index: usize,

    clip_stack: Vec<float4>,
    texture_stack: Vec<TextureHandle>
}

impl DrawList {
    pub fn new() -> Self {
        DrawList {
            vertices: Vec::new(),
            indices: Vec::new(),
            index_offset: 0,
            path: Vec::new(),

            commands: vec![DrawCommand::new()],
            command_start: 0,
            layers: Vec::new(),
            layer_index: 0,
            
            clip_stack: Vec::new(),
            texture_stack: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.command_start = 0;
        self.layers.clear();
        self.index_offset = 0;
        self.layer_index = 0;
        self.vertices.clear();
        self.indices.clear();
        self.path.clear();
        self.clip_stack.clear();
        self.texture_stack.clear();
    }

    pub fn push_layer(&mut self, position: u32) {
        let layer = DrawLayer::new(self.command_start..self.commands.len(), position);
        self.layers.push(layer);
        
        self.command_start = self.commands.len();
        self.push_draw_cmd();
    }
    
    pub fn push_clip_rect(&mut self, rect: float4) {
        self.clip_stack.push(rect);

        let current_rect = self.current_cmd().clip_rect;
        //if current_rect != rect {
        //    self.push_draw_cmd();
        //}
    }

    pub fn pop_clip_rect(&mut self) {
        self.clip_stack.pop();
    }

    pub fn set_texture(&mut self, texture: TextureHandle) {        
        if self.current_cmd().texture_id == ::std::ptr::null_mut() {
            self.current_cmd().texture_id = texture;
        } else if self.current_cmd().texture_id != texture {
            self.push_draw_cmd();
            self.current_cmd().texture_id = texture;
        }
    }
    
    pub fn push_texture(&mut self, texture: TextureHandle) {
        self.texture_stack.push(texture);

        let current_tex = self.current_cmd().texture_id;
        if current_tex != texture {
            self.push_draw_cmd();
        }
    }

    pub fn pop_texture(&mut self) {
        self.texture_stack.pop();
    }


    pub fn swap_window_stack_indices(&mut self, stack_positions: &[usize]) {
        let mut i = 0;
        let len = self.layers.len();
        
        while i < len {
            let layer = unsafe { self.layers.get_unchecked_mut(i) };
            layer.position = stack_positions[layer.position as usize] as u32;
            
            i += 1;
        }

        self.layers.sort_unstable_by_key(|l| l.position);
    }
    
    pub fn commands<'a>(&'a self) -> Vec<&'a [DrawCommand]> {
        let mut layers = Vec::new();
        
        for layer in &self.layers {
            layers.push(&self.commands[layer.command_range.clone()]);
        }

        layers
    }

    fn push_draw_cmd(&mut self) {
        let rect = self.current_clip_rect();
        let texture = self.current_texture();
        
        self.commands.push(DrawCommand {
            index_offset: self.indices.len() as u32 * 2,
            index_count: 0,
            clip_rect: rect,
            texture_id: texture
        })
    }

    fn current_cmd(&mut self) -> &mut DrawCommand {
        if self.commands.len() > 0 {
            self.commands.last_mut().unwrap()
        } else {
            self.commands.push(DrawCommand::new());

            self.commands.last_mut().unwrap()
        }
    }

    fn current_texture(&self) -> TextureHandle {
        self.texture_stack.last().cloned().unwrap_or(::std::ptr::null() as *const () as TextureHandle)
    }

    fn current_clip_rect(&self) -> float4 {
        self.clip_stack.last().cloned().unwrap_or(float4(0f32, 0f32, 0f32, 0f32))
    }

    pub fn add_line(&mut self, a: float2, b: float2, color: u32) {
        self.path()
            .line(a + float2(0.5, 0.5))
            .line(b + float2(0.5, 0.5))
            .stroke(1f32, false, color);
    }

    fn path_rect(&mut self, a: float2, b: float2, rounding: f32) -> PathBuilder {
        let a = a + float2(0.0f32, 0.0f32);
        let b = b - float2(0.0f32, 0.0f32);
        if rounding <= 0f32 {
            self.path()
                .line(a)
                .line(float2(b.0, a.1))
                .line(b)
                .line(float2(a.0, b.1))
        } else {
            let rounding = rounding.min((b.0 - a.0).abs() * (1f32) - 1f32);
            let rounding = rounding.min((b.1 - a.1).abs() * (1f32) - 1f32);

            let r0 = rounding;
            let r1 = rounding;
            let r2 = rounding;
            let r3 = rounding;

            self.path()
                .arc_fast(float2(a.0 + r0, a.1 + r0), r0, 6, 9)
                .arc_fast(float2(b.0 - r1, a.1 + r1), r1, 9, 12)
                .arc_fast(float2(b.0 - r2, b.1 - r2), r2, 0, 3)
                .arc_fast(float2(a.0 + r3, b.1 - r3), r3, 3, 6)
        }
    }

    pub fn add_rect(&mut self, a: float2, b: float2, rounding: f32, thickness: f32, color: u32) {
        self.path_rect(
            a + float2(0.5, 0.5),
            b - float2(0.5, 0.5),
            rounding
        ).stroke(thickness, true, color);
    }

    pub fn add_rect_filled(&mut self, a: float2, b: float2, rounding: f32, color: u32) {
        self.path_rect(a, b, rounding).fill(color);
    }


    pub fn add_poly_line(&mut self, thickness: f32, closed: bool, color: u32) {
        // TEMP
        let uv = float2(0f32, 0f32);

        let vertex_count = self.path.len() * 4;
        let points_len = if closed {
            self.path.len()
        } else {
            self.path.len() - 1
        };
        let index_count = points_len * 6;

        self.vertices.reserve(vertex_count);
        self.current_cmd().index_count += index_count as u32;

        for i in 0..points_len {
            let offset = self.index_offset;

            let p1 = self.path[i];
            let p2 = if (i + 1) == self.path.len() {
                self.path[0]
            } else {
                self.path[i + 1]
            };

            let diff = {
                let diff = p2 - p1;
                let len = {
                    let len = diff.length();
                    if len > 0f32 {
                        len
                    } else {
                        1f32
                    }
                };

                diff * (1f32 / len) * thickness * 0.5f32
            };

            self.vertices.push(Vertex::new(p1 + float2( diff.1, -diff.0), uv, color));
            self.vertices.push(Vertex::new(p2 + float2( diff.1, -diff.0), uv, color));
            self.vertices.push(Vertex::new(p2 + float2(-diff.1,  diff.0), uv, color));
            self.vertices.push(Vertex::new(p1 + float2(-diff.1,  diff.0), uv, color));

            self.indices.push(offset as u16 + 0);
            self.indices.push(offset as u16 + 1);
            self.indices.push(offset as u16 + 2);
            self.indices.push(offset as u16 + 0);
            self.indices.push(offset as u16 + 2);
            self.indices.push(offset as u16 + 3);

            self.index_offset += 4;
        }
    }

    pub fn add_poly_fill(&mut self, color: u32) {
        // TEMP
        let uv = float2(0f32, 0f32);

        let vertex_count = self.path.len();
        let index_count = (self.path.len() - 2) * 3;

        self.vertices.reserve(vertex_count);
        self.current_cmd().index_count += index_count as u32;

        for i in 0..vertex_count {
            self.vertices.push(Vertex::new(self.path[i], uv, color));
        }

        let offset = self.index_offset as u16;
        for i in 2..self.path.len() {
            self.indices.push(offset);
            self.indices.push(offset + i as u16 - 1);
            self.indices.push(offset + i as u16);
        }

        self.index_offset += vertex_count as u32
    }

    pub fn path(&mut self) -> PathBuilder {
        PathBuilder::new(self)
    }

}



pub struct PathBuilder<'a> {
    list: &'a mut DrawList
}

fn build_arc_lut() -> [float2; 12] {
    let mut arr = [float2(0f32, 0f32); 12];
    for x in 0..12 {
        let a = ((x as f32) / 12f32) * ::std::f32::consts::PI * 2f32;
        arr[x] = float2(a.cos(), a.sin());
    }
    arr
}

impl<'a> PathBuilder<'a> {
    pub fn new(list: &'a mut DrawList) -> Self {
        PathBuilder {
            list
        }
    }

    pub fn line(mut self, pos: float2) -> Self {
        self.list.path.push(pos);
        self
    }

    pub fn arc_fast(mut self, center: float2, radius: f32, min: u32, max: u32) -> Self {
        let lut = build_arc_lut();

        for x in min..(max + 1) {
            let c = lut[(x % 12) as usize];
            self.list.path.push(center + c * float2(radius, radius));
        }

        self
    }

    pub fn arc(mut self, center: float2, radius: f32, min: f32, max: f32, segments: u32) -> Self {
        if radius == 0f32 {
            self.list.path.push(center);
        }

        for i in 0..(segments - 1) {
            let a = min + (i as f32 / segments as f32) * (max - min);
            self.list.path.push(float2(center.0 + a.cos() * radius, center.1 + a.sin() * radius));
        }
        
        self
    }

    pub fn stroke(self, thickness: f32, closed: bool, color: u32) -> Self {
        self.list.add_poly_line(thickness, closed, color);
        self.list.path.clear();

        self
    }

    pub fn fill(self, color: u32) {
        self.list.add_poly_fill(color);
        self.list.path.clear();
    }
}
