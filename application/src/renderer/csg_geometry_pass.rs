use crate::renderer::uniforms::SegmentInstanceData;
use propellant::renderer::uniforms::CameraUniform;
use propellant::renderer::ShaderSource;
use propellant::vulkan::ash_vk;
use propellant::vulkan::RingInstanceBuffer;
use propellant::vulkan::RingUniformBuffer;
use propellant::vulkan::MAX_FRAMES_IN_FLIGHT;

pub struct CsgGeometryPass;

impl propellant::renderer::RenderPass for CsgGeometryPass {
    const NAME: &'static str = "Csg Geometry Render Pass";
    type Input = propellant::renderer::RenderGraphInput;
    type Output = propellant::renderer::RenderGraphOutput;

    const FRAG_SHADER: ShaderSource = propellant::frag_shader!("shaders/segment.frag.glsl");
    const VERT_SHADER: ShaderSource = propellant::vert_shader!("shaders/segment.vert.glsl");

    type Bindings = (CameraUniformBinding, SegmentInstanceBinding, CsgBufferBinding);

    fn create() -> Self {
        Self
    }

    fn render(
        &mut self,
        context: &propellant::renderer::RenderPassRenderContext,
        (camera, instances, _): &Self::Bindings,
        _: Self::Input,
    ) -> propellant::renderer::VkResult<Self::Output> {
        if !camera.has_main_camera {
            log::warn!("No main camera found, skipping geometry pass");
            return Ok(propellant::renderer::RenderGraphOutput);
        }

        let command_buffer = *context.command_buffer;
        let instance_count = instances.instance_count;
        unsafe { context.device.cmd_draw(command_buffer, 6, instance_count as u32, 0, 0) };

        Ok(propellant::renderer::RenderGraphOutput)
    }
}

pub struct CameraUniformBinding {
    has_main_camera: bool,
    uniform_buffer: RingUniformBuffer<CameraUniform, MAX_FRAMES_IN_FLIGHT>,
}

impl propellant::renderer::Binding for CameraUniformBinding {
    const NAME: &'static str = "Camera Uniform Binding";

    const DESCRIPTOR_TYPE: ash_vk::DescriptorType = propellant::vulkan::ash_vk::DescriptorType::UNIFORM_BUFFER;
    const BINDING_STAGE: propellant::renderer::BindingStage = propellant::renderer::BindingStage::FragmentVertex;
    const DESCRIPTOR_COUNT: u32 = 1;

    type CreateError = ash_vk::Result;

    fn create(context: &propellant::renderer::RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        let capacity = std::num::NonZeroUsize::new(1).unwrap();
        let uniform_buffer = propellant::vulkan::RingUniformBuffer::create(context.device, capacity)?;
        Ok(Self {
            has_main_camera: true,
            uniform_buffer,
        })
    }

    fn update(&mut self, context: &propellant::renderer::RenderPassRenderContext) -> Result<(), ash_vk::Result> {
        use propellant::components::camera::MainCamera;
        use propellant::components::Camera;
        use propellant::components::Transform;

        /* Get the main camera in the scene */
        let mut camera_query = context.world.query::<(&Transform, &Camera<MainCamera>)>();
        let Some((_, (transform, camera))) = camera_query.iter().next() else {
            self.has_main_camera = false;
            return Ok(());
        };

        /* Set the main camera uniforms */
        let camera_uniform = CameraUniform::from_camera_components(transform, camera);
        self.uniform_buffer.get_mut(context.frame_index).write(&camera_uniform);
        self.has_main_camera = true;

        Ok(())
    }

    fn descriptor_buffer_info(&self, frame_index: usize) -> ash_vk::DescriptorBufferInfo {
        self.uniform_buffer.get(frame_index).descriptor_info()
    }
}

const SEGMENT_INSTANCE_CAPACITY: usize = 128;

pub struct SegmentInstanceBinding {
    instance_count: usize,
    instance_buffer: RingInstanceBuffer<SegmentInstanceData, MAX_FRAMES_IN_FLIGHT>,
}

impl propellant::renderer::Binding for SegmentInstanceBinding {
    const NAME: &'static str = "Segment Instance Buffer";

    const DESCRIPTOR_TYPE: ash_vk::DescriptorType = ash_vk::DescriptorType::STORAGE_BUFFER;
    const BINDING_STAGE: propellant::renderer::BindingStage = propellant::renderer::BindingStage::FragmentVertex;
    const DESCRIPTOR_COUNT: u32 = 1;

    type CreateError = ash_vk::Result;

    fn create(context: &propellant::renderer::RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        let capacity = std::num::NonZeroUsize::new(SEGMENT_INSTANCE_CAPACITY).unwrap();
        Ok(Self {
            instance_count: 0,
            instance_buffer: RingInstanceBuffer::create(context.device, capacity)?,
        })
    }

    fn update(&mut self, context: &propellant::renderer::RenderPassRenderContext) -> Result<(), ash_vk::Result> {
        use crate::components::SegmentRenderer;
        use propellant::components::Transform;

        let instance_buffer = self.instance_buffer.get_mut(context.frame_index);
        self.instance_count = 0;

        let mut query = context.world.query::<(&Transform, &SegmentRenderer)>();
        for (_entity, (transform, segment_renderer)) in query.iter() {
            if self.instance_count >= SEGMENT_INSTANCE_CAPACITY {
                log::warn!("Segment instance buffer full ({SEGMENT_INSTANCE_CAPACITY}), skipping remaining segments");
                break;
            }
            let Some(segment) = context.assets.get_asset(&segment_renderer.handle) else {
                log::warn!(
                    "Dangling segment handle {:?}, unable to render entity",
                    segment_renderer.handle
                );
                continue;
            };
            /* Fixme: culling */
            let instance_data = SegmentInstanceData::from_segment_components(transform, segment);
            instance_buffer.write(self.instance_count, &instance_data);
            self.instance_count += 1;
        }
        Ok(())
    }

    fn descriptor_buffer_info(&self, frame_index: usize) -> ash_vk::DescriptorBufferInfo {
        self.instance_buffer.get(frame_index).descriptor_info()
    }
}

pub struct CsgBufferBinding {
    /// Captured at creation: the segment buffer belongs to the asset manager.
    buffer_info: ash_vk::DescriptorBufferInfo,
}

impl propellant::renderer::Binding for CsgBufferBinding {
    const NAME: &'static str = "Csg Storage Buffer";

    const DESCRIPTOR_TYPE: ash_vk::DescriptorType = ash_vk::DescriptorType::STORAGE_BUFFER;
    const BINDING_STAGE: propellant::renderer::BindingStage = propellant::renderer::BindingStage::Fragment;
    const DESCRIPTOR_COUNT: u32 = 1;

    type CreateError = MissingSegmentsAssetsError;

    fn create(context: &propellant::renderer::RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        let collection = context.assets.get_collection::<crate::assets::SegmentAsset>();
        let collection = collection.ok_or(MissingSegmentsAssetsError)?;
        let buffer_info = collection.storage().buffer_info();
        Ok(Self { buffer_info })
    }

    fn update(&mut self, _context: &propellant::renderer::RenderPassRenderContext) -> Result<(), ash_vk::Result> {
        Ok(())
    }

    fn descriptor_buffer_info(&self, _frame_index: usize) -> ash_vk::DescriptorBufferInfo {
        // same buffer for every frame in flight
        self.buffer_info
    }
}

#[derive(Debug)]
pub struct MissingSegmentsAssetsError;

impl std::fmt::Display for MissingSegmentsAssetsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing segments assets in asset manager")
    }
}

impl std::error::Error for MissingSegmentsAssetsError {}
