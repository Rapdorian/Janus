use crate::pipeline::gbuffer::TextureData;
use crate::pipeline::gbuffer::Textures;
use crate::pipeline::gbuffer::Vertex;
use ultraviolet::*;

mod txt;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    visible: bool,
}

impl Color {
    pub const CLEAR: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
        visible: false,
    };

    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            visible: true,
        }
    }
}

pub struct VoxelBuffer {
    data: VoxelData,
    vbuffer: wgpu::Buffer,
    ibuffer: wgpu::Buffer,
    textures: Textures,
    icnt: u32,
    vcnt: u32,
}

impl VoxelBuffer {
    pub fn from_data(data: VoxelData, ctx: &crate::Context) -> Self {
        let (verts, indices, texels) = data.verts();
        let vbuffer = Vertex::vbuf(&verts, &ctx.device);
        let ibuffer = Vertex::ibuf(&indices, &ctx.device);

        let d_tex = TextureData::new(&texels.0, texels.1);

        let textures = Textures::new(d_tex, &ctx);
        Self {
            data,
            vbuffer,
            ibuffer,
            icnt: indices.len() as u32,
            vcnt: verts.len() as u32,
            textures,
        }
    }

    pub fn from_txt(txt: &str, ctx: &crate::Context) -> Self {
        Self::from_data(VoxelData::from_txt(txt), ctx)
    }

    pub fn new(
        colors: Vec<Color>,
        width: u32,
        height: u32,
        depth: u32,
        ctx: &crate::Context,
    ) -> Self {
        Self::from_data(VoxelData::new(colors, width, height, depth), ctx)
    }

    pub fn textures(&self) -> &Textures {
        &self.textures
    }

    pub fn index_count(&self) -> u32 {
        self.icnt
    }

    pub fn vert_count(&self) -> u32 {
        self.vcnt
    }

    pub fn buffers(&self) -> (&wgpu::Buffer, &wgpu::Buffer) {
        (&self.vbuffer, &self.ibuffer)
    }
}

pub struct VoxelData {
    colors: Vec<Color>,
    width: u32,
    height: u32,
    depth: u32,
}

impl VoxelData {
    pub fn from_txt(txt: &str) -> Self {
        let data = txt::import_txt(txt);
        // find bounds
        let mut x_bound = data[0].1[0]..data[0].1[0];
        let mut y_bound = data[0].1[1]..data[0].1[1];
        let mut z_bound = data[0].1[2]..data[0].1[2];

        for (_, pos) in &data {
            if pos[0] > x_bound.end {
                x_bound.end = pos[0];
            }
            if pos[0] < x_bound.start {
                x_bound.start = pos[0];
            }

            if pos[1] > y_bound.end {
                y_bound.end = pos[1];
            }
            if pos[1] < y_bound.start {
                y_bound.start = pos[1];
            }

            if pos[2] > z_bound.end {
                z_bound.end = pos[2];
            }
            if pos[2] < z_bound.start {
                z_bound.start = pos[2];
            }
        }
        // determine how we'll have to shift the data
        let x_off = 0 - x_bound.start;
        let y_off = 0 - y_bound.start;
        let z_off = 0 - z_bound.start;
        let width = x_bound.end - x_bound.start + 1;
        let height = y_bound.end - y_bound.start + 1;
        let depth = z_bound.end - z_bound.start + 1;

        let len = (width * height * depth) as usize;

        // now we can create our data
        let mut colors = vec![Color::CLEAR; len];
        for (col, pos) in data {
            let o_pos = [pos[0] + x_off, pos[1] + y_off, pos[2] + z_off];
            let ind = (o_pos[0] * height * depth) + (o_pos[1] * depth) + o_pos[2];

            colors[ind as usize] = Color::new(col.r, col.g, col.b);
        }
        Self::new(colors, width as u32, height as u32, depth as u32)
    }

    pub fn new(colors: Vec<Color>, width: u32, height: u32, depth: u32) -> Self {
        Self {
            colors,
            width,
            height,
            depth,
        }
    }

    fn verts(&self) -> (Vec<Vertex>, Vec<u16>, (Vec<u8>, [u32; 2])) {
        let mut verts = vec![];
        let mut indices = vec![];
        let mut texels = vec![];
        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.depth {
                    let index = (x * self.depth * self.height) + (y * self.depth) + z;
                    let voxel = &self.colors[index as usize];
                    if voxel.visible {
                        let ind_offset = verts.len();
                        {
                            let x = x as f32;
                            let y = y as f32;
                            let z = z as f32;

                            let tex_off = (texels.len() / 4) as f32;
                            texels.push(voxel.red);
                            texels.push(voxel.green);
                            texels.push(voxel.blue);
                            texels.push(0xFF);

                            verts.push(Vertex::new(0.0 + x, 0.0 + y, 0.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(0.0 + x, 0.0 + y, 1.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(0.0 + x, 1.0 + y, 0.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(0.0 + x, 1.0 + y, 1.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(1.0 + x, 0.0 + y, 0.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(1.0 + x, 0.0 + y, 1.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(1.0 + x, 1.0 + y, 1.0 + z, 0.0 + tex_off, 0.0));
                            verts.push(Vertex::new(1.0 + x, 1.0 + y, 0.0 + z, 0.0 + tex_off, 0.0));
                        }

                        #[rustfmt::skip]
                        indices.extend([
                            // left face
                            0, 1, 3,
                            0, 3, 2,
                            // bottom face
                            0, 4, 5,
                            0, 5, 1,
                            // // back face
                            0, 2, 4,
                            2, 7, 4,
                            // front face
                            1, 3, 5,
                            3, 6, 5,
                            // right face
                            4, 5, 6,
                            4, 6, 7,
                            // // top face
                            2, 7, 6,
                            2, 6, 3,
                        ].iter().map(|x| (x + ind_offset) as u16));
                    }
                }
            }
        }

        for vert in &mut verts {
            let ind = vert.uv.x;
            vert.uv.x = ind / (texels.len() as f32 / 4.0);
            vert.uv.y = 0.5;
        }
        let tlen = texels.len() as u32;
        (verts, indices, (texels, [tlen / 4, 1]))
    }
}
