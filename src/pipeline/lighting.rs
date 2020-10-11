use super::gbuffer::GBuffer;
use crate::include_shader;
use std::mem;
use ultraviolet::*;
use wgpu::util::DeviceExt;
use wgpu::*;

// This will be the final deferred stage
pub struct LightingPipeline {
    pub pipeline: RenderPipeline,
    tex_layout: BindGroupLayout,
    vbuf: Buffer,
}

impl LightingPipeline {
    // handle renderpass
    pub fn render<'a>(&'a mut self, gbuffer: &'a BindGroup, rpass: &mut RenderPass<'a>) {
        rpass.set_pipeline(&self.pipeline);
        // render a full screen tri
        rpass.set_bind_group(0, gbuffer, &[]);
        rpass.set_vertex_buffer(0, self.vbuf.slice(..));
        rpass.draw(0..3, 0..1);
    }

    pub fn new(ctx: &crate::Context) -> Self {
        // create a fullscreen tri
        let vbuf = Vertex::vbuf(
            &[
                &Vertex::new(-15.0, 10.0),
                &Vertex::new(5.0, 10.0),
                &Vertex::new(5.0, -10.0),
            ],
            &ctx.device,
        );

        // create bind group for g-buffer
        let tex_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("gbuffer bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::FRAGMENT,
                        ty: BindingType::SampledTexture {
                            dimension: TextureViewDimension::D2,
                            component_type: TextureComponentType::Float,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });
        let layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("lighting pipeline layout"),
                bind_group_layouts: &[&tex_layout],
                push_constant_ranges: &[],
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("lighting render pipeline"),
                layout: Some(&layout),
                vertex_stage: ProgrammableStageDescriptor {
                    module: &ctx
                        .device
                        .create_shader_module(include_shader!("lighting.vert.spv")),
                    entry_point: "main",
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &ctx
                        .device
                        .create_shader_module(include_shader!("lighting.frag.spv")),
                    entry_point: "main",
                }),
                rasterization_state: None,
                primitive_topology: PrimitiveTopology::TriangleList,
                color_states: &[ctx.sc_desc.format.into()],
                depth_stencil_state: None,
                vertex_state: VertexStateDescriptor {
                    index_format: IndexFormat::Uint16,
                    vertex_buffers: &[Vertex::desc()],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });
        Self {
            pipeline,
            tex_layout,
            vbuf,
        }
    }

    pub fn bind_gbuffer(&self, gbuffer: &GBuffer, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("gbuffer bind group"),
            layout: &self.tex_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gbuffer.position_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&gbuffer.normals_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&gbuffer.color_view),
                },
            ],
        })
    }
}

struct Vertex {
    pos: Vec2,
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
        }
    }

    pub const fn desc() -> VertexBufferDescriptor<'static> {
        VertexBufferDescriptor {
            stride: mem::size_of::<Self>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[VertexAttributeDescriptor {
                offset: 0,
                format: VertexFormat::Float2,
                shader_location: 0,
            }],
        }
    }

    fn data(&self) -> Vec<u8> {
        Vec::from(self.pos.as_byte_slice())
    }

    pub fn vbuf(data: &[&Self], device: &Device) -> Buffer {
        let data: Vec<u8> = data.iter().flat_map(|x| x.data()).collect();
        device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("lighting vertex buffer"),
            contents: &data,
            usage: BufferUsage::VERTEX,
        })
    }
}
