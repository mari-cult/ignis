use bevy::math::UVec2;
use bevy::render::camera::ManualTextureView;
use bevy::render::render_resource::{Texture, TextureView};
use bevy::render::renderer::RenderDevice;
use smithay::backend::allocator::gbm::GbmBuffer;
use smithay::reexports::{drm, gbm};
use wgpu::hal::{self, api::Vulkan as Api};

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("Invalid GBM descriptor: {0}")]
    InvalidFd(#[from] gbm::InvalidFdError),

    #[error("Device error: {0}")]
    Device(hal::DeviceError),
}

pub fn import_texture(
    device: &RenderDevice,
    buffer: &GbmBuffer,
) -> Result<(Texture, ManualTextureView), ImportError> {
    let device = device.wgpu_device();

    let plane = 0;
    let (width, height) = drm::buffer::Buffer::size(buffer);
    let fd = buffer.fd_for_plane(plane)?;
    let modifier = u64::from(buffer.modifier());
    let offset = u64::from(buffer.offset(plane));
    let stride = u64::from(buffer.stride_for_plane(plane));

    let label = None;
    let mip_level_count = 1;
    let sample_count = 1;
    let dimension = wgpu::TextureDimension::D2;
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = unsafe {
        device
            .as_hal::<Api, _, _>(|device| {
                let device = device.unwrap();

                device.texture_from_dmabuf(
                    fd,
                    modifier,
                    offset,
                    stride,
                    &wgpu::hal::TextureDescriptor {
                        label,
                        size,
                        mip_level_count,
                        sample_count,
                        dimension,
                        format,
                        usage: hal::TextureUses::COLOR_TARGET,
                        memory_flags: hal::MemoryFlags::PREFER_COHERENT,
                        view_formats: vec![],
                    },
                )
            })
            .unwrap()
            .unwrap()
    };

    let texture: Texture = unsafe {
        device
            .create_texture_from_hal::<Api>(
                texture,
                &wgpu::TextureDescriptor {
                    label,
                    size,
                    mip_level_count,
                    sample_count,
                    dimension,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
            )
            .into()
    };

    let texture_view: TextureView = texture
        .create_view(&wgpu::TextureViewDescriptor::default())
        .into();

    let size = UVec2::new(width, height);

    let manual_texture_view = ManualTextureView {
        texture_view: TextureView::from(texture_view),
        size,
        format,
    };

    Ok((texture, manual_texture_view))
}
