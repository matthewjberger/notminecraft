use anyhow::Result;
use glutin::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

mod app;

use app::App;

fn main() -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Not minecraft!");
    let gl_window = ContextBuilder::new().build_windowed(window, &event_loop)?;

    let gl_window = unsafe {
        gl_window
            .make_current()
            .expect("Failed to make GL context current!")
    };

    gl::load_with(|symbol| gl_window.get_proc_address(symbol));

    let dimensions = gl_window.window().inner_size();
    let aspect_ratio = dimensions.width as f32 / std::cmp::max(dimensions.height, 1) as f32;
    let mut app = App::new(aspect_ratio)?;

    event_loop.run(move |event, _, control_flow| {
        let result = || -> Result<()> {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared => {
                    app.handle_events(&event)?;
                    app.update()?;
                    app.render()?;
                    gl_window.swap_buffers()?
                }
                Event::LoopDestroyed => {
                    app.cleanup();
                    return Ok(());
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(dimensions) => {
                        app.aspect_ratio =
                            dimensions.width as f32 / std::cmp::max(dimensions.height, 1) as f32;
                        unsafe {
                            gl::Viewport(0, 0, dimensions.width as _, dimensions.height as _);
                        }
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => {
                        if (keycode, state) == (VirtualKeyCode::Escape, ElementState::Pressed) {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                _ => (),
            }
            Ok(())
        };

        if let Err(error) = result() {
            eprintln!("Application Error: {}", error);
        }
    });
}
