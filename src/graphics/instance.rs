use cgmath::*;
use wgpu::util::DeviceExt;

pub struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    x: u32,
    z: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }

    pub fn make(x: u32, z: u32) -> Instance {
        Instance {
            position: cgmath::Vector3 {
                x: 3.0 * x as f32 - 14.5,
                y: 0.0,
                z: -2.5 * z as f32 + 14.5,
            },
            rotation: cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_y(),
                cgmath::Deg(0.0),
            ),
            x,
            z,
        }
    }
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
    pub fn update(&mut self, time: f32) {
        let angle = 5.0 * self.z as f32 * time;
        self.position.y = (time + self.x as f32).sin();
        self.rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(angle));
    }
}

pub struct InstanceSet {
    pub set: Vec<Instance>,
    buffer: Option<wgpu::Buffer>,
}

impl InstanceSet {
    pub fn create_buffer(&mut self, device: &wgpu::Device) {
        let instance_raw_data: Vec<_> = self.set.iter().map(Instance::to_raw).collect();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_raw_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        self.buffer = Some(instance_buffer);
    }

    pub fn update_buffer(&self, queue: &wgpu::Queue) {
        let instance_raw_data: Vec<_> = self.set.iter().map(Instance::to_raw).collect();
        queue.write_buffer(
            self.get_buffer().unwrap(),
            0,
            bytemuck::cast_slice(&instance_raw_data),
        );
    }
    pub fn make(set: Vec<Instance>) -> InstanceSet {
        InstanceSet { set, buffer: None }
    }
    pub fn get_buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }
    pub fn count(&self) -> usize {
        self.set.len()
    }
}
