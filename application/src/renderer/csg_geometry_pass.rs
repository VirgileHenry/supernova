use propellant::renderer::shaders::ShaderSource;

pub struct CsgGeometryPass {}

impl propellant::renderer::RenderPass for CsgGeometryPass {
    const NAME: &'static str = "Csg Geometry Render Pass";
    type Input = propellant::renderer::RenderGraphInput;
    type Output = propellant::renderer::RenderGraphOutput;

    const FRAG_SHADER: ShaderSource = propellant::frag_shader!("shaders/segment.frag.glsl");
    const VERT_SHADER: ShaderSource = propellant::vert_shader!("shaders/segment.vert.glsl");

    type Bindings = (CameraUniformBinding,);

    fn create() -> Self {
        Self {}
    }

    fn update_uniforms(&mut self, context: &RenderPassRenderContext, bindings: &mut Self::Bindings) -> VkResult<()> {
        /* Write the uniforms */
        let camera_uniform = &bindings.0;
    }

    fn render(
        &mut self,
        vk_device: &propellant::vulkan::VkDeviceHandle,
        assets: &propellant::assets::AssetManager,
        world: &hecs::World,
        input: Self::Input,
    ) -> ash::prelude::VkResult<Self::Output> {
        /* Missing the loop + draw */
    }
}

/* Missing the 2 others, yeah */
struct CameraUniformBinding {
    camera_uniform_buffer: propellant::vulkan::RingUniformBuffer<crate::renderer::uniforms::CameraUniform>,
}

impl propellant::renderer::Binding for CameraUniformBinding {
    const DESCRIPTOR_TYPE: ash::vk::DescriptorType = propellant::vulkan::ash_vk::DescriptorType::UNIFORM_BUFFER;
    const BINDING_STAGE: BindingStage = propellant::renderer::BindingStage::FragmentVertex;

    fn create(vk_device: &propellant::vulkan::VkDeviceHandle) -> ash::prelude::VkResult<Self> {}

    fn update(&mut self, context: &RenderPassRenderContext) -> ash::prelude::VkResult<()> {}

    fn descriptor_buffer_info(&self, frame_index: usize) -> ash::vk::DescriptorBufferInfo {
        self.camera_uniform_buffer.get(frame_index).descriptor_info()
    }
}
