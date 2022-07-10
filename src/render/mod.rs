use std::error::Error;
use std::sync::Arc;
use std::iter;
use std::time::Duration;

use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, viewport::Viewport, vertex::TwoBuffersDefinition};
use vulkano::format::{ClearValue, Format};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool};
use vulkano::framebuffer::{Subpass, Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::image::{Dimensions, StorageImage, AttachmentImage};
use vulkano::sync::{self, GpuFuture};

use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
}
vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}
vulkano::impl_vertex!(Normal, normal);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/render/vert.glsl"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/render/frag.glsl"
    }
}

pub struct Renderer {
    logical_device: Arc<Device>,
    queue: Arc<Queue>,
    uniform_buffer: CpuBufferPool::<vs::ty::Data>,
    vs: vs::Shader,
    fs: fs::Shader,

    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>
}

pub struct Pipeline {
    renderer: Renderer,

    pub width: u32,
    pub height: u32,

    clear_values: Vec<vulkano::format::ClearValue>,
    image: Arc<StorageImage<Format>>,

    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    framebuffer: Arc<dyn FramebufferAbstract + Send + Sync>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[f32]>>,
    normal_buffer: Arc<CpuAccessibleBuffer<[f32]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,

    output_buffer: Arc<CpuAccessibleBuffer<[u8]>>
}

impl Pipeline {
    pub fn render(&mut self, elapsed: Duration) -> Result<Vec<u8>, Box<dyn Error>> {
        let uniform_buffer_subbuffer = {
            let rotation = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
            let rotation = Matrix3::from_angle_y(Rad(rotation as f32));
            let aspect_ratio = (self.width as f32 / 2.0) / self.height as f32;
            let projection = 
                cgmath::perspective(
                    Rad(std::f32::consts::FRAC_PI_2 / 1.5), 
                    aspect_ratio, 
                    0.01, 
                    100.0
                );

            let view = 
                Matrix4::look_at(
                    Point3::new(-0.5, 1.0, -2.0),
                    Point3::new(0.0, 0.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                );
    
            let scale = Matrix4::from_scale(1.0);
            let uniform_data = vs::ty::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: projection.into()
            };
            self.renderer.uniform_buffer.next(uniform_data).unwrap()
        };

        let layout = self.pipeline.descriptor_set_layout(0).unwrap();
        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(layout.clone())
                .add_buffer(uniform_buffer_subbuffer)
                .unwrap()
                .build()
                .unwrap()
            );
        
            let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                self.renderer.logical_device.clone(),
                self.renderer.queue.family()
            )?;

        builder
            .begin_render_pass(self.framebuffer.clone(), SubpassContents::Inline, self.clear_values.clone())?
            .draw_indexed(
                self.pipeline.clone(),
                &DynamicState::none(),
                vec![self.vertex_buffer.clone(), self.normal_buffer.clone()],
                self.index_buffer.clone(),
                descriptor_set.clone(),
                ()
            )
            .unwrap()
            .end_render_pass()
            .unwrap()
            .copy_image_to_buffer(self.image.clone(), self.output_buffer.clone())
            .unwrap();

        let command_buffer = builder.build().unwrap();
        let future = sync::now(self.renderer.logical_device.clone())
            .then_execute(self.renderer.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();

        let v: Vec<u8> = (&self.output_buffer.read()?).to_vec();
        
        Ok(v)
    }

    pub fn new(
        renderer: Renderer, 
        width: u32, 
        height: u32, 
        vertices: std::iter::Cloned<std::slice::Iter<f32>>, 
        normals: std::iter::Cloned<std::slice::Iter<f32>>, 
        indices: std::iter::Cloned<std::slice::Iter<u32>>
    ) -> Result<Self, Box<dyn Error>> {

        let dimensions = Dimensions::Dim2d {
            width: width,
            height: height,
        };

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions.width() as f32, dimensions.height() as f32],
            depth_range: 0.0..1.0
        };

        let clear_values = vec![[0.0, 0.0, 0.0, 0.0].into(), 1f32.into(), ClearValue::None];

        let intermediary = AttachmentImage::transient_multisampled(
            renderer.logical_device.clone(),
            dimensions.width_height(),
            4,
            Format::R8G8B8A8Unorm
        )?;
    
        let depth_buffer = AttachmentImage::transient_multisampled(
            renderer.logical_device.clone(), 
            dimensions.width_height(), 
            4,
            Format::D16Unorm
        )?;
    
        let image = StorageImage::new(
            renderer.logical_device.clone(),
            dimensions,
            Format::R8G8B8A8Unorm,
            Some(renderer.queue.family())
        )?;

        let pipeline = Arc::new(
            GraphicsPipeline::start()
            .vertex_input(TwoBuffersDefinition::<Vertex, Normal>::new())
            .vertex_shader(renderer.vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(viewport.clone()))
            .fragment_shader(renderer.fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(renderer.render_pass.clone(), 0).unwrap())
            .build(renderer.logical_device.clone())?
        ) as Arc<dyn GraphicsPipelineAbstract + Send + Sync>;

        let framebuffer = Arc::new(
            Framebuffer::start(renderer.render_pass.clone())
                .add(intermediary.clone())
                .unwrap()
                .add(depth_buffer.clone())
                .unwrap()
                .add(image.clone())
                .unwrap()
                .build()
                .unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>;

        let vertex_buffer =
            CpuAccessibleBuffer::from_iter(renderer.logical_device.clone(), BufferUsage::all(), false, vertices)
            .unwrap();

        let normal_buffer = 
            CpuAccessibleBuffer::from_iter(renderer.logical_device.clone(), BufferUsage::all(), false, normals)
            .unwrap();

        let index_buffer = 
            CpuAccessibleBuffer::from_iter(renderer.logical_device.clone(), BufferUsage::all(), false, indices)
            .unwrap();

        

        let output_buffer = 
            CpuAccessibleBuffer::from_iter(
                renderer.logical_device.clone(),
                BufferUsage::all(),
                false,
                (0 .. width * height * 4).map(|_| 0u8)
            )?;

        Ok(
            Self {
                renderer,
                width,
                height,
                clear_values,
                image,
                pipeline,
                framebuffer,
                vertex_buffer,
                normal_buffer,
                index_buffer,
                output_buffer
            }
        )
    }
}

impl Renderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let instance = Instance::new(None, &InstanceExtensions::none(), None)?;
        let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

        let queue_family = physical_device
            .queue_families()
            .find(|queue| queue.supports_graphics())
            .unwrap();

        let (logical_device, mut queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &DeviceExtensions::none(),
            [(queue_family, 1.0)].iter().cloned()
        )?;
        let queue = queues.next().unwrap();

        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(logical_device.clone(), BufferUsage::all());

        let vs = vs::Shader::load(logical_device.clone())?;
        let fs = fs::Shader::load(logical_device.clone())?;

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(
                logical_device.clone(),
                attachments: {
                    intermediary: {
                        load: Clear,
                        store: DontCare,
                        format: Format::R8G8B8A8Unorm,
                        samples: 4,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 4,
                    },
                    color: {
                        load: DontCare,
                        store: Store,
                        format: Format::R8G8B8A8Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    color: [intermediary],
                    depth_stencil: {depth},
                    resolve: [color]
                }
            )?
        );

        Ok(
            Self {
                logical_device,
                queue,
                uniform_buffer,
                vs,
                fs,
                render_pass
            }
        )

    }
}