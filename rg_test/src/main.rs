extern crate rg;
extern crate winapi;
extern crate glutin;
extern crate gl;

use glutin::dpi::*;
use glutin::GlContext;

use rg::TextureHandle;
use rg::float2;

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
            common_uniforms: UniformBuffer::new(),
            vertex_buffer: Buffer::empty(gl::ARRAY_BUFFER, gl::DYNAMIC_DRAW, 10000 * mem::size_of::<rg::Vertex>() as isize),
            index_buffer: Buffer::empty(gl::ELEMENT_ARRAY_BUFFER, gl::DYNAMIC_DRAW, 20000 * mem::size_of::<u16>() as isize),
        })
    }
}


impl rg::Renderer for RgOpenGlRenderer {
    fn resize(&mut self, w: f32, h: f32) {}
    fn render(&mut self, list: &rg::DrawList) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        
        let ortho = {
            let l = 0f32;
            let r = WIDTH as f32;
            let b = HEIGHT as f32;
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
        for command in list.commands() {
            unsafe {
                if command.texture_id == 0 as _ {
                    gl::BindTexture(gl::TEXTURE_2D, self.texture.handle as _);
                } else {
                    gl::BindTexture(gl::TEXTURE_2D, command.texture_id as _);
                }
                gl::DrawElements(gl::TRIANGLES, command.index_count as i32, gl::UNSIGNED_SHORT, index_offset as *const _);
            }
            
            index_offset += command.index_count;
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

fn rg_glutin_event(io: &mut rg::IoState, window: &glutin::Window, event: glutin::Event) {
    match event {
        glutin::Event::DeviceEvent { event, .. } => match event {
            glutin::DeviceEvent::MouseMotion { delta } => {
                io.mouse_delta += float2(delta.0 as f32, delta.1 as f32);
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

                io.display_size = float2(new_size.width as f32, new_size.height as f32);
            },
            glutin::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(vk) = input.virtual_keycode {
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
                
                io.mouse = float2(new_pos.x as f32, new_pos.y as f32);
            }
            _ => {}
        }
        _ => {}
    }
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Rusty ImGui")
        .with_dimensions(LogicalSize::new(WIDTH as _, HEIGHT as _));
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Latest)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        //gl::DebugMessageCallback(gl_error_callback, ptr::null_mut());
        //gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::ClearColor(0.05, 0.05, 0.05, 1.0);
        gl::ClearDepth(1.0);
        gl::Viewport(0, 0, WIDTH, HEIGHT);
        gl::Enable(gl::DEPTH_TEST);
        //gl::Enable(gl::FRAMEBUFFER_SRGB);
    }

    
    let mut renderer = RgOpenGlRenderer::new().unwrap();
    let mut cxt = rg::Context::new(Box::new(renderer));

    let mut press_count = 0;
    let mut running = true;
    
    while running {
        cxt.begin_frame();

        {
            let mut io = &mut cxt.io;
            io.clear();
            
            events_loop.poll_events(|event| {
                rg_glutin_event(io, &gl_window, event);
            });
        }

        
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        
        if cxt.begin("Test Window", rg::WindowFlags::Movable | rg::WindowFlags::Closable | rg::WindowFlags::Title) {
            cxt.row(rg::RowType::dynamic(2));
            cxt.column(Some(0.2f32));
            if cxt.button_text("Lots of words and textes!") {
                println!("PRESS 1");
                press_count += 1;
            }
            cxt.column(Some(0.8f32));

            if cxt.button_text("Press me 1!") {
                println!("PRESS 2");
                press_count += 1;
            }
            cxt.row(rg::RowType::dynamic(2));
            cxt.column(Some(0.2f32));

            if cxt.button_text("Press me 2!") {
                println!("PRESS 3");
                press_count += 1;
            }
            /*//cxt.text(&format!("Pressed {} times", press_count));
            //cxt.text(&format!("Pressed {} times", press_count));
            //cxt.text(&format!("Pressed {} times", press_count));
            cxt.text("Molestiae dolorem blanditiis reprehenderit. Consectetur sint corporis saepe accusamus et. Et in qui alias ut ratione optio perferendis necessitatibus. Quae est sit quas eaque laudantium repellendus. Nam at nihil ipsam quas eum. Excepturi doloremque non dolorum sit. Provident tempore blanditiis nesciunt laborum cumque.");
            */
            cxt.end();
        }

        cxt.draw();
        cxt.end_frame();

        gl_window.swap_buffers().unwrap();
    }
}
