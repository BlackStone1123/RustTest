use wgpu::util::DeviceExt;
use winit::event::*;
use winit::window::Window;
mod texture;
use texture::Texture;
mod camera;
mod instance;
mod vertex;
use camera::Camera;
use cgmath::Rotation3;
use instance::*;
use vertex::*;

pub struct State {
    surface: wgpu::Surface,
    depth_texture: Texture,
    shadow_texture: Texture,
    msaa_texture: Texture,

    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    bg_color: wgpu::Color,

    diffuse_pipeline: wgpu::RenderPipeline,
    shadow_render_pipline: wgpu::RenderPipeline,
    solid_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_set: InstanceSet,
    camera: Camera,
    index_len: usize,

    diffuse_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,
    light_bind_group: wgpu::BindGroup,

    time: f32,
}

impl State {
    const SAMPLE_COUNT: u32 = 4;

    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);
        let depth_texture =
            Texture::create_depth_texture(&device, &config, "depth_texture", Self::SAMPLE_COUNT);
        let shadow_texture = Texture::create_depth_texture(&device, &config, "depth_texture", 1);
        let msaa_texture = Texture::create_msaa_texture(&device, &config, "msaa_texture");

        // Vertex / Index / Instance Buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(VERTICES),
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut cube_instances = (0..10)
            .flat_map(|z| (0..10).map(move |x| ArrayInstance::make(x, z)))
            .collect::<Vec<_>>();

        let floor_instance = Instance::make(
            (0.0, -10.0, 0.0).into(),
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)),
            (30.0, 1.0, 30.0).into(),
        );

        cube_instances.push(floor_instance);
        let mut instance_set = InstanceSet::make(cube_instances);
        instance_set.create_buffer(&device);

        // Texture Buffer
        let diffuse_bytes = include_bytes!("dog.jpg");
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "dogTexture").unwrap();

        // Uniform Buffer
        let mut camera = Camera::make_perspective(
            (0.0, 0.0, 50.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        camera.create_buffer(&device);

        let mut light_camera = Camera::make_orthogonal(
            (-10.0, 10.0, 10.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            -50.0,
            50.0,
            -50.0,
            50.0,
            0.1,
            100.0,
        );
        light_camera.create_buffer(&device);

        // Bind Groups
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        diffuse_texture.sampler.as_ref().unwrap(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&shadow_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(
                        shadow_texture.sampler.as_ref().unwrap(),
                    ),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera
                    .get_view_projection_buffer()
                    .unwrap()
                    .as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_camera
                    .get_view_projection_buffer()
                    .unwrap()
                    .as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });

        // Root Signature
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()),
        });

        let solid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("solidShader.wgsl").into()),
        });

        // Normal Pass PSO
        let diffuse_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("diffusePSO"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ShadowRootSignature"),
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: Self::SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Solid Pass PSO
        let solid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SolidPSO"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("SolidRootSignature"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &solid_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &solid_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: Self::SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Shadow Pass PSO
        let shadow_render_pipline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow PSO"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("ShadowRootSignature"),
                        bind_group_layouts: &[&camera_bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &shadow_shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                    unclipped_depth: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            bg_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },

            diffuse_pipeline,
            shadow_render_pipline,
            solid_pipeline,

            vertex_buffer,
            index_buffer,
            index_len: INDICES.len(),
            camera,
            instance_set,
            depth_texture,
            msaa_texture,
            shadow_texture,

            diffuse_bind_group,
            camera_bind_group,
            light_bind_group,

            time: 0.0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
                Self::SAMPLE_COUNT,
            );
            self.msaa_texture =
                Texture::create_msaa_texture(&self.device, &self.config, "msaa_texture");
        }
    }

    pub fn resize_with_current_size(&mut self) {
        self.resize(self.size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                if input.state == ElementState::Pressed {
                    if input.virtual_keycode == Some(VirtualKeyCode::Space) {
                        return true;
                    } else if input.virtual_keycode == Some(VirtualKeyCode::A) {
                        let eye_position = &mut self.camera.eye;
                        eye_position.x -= 0.5;
                        return true;
                    } else if input.virtual_keycode == Some(VirtualKeyCode::S) {
                        let eye_position = &mut self.camera.eye;
                        eye_position.y -= 0.5;
                        return true;
                    } else if input.virtual_keycode == Some(VirtualKeyCode::D) {
                        let eye_position = &mut self.camera.eye;
                        eye_position.x += 0.5;
                        return true;
                    } else if input.virtual_keycode == Some(VirtualKeyCode::W) {
                        let eye_position = &mut self.camera.eye;
                        eye_position.y += 0.5;
                        return true;
                    }
                }
                false
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers,
            } => {
                if let MouseScrollDelta::LineDelta(_i, _j) = delta {
                    let eye_position = &mut self.camera.eye;
                    eye_position.z += if *_j > 0.0 { -0.5 } else { 0.5 };
                }
                false
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.camera.update(&self.queue);
        self.time += 0.5;
        for instance in self.instance_set.set.iter_mut() {
            instance.update(self.time);
        }
        self.instance_set.update_buffer(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render CL"),
            });

        {
            // shadow Pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shdow Render Pass"),
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
                color_attachments: &[],
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_set.get_buffer().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_pipeline(&self.shadow_render_pipline);
            render_pass.set_bind_group(0, &self.light_bind_group, &[]);
            render_pass.draw_indexed(
                0..self.index_len as u32,
                0,
                0..self.instance_set.count() as _,
            );
        }
        {
            // normal pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),

                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: if Self::SAMPLE_COUNT > 1 {
                        &self.msaa_texture.view
                    } else {
                        &surface_view
                    },
                    resolve_target: if Self::SAMPLE_COUNT > 1 {
                        Some(&surface_view)
                    } else {
                        None
                    },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.bg_color),
                        store: true,
                    },
                })],
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_set.get_buffer().unwrap().slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_pipeline(&self.diffuse_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(
                0..self.index_len as u32,
                0,
                0..(self.instance_set.count() - 1) as _,
            );

            render_pass.set_pipeline(&self.solid_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(
                0..self.index_len as u32,
                0,
                (self.instance_set.count() - 1) as _..(self.instance_set.count()) as _,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
