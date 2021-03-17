use anyhow::Result;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};

mod app;
mod input;
mod system;

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
    let mut app = App::new([dimensions.width, dimensions.height])?;

    event_loop.run(move |event, _, control_flow| {
        let result = || -> Result<()> {
            *control_flow = ControlFlow::Poll;

            if app.system.exit_requested {
                *control_flow = ControlFlow::Exit;
            }

            app.handle_events(&event)?;

            match event {
                Event::MainEventsCleared => {
                    app.update()?;
                    app.render()?;
                    gl_window.swap_buffers()?
                }
                Event::LoopDestroyed => {
                    app.cleanup();
                    return Ok(());
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(dimensions) => unsafe {
                        gl::Viewport(0, 0, dimensions.width as _, dimensions.height as _);
                    },
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
