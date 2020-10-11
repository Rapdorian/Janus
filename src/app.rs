use winit::error::OsError;
use winit::event::*;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub fn open_window() -> Result<(Window, EventLoop<()>), OsError> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;
    Ok((window, event_loop))
}

pub trait App: 'static + Sized {
    fn update(&mut self) {}
    fn render(&mut self) {}
    fn resize(&mut self, _width: u32, _height: u32) {
        eprintln!("App::resize() not implemented")
    }
    fn key_event(&mut self, _input: &KeyboardInput, _ctrl: &mut ControlFlow) {}
    fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(new_inner_size.width, new_inner_size.height);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => self.key_event(input, control_flow),
            _ => {}
        }
    }
    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                self.render();
            }
            Event::WindowEvent { ref event, .. } => self.window_event(event, control_flow),
            _ => {}
        });
    }
}
