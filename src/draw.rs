use crate::math::*;

use rect_packer::{
    Packer,
    Rect,
};

pub type TextureHandle = *mut ();

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
    fn render(&mut self, list: &DrawList);
    
    fn create_texture_a8(&mut self, width: u32, height: u32) -> (TextureHandle, TextureHandle);
    fn upload_a8(&mut self, handle: TextureHandle, x: u32, y: u32, width: u32, height: u32, data: &[u8], stride: u32);
}

pub struct DrawCommand {
    pub index_count: u32,
    clip_rect: float4,
    pub texture_id: TextureHandle,
}

impl DrawCommand {
    pub fn new() -> Self {
        DrawCommand {
            index_count: 0,
            clip_rect: float4(0f32, 0f32, 800f32, 800f32),
            texture_id: ::std::ptr::null_mut() as _
        }
    }
}


pub struct FontAtlasPage {
    texture_handle: TextureHandle,
    srv_handle: TextureHandle,
    packer: Packer,
    full: bool,
}

impl FontAtlasPage {
    pub fn new(renderer: &mut Renderer, size: u32) -> Self {
        let (texture_handle, srv_handle) = renderer.create_texture_a8(size, size);

        let config = rect_packer::Config {
            width: size as i32,
            height: size as i32,

            border_padding: 1,
            rectangle_padding: 1,
        };
        let mut packer = Packer::new(config);
        packer.pack(2, 2, false);
        let pixel = [0xff, 0xff, 0xff, 0xff];
        renderer.upload_a8(texture_handle, 0, 0, 2, 2, &pixel, 2);
        
        FontAtlasPage {
            texture_handle,
            srv_handle,
            packer,
            full: false,
        }
    }

    pub fn pack(&mut self, w: i32, h: i32) -> Option<(u32, u32, u32, u32)> {
        if self.full {
            return None;
        }

        if let Some(Rect { x, y, width, height }) = self.packer.pack(w, h, false) {
            Some((x as u32, y as u32, width as u32, height as u32))
        } else {
            self.full = true;
            
            None
        }
    }
}

pub struct FontAtlas {
    pages: Vec<FontAtlasPage>,
    size: u32,
}

impl FontAtlas {
    pub fn new(size: u32) -> Self {
        FontAtlas {
            pages: Vec::new(),
            size
        }
    }

    pub fn pack(&mut self, renderer: &mut Renderer, width: i32, height: i32) -> Option<(usize, u32, u32, u32, u32)> {
        assert!(width < self.size as i32);
        assert!(height < self.size as i32);

        let mut dims = None;
        let mut index = 0;

        'outer: loop {
            index = 0;
            for page in &mut self.pages {
                dims = page.pack(width, height);

                if dims.is_some() {
                    break 'outer;
                }
                
                index += 1;
            }

            self.pages.push(FontAtlasPage::new(renderer, self.size));
        }

        if let Some((a, b, c, d)) = dims {
            Some((index, a, b, c, d))
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FontGlyph {
    page: u16,
    
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    x_advance: f32,

    u: f32,
    v: f32,
    u_2: f32,
    v_2: f32,
}

impl FontGlyph {
    pub fn new() -> Self {
        FontGlyph {
            ..
            ::std::default::Default::default()
        }
    }
}

pub struct Font {
    name: String,
    rasterize_cache: font_kit::canvas::Canvas,
    font_face: font_kit::font::Font,
    pub font_size: f32,
    pub font_factor: f32,
    pub metrics: font_kit::metrics::Metrics,

    font_atlas: FontAtlas,
    glyph_indices: Vec<u16>,
    glyphs: Vec<FontGlyph>,
}

impl Font {
    pub fn new(name: String, handle: &font_kit::handle::Handle, font_size: f32) -> Result<Self, font_kit::error::FontLoadingError> {
        let font_face = font_kit::font::Font::from_handle(handle)?;
        let metrics = font_face.metrics();
        
        Ok(Font {
            name,
            rasterize_cache: font_kit::canvas::Canvas::new(&euclid::Size2D::new(128u32, 128u32), font_kit::canvas::Format::A8),
            font_face,
            font_size,
            font_factor: font_size / metrics.units_per_em as f32,// / font_size,
            metrics,

            font_atlas: FontAtlas::new(1024),
            
            glyph_indices: vec![0u16; u16::max_value() as usize],
            glyphs: vec![FontGlyph::new()],
        })
    }

    fn create_glyph(&mut self, renderer: &mut Renderer, id: u16) -> Option<FontGlyph> {
        let glyph_id = self.font_face.glyph_for_char(::std::char::from_u32(id as u32)?)?;
        let font_size = self.font_size;
        let hinting = font_kit::hinting::HintingOptions::None;
        let raster = font_kit::canvas::RasterizationOptions::GrayscaleAa;
        
        let glyph_bounds = self.font_face.raster_bounds(
            glyph_id,
            self.font_size,
            &euclid::Point2D::zero(),
            hinting,
            raster,
        ).ok()?;

        println!("raster_bounds: {:?}", glyph_bounds);

        if glyph_bounds.size.width == 0 && glyph_bounds.size.height == 0 {
            let advance = self.font_face.advance(glyph_id).ok()?;
            
            let glyph = FontGlyph {
                page: 0u16,
                
                x: 0f32,
                y: 0f32,
                w: 0f32,
                h: 0f32,
                x_advance: advance.x,

                u: 0f32,
                v: 0f32,
                u_2: 0f32,
                v_2: 0f32,
            };

            // add glyph
            let idx = self.glyphs.len();
            self.glyph_indices[id as usize] = idx as u16;
            self.glyphs.push(glyph);

            Some(glyph)
        } else {
            for pixel in &mut self.rasterize_cache.pixels {
                *pixel = 0;
            }

            self.font_face.rasterize_glyph(
                &mut self.rasterize_cache,
                glyph_id,
                self.font_size,
                &euclid::Point2D::zero(),
                hinting,
                raster,
            ).ok()?;

            let advance = self.font_face.advance(glyph_id).ok()?;
            let origin = self.font_face.origin(glyph_id).ok()?;
            
            let (idx, x, y, width, height) = self.font_atlas.pack(
                renderer,
                glyph_bounds.size.width,
                glyph_bounds.size.height
            )?;

            let tex_handle = self.font_atlas.pages[idx].texture_handle;
            // let srv_handle = self.font_atlas.pages[idx].srv_handle;
            renderer.upload_a8(
                tex_handle,
                x,
                y,
                width,
                height,
                &self.rasterize_cache.pixels,
                self.rasterize_cache.stride as u32
            );

            let size = self.font_atlas.size as f32;
            let u = x as f32 / size;
            let v = y as f32 / size;
            let u_2 = (x + width) as f32 / size;
            let v_2 = (y + height) as f32 / size;

            let bounds = self.font_face.typographic_bounds(glyph_id).ok()?;
            let ratio = self.font_size / self.metrics.units_per_em as f32;

            println!("{}: {}", id, bounds.origin.y * ratio); 
            
            let glyph = FontGlyph {
                page: idx as u16,
                
                x: glyph_bounds.origin.x as f32,
                y: glyph_bounds.origin.y as f32,
                w: glyph_bounds.size.width as f32,
                h: glyph_bounds.size.height as f32,
                x_advance: advance.x,

                u,
                v,
                u_2,
                v_2,
            };

            println!("{:#?}", glyph);

            // add glyph
            let idx = self.glyphs.len();
            self.glyph_indices[id as usize] = idx as u16;
            self.glyphs.push(glyph);

            Some(glyph)
        }
    }

    pub fn get_glyph(&mut self, renderer: &mut Renderer, id: u16) -> Option<FontGlyph> {
        let idx = self.glyph_indices[id as usize];

        if idx == 0 {
            self.create_glyph(renderer, id)
        } else {
            Some(self.glyphs[idx as usize])
        }
    }

    pub fn calculate_text_size(&mut self, renderer: &mut Renderer, text: &str, wrap: Option<f32>) -> float2 {
        let mut cursor_x = 0f32;
        let mut cursor_y = 0f32;

        let mut max_x = 0f32;

        let advance_y = (self.font_size / (self.metrics.ascent + self.metrics.descent)) * self.metrics.ascent;

        for ch in text.chars() {
            if ch == '\n' {
                cursor_y += advance_y;
                cursor_x = 0f32;
                continue;
            }
            
            if let Some(glyph) = self.get_glyph(renderer, ch as u16) {
                let advance = glyph.x_advance * self.font_factor;

                cursor_x += advance;
                if let Some(wrap) = wrap {
                    if cursor_x >= wrap {
                        cursor_y += advance_y;
                        cursor_x = advance;
                    }
                }
            }

            max_x = max_x.max(cursor_x);
        }

        float2(max_x, cursor_y + advance_y)
    }
}

pub struct DrawList {
    commands: Vec<DrawCommand>,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub index_offset: u32,
    path: Vec<float2>,

    clip_stack: Vec<float4>,
    texture_stack: Vec<TextureHandle>
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
    
    /*use ::std::f32::consts::PI;
    
    [
        float2(0f32, 0f32),
        float2((1f32 / 12f32 * PI * 2f32).cos(), (1f32 / 12f32 * PI * 2f32).sin()),
        float2((2f32 / 12f32 * PI * 2f32).cos(), (2f32 / 12f32 * PI * 2f32).sin()),
        float2((3f32 / 12f32 * PI * 2f32).cos(), (3f32 / 12f32 * PI * 2f32).sin()),
        float2((4f32 / 12f32 * PI * 2f32).cos(), (4f32 / 12f32 * PI * 2f32).sin()),
        float2((5f32 / 12f32 * PI * 2f32).cos(), (5f32 / 12f32 * PI * 2f32).sin()),
        float2((6f32 / 12f32 * PI * 2f32).cos(), (6f32 / 12f32 * PI * 2f32).sin()),
        float2((7f32 / 12f32 * PI * 2f32).cos(), (7f32 / 12f32 * PI * 2f32).sin()),
        float2((8f32 / 12f32 * PI * 2f32).cos(), (8f32 / 12f32 * PI * 2f32).sin()),
        float2((9f32 / 12f32 * PI * 2f32).cos(), (9f32 / 12f32 * PI * 2f32).sin()),
        float2((10f32 / 12f32 * PI * 2f32).cos(), (10f32 / 12f32 * PI * 2f32).sin()),
        float2((11f32 / 12f32 * PI * 2f32).cos(), (11f32 / 12f32 * PI * 2f32).sin()),
    ]*/
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


impl DrawList {
    pub fn new() -> Self {
        DrawList {
            commands: vec![DrawCommand::new()],
            vertices: Vec::new(),
            indices: Vec::new(),
            index_offset: 0,
            path: Vec::new(),

            clip_stack: Vec::new(),
            texture_stack: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.index_offset = 0;
        self.vertices.clear();
        self.indices.clear();
        self.path.clear();
        self.clip_stack.clear();
        self.texture_stack.clear();
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

    pub fn add_text(&mut self, renderer: &mut Renderer, font: &mut Font, text: &str, pos: float2, color: u32) {
        let mut cursor_x = pos.0;
        let mut cursor_y = pos.1;

        for ch in text.chars() {
            if ch == '\n' {
                cursor_y += (font.font_size / (font.metrics.ascent + font.metrics.descent)) * font.metrics.ascent;
                cursor_x = pos.0;
                continue;
            }
            if let Some(glyph) = font.get_glyph(renderer, ch as u16) {
                let texture = font.font_atlas.pages[glyph.page as usize].srv_handle;
                self.set_texture(texture);

                let cursor_y = cursor_y.ceil();
                
                let x = (cursor_x + glyph.x).round();
                let y = (cursor_y - glyph.y).round();
                let w = x + glyph.w;
                let h = y - glyph.h;
                
                self.vertices.push(Vertex::new(float2(x, y), float2(glyph.u,   glyph.v_2), color));
                self.vertices.push(Vertex::new(float2(x, h), float2(glyph.u,   glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, h), float2(glyph.u_2, glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, y), float2(glyph.u_2, glyph.v_2), color));

                cursor_x += glyph.x_advance * font.font_factor;
                
                let offset = self.index_offset as u16;                
                self.indices.push(offset + 0);
                self.indices.push(offset + 1);
                self.indices.push(offset + 2);
                self.indices.push(offset + 0);
                self.indices.push(offset + 2);
                self.indices.push(offset + 3);
                
                self.index_offset += 4;
                self.current_cmd().index_count += 6;
            }
        }
    }

    pub fn add_text_wrapped(&mut self, renderer: &mut Renderer, font: &mut Font, text: &str, pos: float2, wrap: f32, color: u32) {
        let mut cursor_x = pos.0;
        let mut cursor_y = pos.1;
        let advance_y = (font.font_size / (font.metrics.ascent + font.metrics.descent)) * font.metrics.ascent;

        for ch in text.chars() {
            if ch == '\n' {
                cursor_y += advance_y;
                cursor_x = pos.0;
                continue;
            }
            if let Some(glyph) = font.get_glyph(renderer, ch as u16) {
                let texture = font.font_atlas.pages[glyph.page as usize].srv_handle;
                self.set_texture(texture);

                let cursor_y_ceil = cursor_y.ceil();
                
                let x = (cursor_x + glyph.x).round();
                let y = (cursor_y_ceil - glyph.y).round();
                let w = x + glyph.w;
                let h = y - glyph.h;
                
                self.vertices.push(Vertex::new(float2(x, y), float2(glyph.u,   glyph.v_2), color));
                self.vertices.push(Vertex::new(float2(x, h), float2(glyph.u,   glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, h), float2(glyph.u_2, glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, y), float2(glyph.u_2, glyph.v_2), color));

                let advance = glyph.x_advance * font.font_factor;
                cursor_x += advance;
                if cursor_x >= wrap {
                    cursor_y += advance_y;
                    cursor_x = pos.0;
                }
                
                let offset = self.index_offset as u16;                
                self.indices.push(offset + 0);
                self.indices.push(offset + 1);
                self.indices.push(offset + 2);
                self.indices.push(offset + 0);
                self.indices.push(offset + 2);
                self.indices.push(offset + 3);
                
                self.index_offset += 4;
                self.current_cmd().index_count += 6;
            }
        }
    }

    pub fn add_poly_line(&mut self, thickness: f32, closed: bool, color: u32) {
        // TEMP
        let uv = float2(0f32, 0f32);

        let vertex_count = self.path.len() * 4;
        let index_count = self.path.len() * 6;
        let points_len = if closed {
            self.path.len()
        } else {
            self.path.len() - 1
        };

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

    pub fn commands(&self) -> &Vec<DrawCommand> {
        &self.commands
    }

    fn push_draw_cmd(&mut self) {
        let rect = self.current_clip_rect();
        let texture = self.current_texture();

        self.commands.push(DrawCommand { index_count: 0, clip_rect: rect, texture_id: texture })
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
}

pub struct QuadIndexU16Iterator {
    index: u32,
    vertex: u16,
}

impl Iterator for QuadIndexU16Iterator {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<u16> {
        const INDEX_OFFSETS: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let val = self.vertex + INDEX_OFFSETS[(self.index % 6) as usize];

        self.index += 1;

        if self.index % 6 == 0 {
            self.vertex += 4;
        }

        Some(val)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (<u16>::max_value() as _, None)
    }
}

pub fn index_u16() -> QuadIndexU16Iterator {
    QuadIndexU16Iterator { index: 0, vertex: 0 }
}
