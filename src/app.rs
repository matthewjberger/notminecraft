use anyhow::Result;
use gl::types::*;
use glutin::{
    dpi::PhysicalPosition,
    event::{Event, VirtualKeyCode},
    window::Window,
};
use nalgebra_glm as glm;
use std::{ffi::CString, fs};

use crate::{
    camera::{CameraDirection, FreeCamera},
    input::Input,
    system::System,
};

#[rustfmt::skip]
pub const VERTICES: &[f32; 24] =
    &[
        // Front
       -0.5, -0.5,  0.5,
        0.5, -0.5,  0.5,
        0.5,  0.5,  0.5,
       -0.5,  0.5,  0.5,
        // Back
       -0.5, -0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5,  0.5, -0.5,
       -0.5,  0.5, -0.5
    ];

#[rustfmt::skip]
pub const INDICES: &[u32; 36] =
    &[
        // Front
        0, 1, 2,
        2, 3, 0,
        // Right
        1, 5, 6,
        6, 2, 1,
        // Back
        7, 6, 5,
        5, 4, 7,
        // Left
        4, 0, 3,
        3, 7, 4,
        // Bottom
        4, 5, 1,
        1, 0, 4,
        // Top
        3, 2, 6,
        6, 7, 3,
    ];

pub struct App {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    shader_program: GLuint,
    mvp: glm::Mat4,
    angle: f32,
    camera: FreeCamera,
    pub system: System,
    pub input: Input,
}

impl App {
    pub fn new(dimensions: [u32; 2]) -> Result<Self> {
        Ok(Self {
            vao: Self::create_vao(),
            vbo: Self::create_vbo(),
            ebo: Self::create_ebo(),
            shader_program: Self::create_shader_program()?,
            mvp: glm::Mat4::identity(),
            angle: 0.0,
            camera: FreeCamera::default(),
            system: System::new(dimensions),
            input: Input::default(),
        })
    }

    fn create_vao() -> GLuint {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }
        vao
    }

    fn create_vbo() -> GLuint {
        let vertices_size = std::mem::size_of::<GLfloat>() * VERTICES.len();
        let vertex_bytes =
            unsafe { std::slice::from_raw_parts(VERTICES.as_ptr() as *const u8, vertices_size) };
        let mut vbo = 0;
        let offset = std::mem::size_of::<GLfloat>() as i32;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo as _);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices_size as GLsizeiptr,
                vertex_bytes.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * offset, 0 as *const GLvoid);
        };
        vbo
    }

    fn create_ebo() -> GLuint {
        let indices_size = std::mem::size_of::<GLuint>() * INDICES.len();
        let index_bytes =
            unsafe { std::slice::from_raw_parts(INDICES.as_ptr() as *const u8, indices_size) };
        let mut ebo = 0;
        unsafe {
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo as _);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices_size as GLsizeiptr,
                index_bytes.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        };
        ebo
    }

    fn create_shader_program() -> Result<GLuint> {
        let vertex_shader = Self::load_shader("assets/shaders/block.vs.glsl", gl::VERTEX_SHADER)?;
        let fragment_shader =
            Self::load_shader("assets/shaders/block.fs.glsl", gl::FRAGMENT_SHADER)?;
        let shaders = [vertex_shader, fragment_shader];
        unsafe {
            let program = gl::CreateProgram();
            for shader in shaders.iter() {
                gl::AttachShader(program, *shader);
            }
            gl::LinkProgram(program);
            for shader in shaders.iter() {
                gl::DeleteShader(*shader);
            }
            Ok(program)
        }
    }

    fn load_shader(path: &str, kind: GLuint) -> Result<GLuint> {
        let shader_source = CString::new(fs::read_to_string(path)?.as_bytes())?;
        unsafe {
            let shader = gl::CreateShader(kind);
            gl::ShaderSource(shader, 1, &shader_source.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
            Self::check_compilation(shader)?;
            Ok(shader)
        }
    }

    fn check_compilation(id: GLuint) -> Result<()> {
        let mut success = gl::FALSE as GLint;

        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == gl::TRUE as GLint {
            return Ok(());
        }

        let mut info_log_length = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut info_log_length);
        }

        let mut info_log = vec![0; info_log_length as usize];
        unsafe {
            gl::GetShaderInfoLog(
                id,
                info_log_length,
                std::ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
        }
        eprintln!(
            "ERROR: Shader compilation failed.\n{}\n",
            std::str::from_utf8(&info_log)?
        );

        Ok(())
    }

    pub fn update(&mut self, window: &Window) -> Result<()> {
        if self.input.is_key_pressed(VirtualKeyCode::Escape) {
            self.system.exit_requested = true;
        }

        self.update_free_camera(window)?;

        self.angle += 10.0 * self.system.delta_time as f32;
        let perspective = glm::perspective_zo(
            self.system.aspect_ratio(),
            80_f32.to_radians(),
            0.01,
            1000.0,
        );
        let model = glm::rotate(
            &glm::Mat4::identity(),
            self.angle.to_radians(),
            &glm::Vec3::y(),
        );
        self.mvp = perspective * self.camera.view_matrix() * model;
        Ok(())
    }

    pub fn handle_events(&mut self, event: &Event<()>) -> Result<()> {
        self.system.handle_event(event);
        self.input.handle_event(event, self.system.window_center());
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CCW);

            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            let background_color: &[GLfloat; 4] = &[0.25, 0.25, 0.25, 1.0];
            gl::ClearBufferfv(gl::COLOR, 0, background_color as *const f32);
            gl::ClearBufferfv(gl::DEPTH, 0, &[1.0 as GLfloat] as *const f32);

            gl::UseProgram(self.shader_program);

            let location = Self::uniform_location(self.shader_program, "mvp")?;
            gl::UniformMatrix4fv(location, 1, gl::FALSE, self.mvp.as_ptr());

            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                INDICES.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
        Ok(())
    }

    fn uniform_location(shader_program: GLuint, name: &str) -> Result<GLint> {
        let name: CString = CString::new(name.as_bytes())?;
        unsafe { Ok(gl::GetUniformLocation(shader_program, name.as_ptr())) }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo as *const u32);
            gl::DeleteBuffers(1, &self.ebo as *const u32);
            gl::DeleteProgram(self.shader_program);
        }
    }

    fn update_free_camera(&mut self, window: &Window) -> Result<()> {
        let delta_time = self.system.delta_time as f32;
        if self.input.is_key_pressed(VirtualKeyCode::W) {
            self.camera.translate(CameraDirection::Forward, delta_time);
        }
        if self.input.is_key_pressed(VirtualKeyCode::A) {
            self.camera.translate(CameraDirection::Left, delta_time);
        }
        if self.input.is_key_pressed(VirtualKeyCode::S) {
            self.camera.translate(CameraDirection::Backward, delta_time);
        }
        if self.input.is_key_pressed(VirtualKeyCode::D) {
            self.camera.translate(CameraDirection::Right, delta_time);
        }
        if self.input.is_key_pressed(VirtualKeyCode::LShift) {
            self.camera.translate(CameraDirection::Down, delta_time);
        }
        if self.input.is_key_pressed(VirtualKeyCode::Space) {
            self.camera.translate(CameraDirection::Up, delta_time);
        }
        let offset = self.input.mouse.offset_from_center;
        self.camera.process_mouse_movement(offset.x, offset.y);

        window.set_cursor_grab(true)?;
        window.set_cursor_visible(false);
        let center = self.system.window_center();
        window.set_cursor_position(PhysicalPosition::new(center.x, center.y))?;

        Ok(())
    }
}
