use crate::renderer::render_pass::RenderPassCreateContext;
use crate::renderer::render_pass::RenderPassRenderContext;

#[derive(Debug, Clone, Copy)]
pub enum BindingStage {
    Fragment,
    FragmentVertex,
    Vertex,
}

impl BindingStage {
    fn to_flags(self) -> ash::vk::ShaderStageFlags {
        match self {
            Self::Fragment => ash::vk::ShaderStageFlags::FRAGMENT,
            Self::FragmentVertex => ash::vk::ShaderStageFlags::FRAGMENT | ash::vk::ShaderStageFlags::VERTEX,
            Self::Vertex => ash::vk::ShaderStageFlags::VERTEX,
        }
    }
}

pub trait Binding: Sized {
    const DESCRIPTOR_TYPE: ash::vk::DescriptorType;
    const DESCRIPTOR_COUNT: u32 = 1;
    const BINDING_STAGE: BindingStage;

    fn create(vk_device: &RenderPassCreateContext) -> ash::prelude::VkResult<Self>;
    fn update(&mut self, context: &RenderPassRenderContext) -> ash::prelude::VkResult<()>;
    fn descriptor_buffer_info(&self, frame_index: usize) -> ash::vk::DescriptorBufferInfo;
    fn binding(binding: u32) -> ash::vk::DescriptorSetLayoutBinding<'static> {
        ash::vk::DescriptorSetLayoutBinding::default()
            .binding(binding)
            .stage_flags(Self::BINDING_STAGE.to_flags())
            .descriptor_type(Self::DESCRIPTOR_TYPE)
            .descriptor_count(Self::DESCRIPTOR_COUNT)
    }
}

pub trait Bindings: Sized {
    fn create(vk_device: &RenderPassCreateContext) -> ash::prelude::VkResult<Self>;
    fn update(&mut self, context: &RenderPassRenderContext) -> ash::prelude::VkResult<()>;
    fn count_of_type(desc_type: ash::vk::DescriptorType) -> u32;
    fn desc_set_layout_bindings() -> impl Iterator<Item = ash::vk::DescriptorSetLayoutBinding<'static>>;
    fn descriptor_buffer_infos(
        &self,
        frame_index: usize,
    ) -> impl Iterator<Item = (ash::vk::DescriptorBufferInfo, ash::vk::DescriptorType)>;
}

/* Example impl for testing current design */
impl<A: Binding, B: Binding, C: Binding> Bindings for (A, B, C) {
    fn count_of_type(desc_type: ash::vk::DescriptorType) -> u32 {
        [
            if A::DESCRIPTOR_TYPE == desc_type { 1 } else { 0 },
            if B::DESCRIPTOR_TYPE == desc_type { 1 } else { 0 },
            if C::DESCRIPTOR_TYPE == desc_type { 1 } else { 0 },
        ]
        .into_iter()
        .sum()
    }

    fn create(vk_device: &RenderPassCreateContext) -> ash::prelude::VkResult<Self> {
        Ok((A::create(vk_device)?, B::create(vk_device)?, C::create(vk_device)?))
    }

    fn update(&mut self, context: &RenderPassRenderContext) -> ash::prelude::VkResult<()> {
        self.0.update(context)?;
        self.1.update(context)?;
        self.2.update(context)?;
        Ok(())
    }

    fn desc_set_layout_bindings() -> impl Iterator<Item = ash::vk::DescriptorSetLayoutBinding<'static>> {
        [A::binding(0), B::binding(1), C::binding(1)].into_iter()
    }

    fn descriptor_buffer_infos(
        &self,
        frame_index: usize,
    ) -> impl Iterator<Item = (ash::vk::DescriptorBufferInfo, ash::vk::DescriptorType)> {
        [
            (A::descriptor_buffer_info(&self.0, frame_index), A::DESCRIPTOR_TYPE),
            (B::descriptor_buffer_info(&self.1, frame_index), B::DESCRIPTOR_TYPE),
            (C::descriptor_buffer_info(&self.2, frame_index), C::DESCRIPTOR_TYPE),
        ]
        .into_iter()
    }
}
