use anyhow::Result;
use gl::types::*;
use glutin::{
    dpi::PhysicalPosition,
    event::{Event, VirtualKeyCode},
    window::Window,
};
use nalgebra_glm as glm;

use crate::{
    block::{Cube, World},
    camera::{CameraDirection, FreeCamera},
    input::Input,
    system::System,
};

pub struct App {
    world: World,
    block: Cube,
    camera: FreeCamera,
    pub system: System,
    pub input: Input,
}

impl App {
    pub fn new(dimensions: [u32; 2]) -> Result<Self> {
        // Self::enable_wireframe();
        Ok(Self {
            world: World::new(),
            block: Cube::new()?,
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

    pub fn update(&mut self, window: &Window) -> Result<()> {
        if self.input.is_key_pressed(VirtualKeyCode::Escape) {
            self.system.exit_requested = true;
        }

        self.update_free_camera(window)?;

        let perspective = glm::perspective_zo(
            self.system.aspect_ratio(),
            80_f32.to_radians(),
            0.01,
            1000.0,
        );
        let model = glm::Mat4::identity();
        self.block.mvp = perspective * self.camera.view_matrix() * model;
        Ok(())
    }

    pub fn handle_events(&mut self, event: &Event<()>) -> Result<()> {
        self.system.handle_event(event);
        self.input.handle_event(event, self.system.window_center());
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);

            let background_color: &[GLfloat; 4] = &[0.25, 0.25, 0.25, 1.0];
            gl::ClearBufferfv(gl::COLOR, 0, background_color as *const f32);
            gl::ClearBufferfv(gl::DEPTH, 0, &[1.0 as GLfloat] as *const f32);

            self.block.draw_world(&self.world)?;
        }
        Ok(())
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
