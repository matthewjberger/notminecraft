use anyhow::Result;
use gl::types::*;
use image::{EncodableLayout, GenericImageView};
use nalgebra_glm as glm;
use std::{ffi::CString, fs};

#[derive(Default, Debug)]
pub struct BlockConfiguration {
    pub left: i32,
    pub right: i32,
    pub front: i32,
    pub back: i32,
    pub top: i32,
    pub bottom: i32,
}

impl BlockConfiguration {
    pub fn new(left: Tile, right: Tile, front: Tile, back: Tile, top: Tile, bottom: Tile) -> Self {
        Self {
            left: left as _,
            right: right as _,
            front: front as _,
            back: back as _,
            top: top as _,
            bottom: bottom as _,
        }
    }

    pub fn new_single(id: Tile) -> Self {
        let id = id as i32;
        Self {
            left: id,
            right: id,
            front: id,
            back: id,
            top: id,
            bottom: id,
        }
    }

    pub fn new_same_sides(sides: Tile, top: Tile, bottom: Tile) -> Self {
        let sides = sides as i32;
        Self {
            left: sides,
            right: sides,
            front: sides,
            back: sides,
            top: top as _,
            bottom: bottom as _,
        }
    }
}
pub enum Tile {
    Gravel,
    DirtSnowSide,
    Grass,
    DirtGrassSide,
    Cobblestone = 26,
    Bedrock = 32,
    Dirt = 50,
    OakPlanks = 53,
    TntSide = 62,
    TntTop,
    TntBottom,
}

pub enum Block {
    Gravel,
    Grass,
    DirtWithGrass,
    Dirt,
    Cobblestone,
    Tnt,
    Bedrock,
    OakPlanks,
}

impl Block {
    // TODO: Make this generate a dictionary instead
    fn configuration(&self) -> BlockConfiguration {
        match *self {
            Block::Gravel => BlockConfiguration::new_single(Tile::Gravel),
            Block::Grass => BlockConfiguration::new_single(Tile::Grass),
            Block::Dirt => BlockConfiguration::new_single(Tile::Dirt),
            Block::DirtWithGrass => {
                BlockConfiguration::new_same_sides(Tile::DirtGrassSide, Tile::Grass, Tile::Dirt)
            }
            Block::Cobblestone => BlockConfiguration::new_single(Tile::Cobblestone),
            Block::Tnt => {
                BlockConfiguration::new_same_sides(Tile::TntSide, Tile::TntTop, Tile::TntBottom)
            }
            Block::Bedrock => BlockConfiguration::new_single(Tile::Bedrock),
            Block::OakPlanks => BlockConfiguration::new_single(Tile::OakPlanks),
            _ => BlockConfiguration::default(),
        }
    }
}

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
        let atlas_image = image::open("assets/textures/atlas.png")?;

        let mut atlas = 0;
        unsafe {
            gl::GenTextures(1, &mut atlas);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, atlas);

            let dimension = 16;
            let columns = atlas_image.width() / dimension;
            let rows = atlas_image.height() / dimension;
            let number_of_tiles = rows * columns;

            gl::TexImage3D(
                gl::TEXTURE_2D_ARRAY,
                0,
                gl::RGBA as _,
                dimension as _,
                dimension as _,
                number_of_tiles as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null() as *const GLvoid,
            );

            for row in 0..rows {
                let y = row * dimension;
                for column in 0..columns {
                    let x = column * dimension;
                    let pixels = atlas_image.view(x, y, dimension, dimension).to_image();
                    let pixel_bytes = pixels.as_bytes();
                    let tile = (row * columns) + column;
                    gl::TexSubImage3D(
                        gl::TEXTURE_2D_ARRAY,
                        0,
                        0,
                        0,
                        tile as _,
                        dimension as _,
                        dimension as _,
                        1,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        pixel_bytes.as_ptr() as *const GLvoid,
                    );
                }
            }

            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);

            gl::TexParameterf(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as _,
            );
            gl::TexParameterf(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as _,
            );
        }

        Ok(atlas)
    }

    pub unsafe fn draw(&self, block: Block) -> Result<()> {
        let configuration = block.configuration();

        gl::UseProgram(self.shader_program);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.atlas);

        let location = Self::uniform_location(self.shader_program, "mvp")?;
        gl::UniformMatrix4fv(location, 1, gl::FALSE, self.mvp.as_ptr());

        let location = Self::uniform_location(self.shader_program, "blockId")?;

        gl::BindVertexArray(self.vao);

        // back
        gl::Uniform1i(location, configuration.back);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);

        // front
        gl::Uniform1i(location, configuration.front);
        gl::DrawArrays(gl::TRIANGLES, 6, 6);

        // left
        gl::Uniform1i(location, configuration.left);
        gl::DrawArrays(gl::TRIANGLES, 12, 6);

        // right
        gl::Uniform1i(location, configuration.right);
        gl::DrawArrays(gl::TRIANGLES, 18, 6);

        // bottom
        gl::Uniform1i(location, configuration.bottom);
        gl::DrawArrays(gl::TRIANGLES, 24, 6);

        // top
        gl::Uniform1i(location, configuration.top);
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
