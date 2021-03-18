use anyhow::Result;
use gl::types::*;
use image::{EncodableLayout, GenericImageView};
use nalgebra_glm as glm;
use std::{ffi::CString, fs};

#[rustfmt::skip]
pub const VERTICES: &[f32; 180] =
    &[
        // back
       -0.5, -0.5, -0.5,  0.0, 0.0,
        0.5, -0.5, -0.5,  1.0, 0.0,
        0.5,  0.5, -0.5,  1.0, 1.0,
        0.5,  0.5, -0.5,  1.0, 1.0,
       -0.5,  0.5, -0.5,  0.0, 1.0,
       -0.5, -0.5, -0.5,  0.0, 0.0,

       // front
       -0.5, -0.5,  0.5,  0.0, 0.0,
        0.5, -0.5,  0.5,  1.0, 0.0,
        0.5,  0.5,  0.5,  1.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 1.0,
       -0.5,  0.5,  0.5,  0.0, 1.0,
       -0.5, -0.5,  0.5,  0.0, 0.0,

       // left
       -0.5,  0.5,  0.5,  1.0, 1.0, // upper right
       -0.5,  0.5, -0.5,  0.0, 1.0, // upper left
       -0.5, -0.5, -0.5,  0.0, 0.0, // lower left
       -0.5, -0.5, -0.5,  0.0, 0.0, // lower left
       -0.5, -0.5,  0.5,  1.0, 0.0, // lower right
       -0.5,  0.5,  0.5,  1.0, 1.0, // upper right

        // right
        0.5,  0.5,  0.5,  0.0, 1.0, // upper left
        0.5,  0.5, -0.5,  1.0, 1.0, // upper right
        0.5, -0.5, -0.5,  1.0, 0.0, // lower right
        0.5, -0.5, -0.5,  1.0, 0.0, // lower right
        0.5, -0.5,  0.5,  0.0, 0.0, // lower left
        0.5,  0.5,  0.5,  0.0, 1.0, // upper left

        // bottom
       -0.5, -0.5, -0.5,  0.0, 1.0, // upper left
        0.5, -0.5, -0.5,  1.0, 1.0, // upper right
        0.5, -0.5,  0.5,  1.0, 0.0, // lower right
        0.5, -0.5,  0.5,  1.0, 0.0, // lower right
       -0.5, -0.5,  0.5,  0.0, 0.0, // lower left
       -0.5, -0.5, -0.5,  0.0, 1.0, // upper left

        // top
       -0.5,  0.5, -0.5,  0.0, 1.0,
        0.5,  0.5, -0.5,  1.0, 1.0,
        0.5,  0.5,  0.5,  1.0, 0.0,
        0.5,  0.5,  0.5,  1.0, 0.0,
       -0.5,  0.5,  0.5,  0.0, 0.0,
       -0.5,  0.5, -0.5,  0.0, 1.0
    ];

pub struct Cube {
    vao: GLuint,
    vbo: GLuint,
    shader_program: GLuint,
    atlas: GLuint,
    pub mvp: glm::Mat4,
}

impl Cube {
    pub fn new() -> Result<Self> {
        Ok(Self {
            vao: Self::create_vao(),
            vbo: Self::create_vbo(),
            shader_program: Self::create_shader_program()?,
            atlas: Self::create_atlas()?,
            mvp: glm::Mat4::identity(),
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
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 5 * offset, 0 as *const GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                5 * offset,
                (3 * offset) as *const GLvoid,
            );
        };
        vbo
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

        let row = 0;
        let column = 22;
        let dimension = 16;
        let img = img_
            .view(column * dimension, row * dimension, dimension, dimension)
            .to_image();

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

    pub unsafe fn draw(&self) -> Result<()> {
        gl::UseProgram(self.shader_program);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, self.atlas);

        let location = Self::uniform_location(self.shader_program, "mvp")?;
        gl::UniformMatrix4fv(location, 1, gl::FALSE, self.mvp.as_ptr());

        gl::BindVertexArray(self.vao);

        // back
        gl::DrawArrays(gl::TRIANGLES, 0, 6);

        // front
        gl::DrawArrays(gl::TRIANGLES, 6, 6);

        // left
        gl::DrawArrays(gl::TRIANGLES, 12, 6);

        // right
        gl::DrawArrays(gl::TRIANGLES, 18, 6);

        // bottom
        gl::DrawArrays(gl::TRIANGLES, 24, 6);

        // top
        gl::DrawArrays(gl::TRIANGLES, 30, 6);

        Ok(())
    }

    fn uniform_location(shader_program: GLuint, name: &str) -> Result<GLint> {
        let name: CString = CString::new(name.as_bytes())?;
        unsafe { Ok(gl::GetUniformLocation(shader_program, name.as_ptr())) }
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteProgram(self.shader_program);
            gl::DeleteTextures(1, &self.atlas);
        }
    }
}
