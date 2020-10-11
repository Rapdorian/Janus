// TODO: Depth testing in gbuffer
pub mod app;
mod camera;
mod context;
pub mod pipeline;
pub mod voxel_data;

pub use camera::*;
pub use context::*;

#[macro_export]
macro_rules! include_shader {
    ($file:expr) => {
        wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/", $file))
    };
}
