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
    const NAME: &'static str;
    const DESCRIPTOR_TYPE: ash::vk::DescriptorType;
    const DESCRIPTOR_COUNT: u32;
    const BINDING_STAGE: BindingStage;

    type CreateError: std::error::Error;

    fn create(context: &RenderPassCreateContext) -> Result<Self, Self::CreateError>;
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
    type CreateError: std::error::Error;

    fn create(vk_device: &RenderPassCreateContext) -> Result<Self, Self::CreateError>;
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
    type CreateError = BindingsCreateError<A, B, C>;

    fn count_of_type(desc_type: ash::vk::DescriptorType) -> u32 {
        let mut count = 0;
        if A::DESCRIPTOR_TYPE == desc_type {
            count += A::DESCRIPTOR_COUNT;
        }
        if B::DESCRIPTOR_TYPE == desc_type {
            count += B::DESCRIPTOR_COUNT;
        }
        if C::DESCRIPTOR_TYPE == desc_type {
            count += C::DESCRIPTOR_COUNT;
        }
        count
    }

    fn create(vk_device: &RenderPassCreateContext) -> Result<Self, Self::CreateError> {
        Ok((
            A::create(vk_device).map_err(BindingsCreateError::A)?,
            B::create(vk_device).map_err(BindingsCreateError::B)?,
            C::create(vk_device).map_err(BindingsCreateError::C)?,
        ))
    }

    fn update(&mut self, context: &RenderPassRenderContext) -> ash::prelude::VkResult<()> {
        self.0.update(context)?;
        self.1.update(context)?;
        self.2.update(context)?;
        Ok(())
    }

    fn desc_set_layout_bindings() -> impl Iterator<Item = ash::vk::DescriptorSetLayoutBinding<'static>> {
        [A::binding(0), B::binding(1), C::binding(2)].into_iter()
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

pub enum BindingsCreateError<A: Binding, B: Binding, C: Binding> {
    A(A::CreateError),
    B(B::CreateError),
    C(C::CreateError),
}

impl<A: Binding, B: Binding, C: Binding> std::fmt::Debug for BindingsCreateError<A, B, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(err) => write!(f, "BindingsCreateError::A({err})"),
            Self::B(err) => write!(f, "BindingsCreateError::B({err})"),
            Self::C(err) => write!(f, "BindingsCreateError::C({err})"),
        }
    }
}

impl<A: Binding, B: Binding, C: Binding> std::fmt::Display for BindingsCreateError<A, B, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(err) => write!(f, "Binding \"{}\" error: {}", A::NAME, err),
            Self::B(err) => write!(f, "Binding \"{}\" error: {}", B::NAME, err),
            Self::C(err) => write!(f, "Binding \"{}\" error: {}", C::NAME, err),
        }
    }
}

impl<A: Binding, B: Binding, C: Binding> std::error::Error for BindingsCreateError<A, B, C> {}
