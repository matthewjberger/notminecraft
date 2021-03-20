use anyhow::Result;
use gl::types::*;
use image::{EncodableLayout, GenericImageView};
use nalgebra_glm as glm;
use std::{ffi::CString, fs};

const CHUNK_WIDTH: usize = 4;
const CHUNK_LENGTH: usize = 4;
const CHUNK_DEPTH: usize = 8;
const WORLD_WIDTH: usize = 4;
const WORLD_LENGTH: usize = 4;

pub struct World {
    pub chunks: Vec<Vec<Chunk>>,
}

impl World {
    pub fn new() -> Self {
        let mut chunks = Vec::new();
        for y in 0..WORLD_LENGTH {
            let mut chunks_x = Vec::new();
            for x in 0..WORLD_WIDTH {
                let mut chunk = Chunk::default();
                chunk.position = glm::vec3((x * CHUNK_WIDTH) as f32, (y * CHUNK_LENGTH) as _, 0.0);
                chunks_x.push(chunk);
            }
            chunks.push(chunks_x);
        }
        Self { chunks }
    }
}

pub struct Chunk {
    pub position: glm::Vec3,
    pub blocks: [[[Block; CHUNK_DEPTH]; CHUNK_LENGTH]; CHUNK_WIDTH],
}

impl Default for Chunk {
    fn default() -> Self {
        let blocks = [[[Block::default(); CHUNK_DEPTH]; CHUNK_LENGTH]; CHUNK_WIDTH];
        Self {
            position: glm::vec3(0.0, 0.0, 0.0),
            blocks,
        }
    }
}

#[derive(Default)]
pub struct BlockConfiguration {
    pub left: i32,
    pub right: i32,
    pub front: i32,
    pub back: i32,
    pub top: i32,
    pub bottom: i32,
    pub is_entity: bool,
    pub is_solid: bool,
}

impl BlockConfiguration {
    pub fn empty() -> Self {
        Self {
            left: Tile::Air as _,
            right: Tile::Air as _,
            front: Tile::Air as _,
            back: Tile::Air as _,
            top: Tile::Air as _,
            bottom: Tile::Air as _,
            is_entity: false,
            is_solid: false,
        }
    }

    pub fn new(left: Tile, right: Tile, front: Tile, back: Tile, top: Tile, bottom: Tile) -> Self {
        Self {
            left: left as _,
            right: right as _,
            front: front as _,
            back: back as _,
            top: top as _,
            bottom: bottom as _,
            is_entity: false,
            is_solid: true,
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
            is_entity: false,
            is_solid: true,
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
            is_entity: false,
            is_solid: true,
        }
    }

    pub fn new_entity(tile: Tile) -> Self {
        let mut config = Self::default();
        config.front = tile as _;
        config.is_entity = true;
        config.is_solid = false;
        config
    }
}
pub enum Tile {
    Air = -1,
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
    Rose = 68,
    Thistle,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Block {
    Air,
    Gravel,
    Grass,
    DirtWithGrass,
    Dirt,
    Cobblestone,
    Tnt,
    Bedrock,
    OakPlanks,
    Rose,
    Thistle,
}

impl Default for Block {
    fn default() -> Self {
        Self::Dirt
    }
}

impl Block {
    // TODO: Make this generate a dictionary instead
    fn configuration(&self) -> BlockConfiguration {
        match *self {
            Block::Air => BlockConfiguration::default(),
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
            Block::Rose => BlockConfiguration::new_entity(Tile::Rose),
            Block::Thistle => BlockConfiguration::new_entity(Tile::Thistle),
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

    pub unsafe fn draw_world(&self, world: &World) -> Result<()> {
        for (row_index, row) in world.chunks.iter().enumerate() {
            for (column_index, chunk) in row.iter().enumerate() {
                for x in 0..CHUNK_WIDTH {
                    for z in 0..CHUNK_LENGTH {
                        for y in 0..CHUNK_DEPTH {
                            let block = &chunk.blocks[x][z][y];

                            if Block::Air == *block {
                                return Ok(());
                            }

                            let configuration = block.configuration();

                            gl::UseProgram(self.shader_program);

                            gl::ActiveTexture(gl::TEXTURE0);
                            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.atlas);

                            let mvp_location = Self::uniform_location(self.shader_program, "mvp")?;
                            let id_location =
                                Self::uniform_location(self.shader_program, "blockId")?;

                            gl::BindVertexArray(self.vao);

                            let mvp = glm::translate(&self.mvp, &chunk.position);
                            let mvp = glm::translate(&mvp, &glm::vec3(x as _, y as _, z as _));
                            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, mvp.as_ptr());

                            if configuration.is_entity {
                                // center the quad
                                let mvp = glm::translate(&mvp, &glm::vec3(0.0, 0.0, -0.5));
                                gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, mvp.as_ptr());

                                // front
                                gl::Uniform1i(id_location, configuration.front);
                                gl::DrawArrays(gl::TRIANGLES, 6, 6);

                                // rotate and draw the quad
                                let mvp = glm::rotate(&mvp, -90_f32.to_radians(), &glm::Vec3::y());
                                let mvp = glm::translate(&mvp, &glm::vec3(0.0, 0.0, -0.5));
                                gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, mvp.as_ptr());

                                gl::DrawArrays(gl::TRIANGLES, 6, 6);
                            } else {
                                gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, mvp.as_ptr());

                                // TODO: This doesn't handle checking for solids across chunk borders

                                // back
                                let should_render =
                                    if let Some(neighbor) = chunk.blocks[x].get(z - 1) {
                                        !neighbor[y].configuration().is_solid
                                    } else {
                                        true
                                    };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.back);
                                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                                }

                                // front
                                let should_render =
                                    if let Some(neighbor) = chunk.blocks[x].get(z + 1) {
                                        !neighbor[y].configuration().is_solid
                                    } else {
                                        true
                                    };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.front);
                                    gl::DrawArrays(gl::TRIANGLES, 6, 6);
                                }

                                // left
                                let should_render = if let Some(neighbor) = chunk.blocks.get(x - 1)
                                {
                                    !neighbor[z][y].configuration().is_solid
                                } else {
                                    true
                                };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.left);
                                    gl::DrawArrays(gl::TRIANGLES, 12, 6);
                                }

                                // right
                                let should_render = if let Some(neighbor) = chunk.blocks.get(x + 1)
                                {
                                    !neighbor[z][y].configuration().is_solid
                                } else {
                                    true
                                };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.right);
                                    gl::DrawArrays(gl::TRIANGLES, 18, 6);
                                }

                                // bottom
                                let should_render =
                                    if let Some(neighbor) = chunk.blocks[x][z].get(y - 1) {
                                        !neighbor.configuration().is_solid
                                    } else {
                                        true
                                    };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.bottom);
                                    gl::DrawArrays(gl::TRIANGLES, 24, 6);
                                }

                                // top
                                let should_render =
                                    if let Some(neighbor) = chunk.blocks[x][z].get(y + 1) {
                                        !neighbor.configuration().is_solid
                                    } else {
                                        true
                                    };

                                if should_render {
                                    gl::Uniform1i(id_location, configuration.top);
                                    gl::DrawArrays(gl::TRIANGLES, 30, 6);
                                }
                            }
                        }
                    }
                }
            }
        }
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
