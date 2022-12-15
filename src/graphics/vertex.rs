#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, -1.0],
        color: [0.0, 1.0],
    }, //0
    Vertex {
        position: [1.0, -1.0, -1.0],
        color: [1.0, 1.0],
    }, //1
    Vertex {
        position: [-1.0, 1.0, -1.0],
        color: [0.0, 0.0],
    }, //2
    Vertex {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 0.0],
    }, //3
    Vertex {
        position: [-1.0, -1.0, 1.0],
        color: [1.0, 1.0],
    }, //4
    Vertex {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 1.0],
    }, //5
    Vertex {
        position: [-1.0, 1.0, 1.0],
        color: [1.0, 0.0],
    }, //6
    Vertex {
        position: [1.0, 1.0, 1.0],
        color: [0.0, 0.0],
    }, //7
    Vertex {
        position: [-1.0, -1.0, -1.0],
        color: [1.0, 1.0],
    }, //8
    Vertex {
        position: [-1.0, 1.0, -1.0], // 9
        color: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0], // 10
        color: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0], // 11
        color: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0], // 12
        color: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0], // 13
        color: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0], // 14
        color: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0], // 15
        color: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0], // 16
        color: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0], // 17
        color: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0], // 18
        color: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0], // 19
        color: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0], // 20
        color: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0], // 21
        color: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0], // 22
        color: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0], // 23
        color: [1.0, 0.0],
    },
];

pub const INDICES: &[u16] = &[
    0, 2, 1, 2, 3, 1, 4, 5, 7, 4, 7, 6, 8, 10, 9, 10, 11, 9, 12, 13, 15, 12, 15, 14, 16, 17, 18,
    18, 17, 19, 20, 23, 21, 20, 22, 23,
];
