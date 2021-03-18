use gl::types::*;

#[rustfmt::skip]
pub const VERTICES: &[f32; 40] =
    &[
        // Front
       -0.5, -0.5,  0.5, 0.0, 0.0, // lower left
        0.5, -0.5,  0.5, 1.0, 0.0, // lower right
        0.5,  0.5,  0.5, 1.0, 1.0, // upper right
       -0.5,  0.5,  0.5, 0.0, 1.0, // upper left
        // Back
       -0.5, -0.5, -0.5, 0.0, 0.0,
        0.5, -0.5, -0.5, 1.0, 0.0,
        0.5,  0.5, -0.5, 1.0, 1.0,
       -0.5,  0.5, -0.5, 0.0, 1.0,
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

pub struct Cube {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
}

impl Cube {
    pub fn new() -> Self {
        Self {
            vao: Self::create_vao(),
            vbo: Self::create_vbo(),
            ebo: Self::create_ebo(),
        }
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

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
        gl::DrawElements(
            gl::TRIANGLES,
            INDICES.len() as i32,
            gl::UNSIGNED_INT,
            std::ptr::null(),
        );
    }
}

impl Drop for Cube {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);
        }
    }
}
