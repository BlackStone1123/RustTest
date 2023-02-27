use wgpu::util::DeviceExt;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    projector: Box<dyn Projector>,
    buffer: Option<wgpu::Buffer>,
}
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = self.projector.get_projection_matrix();
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn get_view_projection_matrix(&self) -> CameraUniform {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&self);
        uniform
    }

    pub fn get_view_projection_buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }

    pub fn create_buffer(&mut self, device: &wgpu::Device) {
        let camera_uniform = self.get_view_projection_matrix();
        let camera_buffer: wgpu::Buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        self.buffer = Some(camera_buffer);
    }

    pub fn update(&self, queue: &wgpu::Queue) {
        let camera_uniform = self.get_view_projection_matrix();
        queue.write_buffer(
            self.get_view_projection_buffer().unwrap(),
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projector.resize(width, height)
    }

    pub fn make_perspective(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Camera {
        let perspective = PerspectiveProjector {
            aspect,
            fovy,
            znear,
            zfar,
        };

        Camera {
            eye,
            target,
            up,
            buffer: None,
            projector: Box::new(perspective),
        }
    }

    pub fn make_orthogonal(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Camera {
        let ortho = OrthoProjector {
            left,
            right,
            top,
            bottom,
            near,
            far,
        };

        Camera {
            eye,
            target,
            up,
            buffer: None,
            projector: Box::new(ortho),
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

trait Projector {
    fn get_projection_matrix(&self) -> cgmath::Matrix4<f32>;
    fn resize(&mut self, width: u32, height: u32);
}

struct PerspectiveProjector {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projector for PerspectiveProjector {
    fn get_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32
    }
}

struct OrthoProjector {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    near: f32,
    far: f32,
}

impl Projector for OrthoProjector {
    fn get_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
    fn resize(&mut self, _width: u32, _height: u32) {}
}
