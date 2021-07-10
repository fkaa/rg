use sdl2::{
    event::{WindowEvent, Event},
    keyboard::Keycode,
    mouse::MouseButton,
    video::{
        GLProfile,
        SwapInterval,
    }
};


use rg::TextureHandle;
use rg::float2;
use rg::Rect;

use std::slice;
use std::ptr;
use std::mem;
use std::ffi::CString;

pub struct InputElement {
    pub shader_slot: u32,
    pub buffer_slot: u32,
    pub format: u32,
    pub components: u32,
    pub normalized: u8,
    pub stride: u32,
    pub offset: u32,
}

// TODO: 
pub struct InputLayout {
    pub elements: Vec<InputElement>,
    pub vao: u32,
}

impl InputLayout {
    pub fn new(elements: Vec<InputElement>) -> Self {
        let mut vao = 0;
        
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }
        
        InputLayout {
            elements,
            vao,
        }
    }

    pub fn bind(&mut self, buffers: &[(u32, &Buffer)], index_buffer: Option<&Buffer>) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
        
        for &(slot, ref buffer) in buffers {
            unsafe {
                gl::BindBuffer(buffer.ty(), buffer.handle());
            }
            
            for attr in self.elements.iter().filter(|a| a.buffer_slot == slot) {
                unsafe {
                    gl::EnableVertexAttribArray(attr.shader_slot);
                    gl::VertexAttribPointer(
                        attr.shader_slot,
                        attr.components as i32,
                        attr.format,
                        attr.normalized,
                        attr.stride as i32,
                        attr.offset as *const _
                    );
                }
            }
        }

        if let Some(buf) = index_buffer {
            unsafe {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buf.handle());
            }
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub handle: u32,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

impl Texture {
    pub fn with_data_2d(
        data: &[u8],
        width: u32,
        height: u32,
        internal_format: u32,
        format: u32,
        filter: u32
    ) -> Self {
        let mut handle = 0;
        
        unsafe {
            gl::GenTextures(1, &mut handle);
            
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format as i32, width as i32, height as i32, 0, format as u32, gl::UNSIGNED_BYTE, data.as_ptr() as *const _);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        }

        Texture {
            handle
        }
    }
}

#[derive(Debug)]
pub enum VariableBinding {
    Attribute(String, u32),
    Uniform(String, u32),
    UniformBlock(String, u32),
    Sampler(String, u32),
}

#[derive(Debug)]
pub struct Shader {
    pub program: u32,
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

impl Shader {
    pub fn new(
        vs: &str,
        ps: Option<&str>,
        bindings: Vec<VariableBinding>
    ) -> Result<Self, String> {
        unsafe fn compile_shader(ty: u32, shdr: &str) -> Result<u32, String> {
            let shader = gl::CreateShader(ty);
            let len = shdr.len() as i32;
            let shdr = shdr.as_ptr() as *const i8;
            gl::ShaderSource(shader, 1, &shdr, &len);
            gl::CompileShader(shader);

            let mut success = 0i32;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success as _);

            if success == gl::FALSE as i32 {
                let mut log_size = 0i32;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_size as _);

                let mut log = vec![0u8; log_size as usize];
                gl::GetShaderInfoLog(shader, log_size, ptr::null_mut(), log.as_mut_ptr() as _);

                gl::DeleteShader(shader);
                Err(String::from_utf8_unchecked(log))
            } else {
                Ok(shader)
            }
        }
        
        let vs = unsafe { compile_shader(gl::VERTEX_SHADER, vs)? };
        let ps = if let Some(ps) = ps {
            Some(unsafe { compile_shader(gl::FRAGMENT_SHADER, ps)? })
        } else {
            None
        };

        unsafe {
            let program = gl::CreateProgram();
            
            gl::AttachShader(program, vs);
            if let Some(ps) = ps {
                gl::AttachShader(program, ps);
            }

            for bind in &bindings {
                match bind {
                    &VariableBinding::Attribute(ref name, id) => {
                        let c_str = CString::new(name.clone()).unwrap();
                        gl::BindAttribLocation(program, id, c_str.to_bytes_with_nul().as_ptr() as *const _);     
                    },
                    _ => {}
                }
            }

            gl::LinkProgram(program);

            let mut success = 0i32;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == gl::FALSE as i32 {
                let mut log_size = 0i32;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_size as _);

                let mut log = vec![0u8; log_size as usize];
                gl::GetProgramInfoLog(program, log_size, ptr::null_mut(), log.as_mut_ptr() as _);

                gl::DeleteProgram(program);
                return Err(String::from_utf8_unchecked(log));
            }

            gl::DetachShader(program, vs);
            if let Some(ps) = ps {
                gl::DetachShader(program, ps);
            }

            gl::UseProgram(program);

            // after linking we setup sampler bindings as specified in the shader
            for bind in bindings {
                match bind {
                    VariableBinding::Uniform(name, id) => {
                        // TODO: impl for block?
                    },
                    VariableBinding::UniformBlock(name, id) => {
                        let c_str = CString::new(name).unwrap();
                        let index = gl::GetUniformBlockIndex(program, c_str.to_bytes_with_nul().as_ptr() as *const _);

                        gl::UniformBlockBinding(program, index, id);
                    }
                    VariableBinding::Sampler(name, id) => {
                        let c_str = CString::new(name).unwrap();
                        let index = gl::GetUniformLocation(program, c_str.to_bytes_with_nul().as_ptr() as *const _);
                        
                        gl::Uniform1i(index, id as i32);
                    },
                    _ => {}
                }
            }

            Ok(Shader {
                program
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program);
        }
    }

    pub fn bind_uniform_block<T>(
        &self,
        idx: u32,
        buffer: &UniformBuffer<T>
    ) {
        self.bind();
        
        unsafe {
            gl::BindBufferBase(
                gl::UNIFORM_BUFFER,
                idx,
                buffer.handle()
            );
        }
    }

    pub fn bind_texture(&self, index: u32, texture: &Texture) {
        self.bind();
        
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + index);
            gl::BindTexture(gl::TEXTURE_2D, texture.handle);
        }
    }
}

// TODO: add size field to buffers to check for overflow on writes..
#[derive(Debug)]
pub struct Buffer {
    pub handle: u32,
    pub ty: u32,
    pub size: usize,
}

impl Buffer {
    fn empty(
        ty: u32,
        usage: u32,
        size: isize
    ) -> Buffer {
        let mut buffer = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(ty, buffer);
            gl::BufferData(ty, size, ptr::null_mut(), usage);
        }

        Buffer {
            handle: buffer,
            ty,
            size: size as usize,
        }
    }
    
    fn with_data(
        ty: u32,
        usage: u32,
        data: &[u8]
    ) -> Buffer {
        let mut buffer = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(ty, buffer);
            gl::BufferData(ty, data.len() as isize, data.as_ptr() as _, usage);
        }

        Buffer {
            handle: buffer,
            ty,
            size: data.len()
        }
    }

    pub fn ty(&self) -> u32 {
        self.ty
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn write(&self, data: &[u8]) {
        unsafe {
            gl::BindBuffer(self.ty, self.handle);
            gl::BufferSubData(self.ty, 0, data.len() as isize, data.as_ptr() as *const _ as *const _);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

pub struct UniformBuffer<T> {
    pub buffer: Buffer,
    _phantom: ::std::marker::PhantomData<T>,
}

impl<T> UniformBuffer<T> {
    pub fn new() -> Self {
        UniformBuffer {
            buffer: Buffer::empty(gl::UNIFORM_BUFFER, gl::DYNAMIC_DRAW, mem::size_of::<T>() as isize),
            _phantom: ::std::marker::PhantomData,
        }
    }

    pub fn write(&self, value: &T) {
        let buffer = &self.buffer;
        
        unsafe {
            gl::BindBuffer(buffer.ty(), buffer.handle());
            gl::BufferSubData(buffer.ty(), 0, mem::size_of::<T>() as isize, value as *const _ as *const _);
        }
    }

    pub fn handle(&self) -> u32 {
        self.buffer.handle
    }
}

struct CommonUniforms {
    projection: [f32; 16],
}

struct Vertex {
    pos: [f32; 2],
    
}

struct RgOpenGlRenderer {
    texture: Texture,
    layout: InputLayout,
    shader: Shader,
    width: f32,
    height: f32,

    common_uniforms: UniformBuffer<CommonUniforms>,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl RgOpenGlRenderer {
    pub fn new() -> Result<Self, String> {
        let texture = Texture::with_data_2d(
            &[0xff, 0xff, 0xff, 0xff],
            1,
            1,
            gl::RGBA8,
            gl::RGBA,
            gl::NEAREST,
        );

        let layout = InputLayout::new(vec![
            InputElement { shader_slot: 0, buffer_slot: 0, format: gl::FLOAT, normalized: gl::FALSE, components: 2, stride: 20, offset: 0 },
            InputElement { shader_slot: 1, buffer_slot: 0, format: gl::FLOAT, normalized: gl::FALSE, components: 2, stride: 20, offset: 8 },
            InputElement { shader_slot: 2, buffer_slot: 0, format: gl::UNSIGNED_BYTE, normalized: gl::TRUE, components: 4, stride: 20, offset: 16 },
        ]);

        let shader = Shader::new(
            include_str!("../shaders/main.vs"),
            Some(include_str!("../shaders/main.fs")),
            vec![
                VariableBinding::Attribute(String::from("Position"), 0),
                VariableBinding::Attribute(String::from("Uv"), 1),
                VariableBinding::Attribute(String::from("Color"), 2),
                VariableBinding::UniformBlock(String::from("Common"), 0),
            ]
        )?;
        
        Ok(RgOpenGlRenderer {
            texture,
            layout,
            shader,
            width: WIDTH as f32,
            height: HEIGHT as f32,
            common_uniforms: UniformBuffer::new(),
            vertex_buffer: Buffer::empty(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, 10000 * mem::size_of::<rg::Vertex>() as isize),
            index_buffer: Buffer::empty(gl::ELEMENT_ARRAY_BUFFER, gl::DYNAMIC_DRAW, 20000 * mem::size_of::<u16>() as isize),
        })
    }
}


impl rg::Renderer for RgOpenGlRenderer {
    fn resize(&mut self, w: f32, h: f32) {
        self.width = w;
        self.height = h;
    }
    
    fn render(&mut self, list: &rg::DrawList) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Viewport(0, 0, self.width as _, self.height as _);
        }
        
        let ortho = {
            let l = 0f32;
            let r = self.width as f32;
            let b = self.height as f32;
            let t = 0f32;

            [
                2f32 / (r - l),    0f32,              0f32,   0f32,
                0f32,              2f32 / (t - b),    0f32,   0f32,
                0f32,              0f32,              0.5f32, 0f32,
                (r + l) / (l - r), (t + b) / (b - t), 0.5f32, 1f32,
            ]
        };
        self.common_uniforms.write(&CommonUniforms {
            projection: ortho
        });

        let vertices = unsafe {
            slice::from_raw_parts(
                list.vertices.as_ptr() as *const u8,
                list.vertices.len() * mem::size_of::<rg::Vertex>(),
            )
        };
        self.vertex_buffer.write(vertices);

        let indices = unsafe {
            slice::from_raw_parts(
                list.indices.as_ptr() as *const u8,
                list.indices.len() * mem::size_of::<u16>(),
            )
        };
        self.index_buffer.write(indices);

        self.layout.bind(&[
            (0, &self.vertex_buffer),
        ], Some(&self.index_buffer));
        
        self.shader.bind();
        self.shader.bind_uniform_block(0, &self.common_uniforms);

        let mut index_offset = 0;
        
        for layer in list.commands() {
            for command in layer {
                unsafe {
                    if command.texture_id == 0 as _ {
                        gl::BindTexture(gl::TEXTURE_2D, self.texture.handle as _);
                    } else {
                        gl::BindTexture(gl::TEXTURE_2D, command.texture_id as _);
                    }
                    
                    gl::DrawElements(gl::TRIANGLES, command.index_count as i32, gl::UNSIGNED_SHORT, command.index_offset as *const _);
                }
            }
            
        }
    }

    
    fn create_texture_a8(&mut self, width: u32, height: u32) -> (TextureHandle, TextureHandle) {
        let zeroed = vec![0u8; (width * height) as usize];
        let texture = Texture::with_data_2d(
            &zeroed,
            width,
            height,
            gl::R8,
            gl::RED,
            gl::LINEAR
        );

        let handle = texture.handle;

        ::std::mem::forget(texture);

        (handle as TextureHandle, handle as TextureHandle)
    }
    
    fn upload_a8(&mut self, handle: TextureHandle, x: u32, y: u32, width: u32, height: u32, data: &[u8], stride: u32) {
        let handle = handle as u32;

        unsafe {
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, stride as i32);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x as i32, y as i32,
                width as i32, height as i32,
                gl::RED,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _
            );
        }
        /*unsafe {
            (*self.context).UpdateSubresource(
                handle as *mut _,
                0,
                &D3D11_BOX {
                    left: x,
                    top: y,
                    front: 0,
                    right: x + width,
                    bottom: y + height,
                    back: 1,
                },
                data.as_ptr() as *const _,
                stride,
                0
            );
        }*/
    }
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn handle_event(io: &mut rg::IoState, renderer: &mut rg::Renderer, event: &Event) {
        match event {
            Event::MouseMotion {
                x,
                y,
                ..
            } => {
                let new_pos = float2(*x as _, *y as _);
                let old = io.mouse;
                io.mouse = new_pos;

                io.mouse_delta += io.mouse - old;
            },
            Event::MouseButtonDown {
                mouse_btn,
                x,
                y,
                ..
            } => {
                let idx = *mouse_btn as usize;

                if !io.mouse_down[idx] {
                    io.mouse_pressed[idx] = true;
                    io.mouse_clicked_pos[idx] = io.mouse;
                }

                io.mouse_down[idx] = true;
            },
            Event::MouseButtonUp {
                mouse_btn,
                x,
                y,
                ..
            } => {
                let idx = *mouse_btn as usize;

                io.mouse_down[idx] = false;
                io.mouse_released[idx] = true;
            },
            Event::Window { win_event, .. } => {
                match win_event {
                    WindowEvent::Resized(w, h) => {
                        let size = float2(*w as f32, *h as f32);
                        
                        io.display_size = size;
                        renderer.resize(size.0, size.1);
                    },
                    _ => {}
                }
            },
            /*Event::KeyDown {
                keycode,
                scancode,
                keymod: _,
                ..
            } => {
                if let Some(scan) = scancode {
                    let idx = *scan as usize;

                    if !self.scan_down[idx] {
                        self.scan_press.set(idx, true);
                    }

                    self.scan_down.set(idx, true);
                }

                if let Some(key) = keycode {
                    let idx = (*key as usize) & !(1 << 30);

                    if !self.key_down[idx] {
                        self.key_press.set(idx, true);
                    }

                    self.key_down.set(idx, true);
                }
            }
            Event::KeyUp {
                keycode,
                scancode,
                keymod: _,
                ..
            } => {
                if let Some(scan) = scancode {
                    let idx = *scan as usize;

                    self.scan_down.set(idx, false);
                    self.scan_up.set(idx, true);
                }

                if let Some(key) = keycode {
                    let idx = (*key as usize) & !(1 << 30);

                    self.key_down.set(idx, false);
                    self.key_up.set(idx, true);
                }
            }*/
            _ => {}
        }



    /*match event {
        glutin::Event::DeviceEvent { event, .. } => match event {
            glutin::DeviceEvent::MouseMotion { delta } => {
                //io.mouse_delta += float2(delta.0 as f32, delta.1 as f32);
            }
            glutin::DeviceEvent::MouseWheel { delta } => {
                match delta {
                    glutin::MouseScrollDelta::LineDelta(x, y) => {
                        io.mouse_scroll = float2(x as f32, y as f32);
                    }
                    glutin::MouseScrollDelta::PixelDelta(pos) => {
                        let dpi_factor = window.get_hidpi_factor();
                        let new_size = pos.to_physical(dpi_factor);

                        io.mouse_scroll = float2(new_size.x as f32, new_size.y as f32);
                    }
                }
            }
            _ => {}
        }
        glutin::Event::WindowEvent{ event, window_id } => match event {
            //glutin::WindowEvent::CloseRequested => *finished = true,
            glutin::WindowEvent::Resized(logical_size) => {
                let dpi_factor = window.get_hidpi_factor();
                let new_size = logical_size.to_physical(dpi_factor);
                let size = float2(new_size.width as f32, new_size.height as f32);
                
                io.display_size = size;
                renderer.resize(size.0, size.1);
            },
            glutin::WindowEvent::ReceivedCharacter(ch) => {
                if !ch.is_control() {
                    io.text_edit_actions.push(rg::TextEditAction::Char(ch));
                }
            }
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(vk) = input.virtual_keycode {
                    if input.state == glutin::ElementState::Pressed {
                        let shift = input.modifiers.shift;
                        let key = match vk {
                            glutin::VirtualKeyCode::Left => Some(rg::TextEditKey::Left(shift)),
                            glutin::VirtualKeyCode::Right => Some(rg::TextEditKey::Right(shift)),
                            glutin::VirtualKeyCode::Back => Some(rg::TextEditKey::Backspace),
                            glutin::VirtualKeyCode::Delete => Some(rg::TextEditKey::Delete),
                            _ => None
                        };
                        if let Some(key) = key {
                            io.text_edit_actions.push(rg::TextEditAction::Key(key));
                        }
                    }
                    
                    let idx = vk as usize;
                    if input.state == glutin::ElementState::Released {
                        io.down[idx] = false;
                    } else {
                        if !io.down[idx] {
                            io.pressed[idx] = true;
                        }
                        io.down[idx] = true;
                    }
                }
            }
            glutin::WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers
            } => {
                let idx =  match button {
                    glutin::MouseButton::Left => 0,
                    glutin::MouseButton::Right => 1,
                    _ => 4,
                };
                
                if state == glutin::ElementState::Released {
                    io.mouse_down[idx] = false;
                    io.mouse_released[idx] = true;
                } else {
                    if !io.mouse_down[idx] {
                        io.mouse_pressed[idx] = true;
                        io.mouse_clicked_pos[idx] = io.mouse;
                    }
                    io.mouse_down[idx] = true;
                }
            },
            glutin::WindowEvent::CursorMoved { position, .. } => {
                let dpi_factor = window.get_hidpi_factor();
                let new_pos = position.to_physical(dpi_factor);

                let old = io.mouse;
                io.mouse = float2(new_pos.x as f32, new_pos.y as f32);

                io.mouse_delta += io.mouse - old;
            }
            _ => {}
        }
        _ => {}
    }*/
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_context_version(3, 3);
    video_subsystem
        .gl_attr()
        .set_context_profile(GLProfile::Core);

    let w = 800;
    let h = 600;

    let window = video_subsystem
        .window("window", w, h)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let cxt = window.gl_create_context().unwrap();

    window.gl_make_current(&cxt).unwrap();

    video_subsystem.gl_set_swap_interval(SwapInterval::VSync).unwrap();

    unsafe {
        gl::load_with(|symbol| video_subsystem.gl_get_proc_address(symbol) as _);
        //gl::DebugMessageCallback(gl_error_callback, ptr::null_mut());
        //gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::ClearColor(0.25, 0.25, 0.25, 1.0);
        gl::ClearDepth(1.0);
        gl::Viewport(0, 0, WIDTH, HEIGHT);
        gl::Enable(gl::DEPTH_TEST);
        //gl::Enable(gl::FRAMEBUFFER_SRGB);
    }

    
    let mut renderer = RgOpenGlRenderer::new().unwrap();
    let mut cxt = rg::Context::new(Box::new(renderer));


    use rg::Renderer;

    let size = float2(w as _, h as _);
    cxt.io.display_size = size;
    cxt.renderer.resize(size.0, size.1);

    let mut press_count = 0;
    let mut running = true;

    let mut text = String::from("Here is some sample text for text field");

    let mut deltas = [0f32; 60];
    let mut frame = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();

    
    while running {
        cxt.begin_frame();

        deltas[frame % 60] = cxt.io.delta;
        frame += 1;
        let avg = (deltas.iter().sum::<f32>() / 60f32) * 1000f32;
        let min = deltas.iter().min_by(|a,b|a.partial_cmp(b).unwrap()).unwrap() * 1000f32;
        let max = deltas.iter().max_by(|a,b|a.partial_cmp(b).unwrap()).unwrap() * 1000f32;
        
        //gl_window.set_title(&format!("UI - avg={:.3}ms,min={:.3}ms,max={:.3}ms", avg, min, max));
        {
            let mut io = &mut cxt.io;

            /*if let Some(cursor) = io.cursor {
                let glutin_cursor = match cursor {
                    rg::CursorType::Default => glutin::MouseCursor::Default,
                    rg::CursorType::Caret => glutin::MouseCursor::Text,
                    rg::CursorType::ResizeHorizontal => glutin::MouseCursor::EResize,
                    rg::CursorType::ResizeVertical => glutin::MouseCursor::NResize,
                    rg::CursorType::ResizeNE => glutin::MouseCursor::NeResize,
                    rg::CursorType::ResizeNW => glutin::MouseCursor::NwResize,
                };

                gl_window.set_cursor(glutin_cursor);
            }*/

            io.clear();
            let renderer = &mut *cxt.renderer;

            for event in event_pump.poll_iter() {
                handle_event(&mut io, renderer, &event);

                if let Event::Quit { .. } = event {
                    running = false;
                    break;
                }
            }
        }

        
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if cxt.begin_root("Root", rg::WindowFlags::NoBg) {
            /*if cxt.begin_menu_bar() {
            cxt.end_menu_bar();
        }*/
            let height = cxt.available_height();
            cxt.row(rg::RowType::dynamic_ex(2, height));
            cxt.column(Some(0.35f32));
            if cxt.begin_panel("Left", rg::PanelFlags::None) {
                if cxt.begin_tab_bar("tabbar") {
                    if cxt.begin_tab_item("Test Tab") {
                        cxt.row(rg::RowType::dynamic(1));
                        cxt.column(Some(1f32));
                        cxt.paragraph("Inside Tab 1!");

                        cxt.row(rg::RowType::dynamic(3));
                        cxt.column(Some(1.0 / 3.0));
                        cxt.button_text("hello");
                        cxt.column(Some(1.0 / 3.0));
                        cxt.button_text("abc");
                        cxt.column(Some(1.0 / 3.0));
                        cxt.button_text("iasd asd");


                        cxt.row(rg::RowType::dynamic(2));
                        {
                            cxt.column(Some(0.8f32));
                            cxt.paragraph("Molestiae dolorem blanditiis reprehenderit. Consectetur sint corporis saepe accusamus et. Et in qui alias ut ratione optio perferendis necessitatibus. Quae est sit quas eaque laudantium repellendus. Nam at nihil ipsam quas eum. Excepturi doloremque non dolorum sit. Provident tempore blanditiis nesciunt laborum cumque.");
                            cxt.column(Some(0.2f32));
                            cxt.paragraph("Much less text than the one to the left.");
                        }
                        
                        cxt.end_tab_item();
                    }
                    if cxt.begin_tab_item("Tab Two") {
                        cxt.row(rg::RowType::dynamic(1));
                        cxt.column(Some(1f32));
                        cxt.paragraph("Inside Tab 2!");
                        cxt.end_tab_item();
                    }
                    if cxt.begin_tab_item("Another Tab") {
                        cxt.row(rg::RowType::dynamic(1));
                        cxt.column(Some(1f32));
                        cxt.paragraph("Inside Tab 3!");
                        
                        cxt.end_tab_item();
                    }
                    cxt.end_tab_bar();
                }
                cxt.end_panel();
            }
            
            cxt.column(Some(0.65f32));
            if cxt.begin_panel("Right", rg::PanelFlags::Styled) {
                cxt.row(rg::RowType::dynamic(2));
                {
                    cxt.column(Some(0.8f32));
                    cxt.paragraph("Molestiae dolorem blanditiis reprehenderit. Consectetur sint corporis saepe accusamus et. Et in qui alias ut ratione optio perferendis necessitatibus. Quae est sit quas eaque laudantium repellendus. Nam at nihil ipsam quas eum. Excepturi doloremque non dolorum sit. Provident tempore blanditiis nesciunt laborum cumque.");
                    cxt.column(Some(0.2f32));
                    cxt.paragraph("Much less text than the one to the left.");
                }
                cxt.row(rg::RowType::dynamic(2));
                cxt.column(Some(0.3f32));
                cxt.textfield("textfield", &mut text);
                /*cxt.row(rg::RowType::dynamic(2));
                cxt.column(Some(1f32));
                cxt.paragraph(&text);
                cxt.row(rg::RowType::dynamic(2));
                cxt.column(Some(0.3f32));
                cxt.textfield("textfield2", &mut text);*/
                cxt.end_panel();
            }
            
            cxt.end();
        }


        
        cxt.draw();
        cxt.end_frame();

        if cxt.io.is_key_down(0) {
            {
                let list = &mut cxt.draw_list;
                list.clear();
                
                let dbg_area = Rect::new(float2(20f32, 20f32), float2(700f32, 300f32));
                let dbg_inner = dbg_area.pad(5f32);
                let dbg_w = dbg_inner.width();
                let max_y = 60f32;

                let bot_left = float2(dbg_inner.min.0, dbg_inner.max.1);
                
                list.add_rect_filled(dbg_area.min, dbg_area.max, 0f32, 0xbb000000);
                list.add_rect_filled(dbg_inner.min, dbg_inner.max, 0f32, 0x22ffffff);
                let y = bot_left.1 - (16.6f32 / max_y) * dbg_inner.height();
                list.add_line(float2(bot_left.0, y), float2(dbg_inner.max.0, y), 0x55ffffff);
                let mut path = list.path();

                for (i, &d) in deltas.iter().enumerate() {
                    let x = bot_left.0 + (i as f32 / 59f32 * dbg_w);
                    let y = bot_left.1 - ((d * 1000f32) / max_y) * dbg_inner.height();
                    
                    let point = float2(x, y);

                    path = path.line(point);
                }
                
                path.stroke(1f32, false, 0xffffffff);
            }
            let list = &mut cxt.draw_list;
                list.push_layer(10);

            cxt.renderer.render(list);

            list.clear();
        }

        window.gl_swap_window();
    }
}
