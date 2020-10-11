use futures::executor::block_on;
use janus::app;
use janus::app::App;
use janus::pipeline::gbuffer::*;
use janus::pipeline::lighting::LightingPipeline;
use janus::voxel_data::*;
use janus::Context;
use std::time::*;
use ultraviolet::*;
use winit::event::ElementState;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event_loop::ControlFlow;

struct Application {
    window: winit::window::Window,
    ctx: Context,
    gbuf_pipe: GBufferPipeline,
    fin_pipe: LightingPipeline,
    gbuf: GBuffer,
    gbuf_bind: wgpu::BindGroup,
    voxels: VoxelBuffer,
    uniforms: Uniforms,
    ubuf: wgpu::Buffer,
    ubind: wgpu::BindGroup,
    //textures: Textures,
    tex_bind: wgpu::BindGroup,
    last_time: Instant,
    pos: Vec3,
    is_rotating: bool,
}

impl Application {
    async fn run() {
        let (window, event_loop) = app::open_window().expect("Failed to open window");
        let ctx = Context::new(&window).await;

        let fin_pipe = LightingPipeline::new(&ctx);
        let gbuf_pipe = GBufferPipeline::new(&ctx);
        let gbuf = GBuffer::new(&ctx.device, 1920, 1080);

        let uni = Uniforms {
            view_proj: ultraviolet::Mat4::identity(),
            model: ultraviolet::Mat4::identity(),
        };
        let ubuf = uni.buffer(&ctx.device);
        let ubind = gbuf_pipe.bind_uniform(&ubuf, &ctx.device);

        let voxels = VoxelBuffer::from_txt(include_str!("link.txt"), &ctx);

        let tex_bind = gbuf_pipe.bind_textures(voxels.textures(), &ctx.device);

        let app = Self {
            gbuf_pipe,
            gbuf_bind: fin_pipe.bind_gbuffer(&gbuf, &ctx.device),
            fin_pipe,
            gbuf,
            ctx,
            voxels,
            uniforms: uni,
            ubuf,
            ubind,
            window,
            //textures,
            tex_bind,
            last_time: Instant::now(),
            pos: Vec3::new(0.0, 10.0, -50.0),
            is_rotating: false,
        };
        app.run(event_loop);
    }
}

impl App for Application {
    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    fn key_event(&mut self, input: &KeyboardInput, _ctrl: &mut ControlFlow) {
        if let Some(VirtualKeyCode::W) = input.virtual_keycode {
            self.pos.z += 0.9;
        }
        if let Some(VirtualKeyCode::S) = input.virtual_keycode {
            self.pos.z -= 0.9;
        }
        if let Some(VirtualKeyCode::A) = input.virtual_keycode {
            self.pos.x -= 0.9;
        }
        if let Some(VirtualKeyCode::D) = input.virtual_keycode {
            self.pos.x += 0.9;
        }
        if let Some(VirtualKeyCode::E) = input.virtual_keycode {
            self.pos.y += 0.9;
        }
        if let Some(VirtualKeyCode::Q) = input.virtual_keycode {
            self.pos.y -= 0.9;
        }
        if let Some(VirtualKeyCode::R) = input.virtual_keycode {
            if let ElementState::Released = input.state {
                self.is_rotating = !self.is_rotating;
                self.last_time = Instant::now(); // reset timer
            }
        }
        self.window.request_redraw();
    }

    fn render(&mut self) {
        // this is gross but I don't care right now
        // if this was an actuall game we wouldn't reconstruct the camera every frame
        let mut camera = janus::Camera::new(self.pos, Vec3::new(0.0, 10.0, 0.0));
        camera.zfar = f32::MAX;
        camera.aspect = self.ctx.size().0 as f32 / self.ctx.size().1 as f32;

        self.uniforms.view_proj = camera.proj();
        self.uniforms.view_proj = self.uniforms.view_proj;
        self.uniforms.model = Mat4::from_rotation_y(-90.0f32.to_radians());
        if self.is_rotating {
            self.uniforms.model =
                Mat4::from_rotation_y(self.last_time.elapsed().as_millis() as f32 / 1_000.0)
                    * self.uniforms.model;
        }
        self.uniforms.update_buffer(&self.ubuf, &self.ctx.queue);

        let (vbuf, ibuf) = self.voxels.buffers();

        let mut encoder = self.ctx.encoder();
        {
            let mut rpass = self.gbuf.render(&mut encoder);
            self.gbuf_pipe.render_ind(
                vbuf,
                ibuf,
                self.voxels.index_count(),
                &self.ubind,
                &self.tex_bind,
                &mut rpass,
            );
        }

        let frame = self.ctx.next_frame();
        {
            let mut rpass = self.ctx.render_pass(&mut encoder, &frame);
            self.fin_pipe.render(&self.gbuf_bind, &mut rpass);
        }
        self.ctx.run_encoder(encoder);
        std::thread::sleep(Duration::from_millis(16));
        self.window.request_redraw();
    }
}

fn main() {
    block_on(Application::run());
}
