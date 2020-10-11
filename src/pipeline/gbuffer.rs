use crate::include_shader;
use std::mem;
use ultraviolet::*;
use wgpu::util::DeviceExt;
use wgpu::*;

pub struct GBuffer {
    pub position_view: TextureView,
    pub position_tex: Texture,
    pub normals_view: TextureView,
    pub normals_tex: Texture,
    pub color_view: TextureView,
    pub color_tex: Texture,
    pub depth_tex: Texture,
    pub depth_view: TextureView,
}

impl GBuffer {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let (position_tex, position_view) = create_tex(device, width, height);
        let (normals_tex, normals_view) = create_tex(device, width, height);
        let (color_tex, color_view) = create_tex(device, width, height);
        let (depth_tex, depth_view) = create_depth_tex(device, width, height);

        Self {
            position_tex,
            position_view,
            normals_tex,
            normals_view,
            color_tex,
            color_view,
            depth_tex,
            depth_view,
        }
    }

    pub fn render<'a>(&'a self, encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[
                RenderPassColorAttachmentDescriptor {
                    attachment: &self.position_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                },
                RenderPassColorAttachmentDescriptor {
                    attachment: &self.normals_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                },
                RenderPassColorAttachmentDescriptor {
                    attachment: &self.color_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }
}

pub struct Textures {
    pub diffuse_view: TextureView,
    pub diffuse_tex: Texture,
}

#[derive(Debug)]
pub struct TextureData<'a> {
    pub texels: &'a [u8],
    pub dim: [u32; 2],
}

impl<'a> TextureData<'a> {
    pub fn new(texels: &'a [u8], dim: [u32; 2]) -> Self {
        Self { texels, dim }
    }

    pub fn create(&self, ctx: &crate::Context) -> (Texture, TextureView) {
        let tex = ctx.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: self.dim[0],
                height: self.dim[1],
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });

        let view = tex.create_view(&TextureViewDescriptor::default());

        ctx.queue.write_texture(
            TextureCopyView {
                texture: &tex,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &self.texels,
            TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * self.dim[0],
                rows_per_image: self.dim[1],
            },
            Extent3d {
                width: self.dim[0],
                height: self.dim[1],
                depth: 1,
            },
        );
        (tex, view)
    }
}

impl Textures {
    pub fn new<'a>(diffuse: TextureData<'a>, ctx: &crate::Context) -> Textures {
        let (diffuse_tex, diffuse_view) = diffuse.create(ctx);
        Self {
            diffuse_tex,
            diffuse_view,
        }
    }
}

pub struct GBufferPipeline {
    pub pipeline: RenderPipeline,
    uniform_layout: BindGroupLayout,
    tex_layout: BindGroupLayout,
}

impl GBufferPipeline {
    pub fn render<'a>(
        &'a mut self,
        vbuf: &'a Buffer,
        vcnt: u32,
        uniforms: &'a BindGroup,
        textures: &'a BindGroup,
        rpass: &mut RenderPass<'a>,
    ) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, uniforms, &[]);
        rpass.set_bind_group(1, textures, &[]);
        rpass.set_vertex_buffer(0, vbuf.slice(..));
        rpass.draw(0..vcnt, 0..1);
    }

    pub fn render_ind<'a>(
        &'a mut self,
        vbuf: &'a Buffer,
        ibuf: &'a Buffer,
        icnt: u32,
        uniforms: &'a BindGroup,
        textures: &'a BindGroup,
        rpass: &mut RenderPass<'a>,
    ) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, uniforms, &[]);
        rpass.set_bind_group(1, textures, &[]);
        rpass.set_index_buffer(ibuf.slice(..));
        rpass.set_vertex_buffer(0, vbuf.slice(..));
        rpass.draw_indexed(0..icnt, 0, 0..1);
    }

    pub fn new(ctx: &crate::Context) -> Self {
        let uniform_layout = Uniforms::layout(&ctx.device);

        let tex_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        dimension: TextureViewDimension::D2,
                        component_type: TextureComponentType::Float,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("GBuffer pipeline layout"),
                bind_group_layouts: &[&uniform_layout, &tex_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Gbuffer Pipeline"),
                layout: Some(&layout),
                vertex_stage: ProgrammableStageDescriptor {
                    module: &ctx
                        .device
                        .create_shader_module(include_shader!("gbuffer.vert.spv")),
                    entry_point: "main",
                },
                fragment_stage: Some(ProgrammableStageDescriptor {
                    module: &ctx
                        .device
                        .create_shader_module(include_shader!("gbuffer.frag.spv")),
                    entry_point: "main",
                }),
                rasterization_state: None,
                primitive_topology: PrimitiveTopology::TriangleList,
                color_states: &[
                    // position
                    ColorStateDescriptor {
                        format: TextureFormat::Rgba16Float,
                        alpha_blend: BlendDescriptor::REPLACE,
                        color_blend: BlendDescriptor::REPLACE,
                        write_mask: ColorWrite::COLOR,
                    },
                    // normals
                    ColorStateDescriptor {
                        format: TextureFormat::Rgba16Float,
                        alpha_blend: BlendDescriptor::REPLACE,
                        color_blend: BlendDescriptor::REPLACE,
                        write_mask: ColorWrite::COLOR,
                    },
                    // albedo/specular
                    ColorStateDescriptor {
                        format: TextureFormat::Rgba16Float,
                        alpha_blend: BlendDescriptor::REPLACE,
                        color_blend: BlendDescriptor::REPLACE,
                        write_mask: ColorWrite::ALL,
                    },
                ],
                depth_stencil_state: Some(DepthStencilStateDescriptor {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less, // TODO: Might need to play with this
                    stencil: StencilStateDescriptor::default(),
                }),
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
            uniform_layout,
            tex_layout,
        }
    }

    pub fn bind_uniform(&self, uniforms: &Buffer, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.uniform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniforms.slice(..)),
            }],
        })
    }

    pub fn bind_textures(&self, textures: &Textures, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("gbuffer bind group"),
            layout: &self.tex_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&textures.diffuse_view),
            }],
        })
    }
}

#[derive(Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub const fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            uv: Vec2::new(u, v),
        }
    }

    pub const fn desc() -> VertexBufferDescriptor<'static> {
        VertexBufferDescriptor {
            stride: mem::size_of::<Self>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &[
                VertexAttributeDescriptor {
                    offset: 0,
                    format: VertexFormat::Float3,
                    shader_location: 0,
                },
                VertexAttributeDescriptor {
                    offset: mem::size_of::<Vec3>() as BufferAddress,
                    format: VertexFormat::Float2,
                    shader_location: 1,
                },
            ],
        }
    }

    pub fn data(&self) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(self.pos.as_byte_slice());
        data.extend_from_slice(self.uv.as_byte_slice());
        data
    }

    pub fn vbuf(data: &[Self], device: &Device) -> Buffer {
        let data: Vec<u8> = data.iter().flat_map(|x| x.data()).collect();
        device.create_buffer_init(&util::BufferInitDescriptor {
            label: None,
            contents: &data,
            usage: BufferUsage::VERTEX,
        })
    }

    pub fn ibuf(data: &[u16], device: &Device) -> Buffer {
        device.create_buffer_init(&util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data),
            usage: BufferUsage::INDEX,
        })
    }
}

pub struct Uniforms {
    pub view_proj: Mat4,
    pub model: Mat4,
}

impl Uniforms {
    pub const fn size() -> std::num::NonZeroU64 {
        unsafe { std::num::NonZeroU64::new_unchecked(std::mem::size_of::<Self>() as u64) }
    }

    fn data(&self) -> Vec<u8> {
        let mut data = vec![];
        data.extend_from_slice(self.view_proj.as_byte_slice());
        data.extend_from_slice(self.model.as_byte_slice());
        data
    }

    pub fn layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Gbuffer Uniform"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: Some(Self::size()),
                },
                count: None,
            }],
        })
    }

    pub fn buffer(&self, device: &Device) -> Buffer {
        device.create_buffer_init(&util::BufferInitDescriptor {
            label: None,
            contents: &self.data(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        })
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, queue: &wgpu::Queue) {
        queue.write_buffer(buffer, 0, &self.data());
    }
}

fn create_tex(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::STORAGE,
    });

    let view = tex.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(TextureFormat::Rgba16Float),
        dimension: None,
        aspect: wgpu::TextureAspect::All,

        base_mip_level: 0,
        level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });
    (tex, view)
}

fn create_depth_tex(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
    let tex = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
    });

    let view = tex.create_view(&TextureViewDescriptor::default());
    (tex, view)
}
