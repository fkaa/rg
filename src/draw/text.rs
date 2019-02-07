use crate::math::*;
use super::{
    TextureHandle, Renderer, DrawList, Vertex
};

use rect_packer::{
    Packer,
    Rect,
};

pub struct FontAtlasPage {
    texture_handle: TextureHandle,
    pub srv_handle: TextureHandle,
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
    pub pages: Vec<FontAtlasPage>,
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
    pub page: u16,
    
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub x_advance: f32,

    pub u: f32,
    pub v: f32,
    pub u_2: f32,
    pub v_2: f32,
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

    pub font_atlas: FontAtlas,
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

    pub fn height(&self) -> f32 {
        self.font_size
    }

    pub fn text_width(&mut self, renderer: &mut Renderer, text: &str) -> f32 {
        let mut x = 0f32;
        
        for ch in text.chars() {
            if let Some(glyph) = self.get_glyph(renderer, ch as u16) {
                let advance = glyph.x_advance * self.font_factor;

                x += advance;
            }
        }

        x
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

impl DrawList {
    pub fn add_text(&mut self, renderer: &mut Renderer, font: &mut Font, text: &str, mut pos: float2, color: u32) {
        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(renderer, ch as u16) {
                let texture = font.font_atlas.pages[glyph.page as usize].srv_handle;
                self.set_texture(texture);

                
                let x = (pos.0 + glyph.x).round();
                let y = (-glyph.y).round();
                let w = x + glyph.w;
                let h = y - glyph.h;
                
                self.vertices.push(Vertex::new(float2(x, y), float2(glyph.u,   glyph.v_2), color));
                self.vertices.push(Vertex::new(float2(x, h), float2(glyph.u,   glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, h), float2(glyph.u_2, glyph.v),   color));
                self.vertices.push(Vertex::new(float2(w, y), float2(glyph.u_2, glyph.v_2), color));

                pos.0 += glyph.x_advance * font.font_factor;
                
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
}
