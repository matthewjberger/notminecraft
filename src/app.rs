use anyhow::Result;
use gl::types::*;
use glutin::{
    dpi::PhysicalPosition,
    event::{Event, VirtualKeyCode},
    window::Window,
};
use image::{EncodableLayout, GenericImageView};
use nalgebra_glm as glm;
use std::{ffi::CString, fs};

use crate::{
    camera::{CameraDirection, FreeCamera},
    cube::Cube,
    input::Input,
    system::System,
};

pub struct App {
    cube: Cube,
    shader_program: GLuint,
    atlas: GLuint,
    mvp: glm::Mat4,
    angle: f32,
    camera: FreeCamera,
    pub system: System,
    pub input: Input,
}

impl App {
    pub fn new(dimensions: [u32; 2]) -> Result<Self> {
        Ok(Self {
            cube: Cube::new(),
            shader_program: Self::create_shader_program()?,
            atlas: Self::create_atlas()?,
            mvp: glm::Mat4::identity(),
            angle: 0.0,
            camera: FreeCamera::default(),
            system: System::new(dimensions),
            input: Input::default(),
        })
    }

    #[allow(dead_code)]
    pub fn enable_wireframe() {
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        }
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

    fn create_atlas() -> Result<GLuint> {
        let img_ = image::open("assets/textures/atlas.png")?;
        img_.flipv();

        let row = 0;
        let column = 2;
        let dimension = 16;
        let img = img_.view(row, column, dimension, dimension).to_image();

        let mut atlas = 0;
        unsafe {
            gl::GenTextures(1, &mut atlas);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, atlas);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as _,
                img.width() as _,
                img.height() as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                img.as_bytes().as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(atlas);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        Ok(atlas)
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

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.atlas);

            let location = Self::uniform_location(self.shader_program, "mvp")?;
            gl::UniformMatrix4fv(location, 1, gl::FALSE, self.mvp.as_ptr());

            self.cube.draw();
        }
        Ok(())
    }

    fn uniform_location(shader_program: GLuint, name: &str) -> Result<GLint> {
        let name: CString = CString::new(name.as_bytes())?;
        unsafe { Ok(gl::GetUniformLocation(shader_program, name.as_ptr())) }
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

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteTextures(1, &self.atlas);
        }
    }
}
