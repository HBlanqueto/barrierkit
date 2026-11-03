//! Decoded video frames as dmabufs, imported into the kit's Vulkan device so Skia
//! samples them in place: the GPU decodes, the GPU composites, the CPU copies nothing.

use std::cell::RefCell;
use std::os::fd::{AsRawFd, BorrowedFd, IntoRawFd};
use std::rc::{Rc, Weak};

use ash::vk::{self as ash_vk, Handle};
use freya_engine::prelude::vk;
use skia_safe::gpu::{BackendTexture, backend_textures};

use super::vulkan::GraphicsShared;

thread_local! {
    static CURRENT: RefCell<Weak<GraphicsShared>> = const { RefCell::new(Weak::new()) };
}

pub(crate) fn set_current(shared: &Rc<GraphicsShared>) {
    CURRENT.with(|current| *current.borrow_mut() = Rc::downgrade(shared));
}

pub(crate) fn current() -> Option<Rc<GraphicsShared>> {
    CURRENT.with(|current| current.borrow().upgrade())
}

const DRM_FORMAT_ABGR8888: u32 = u32::from_le_bytes(*b"AB24");
const DRM_FORMAT_XBGR8888: u32 = u32::from_le_bytes(*b"XB24");
const DRM_FORMAT_ARGB8888: u32 = u32::from_le_bytes(*b"AR24");
const DRM_FORMAT_XRGB8888: u32 = u32::from_le_bytes(*b"XR24");

pub(crate) fn vk_format(fourcc: u32) -> Option<ash_vk::Format> {
    match fourcc {
        DRM_FORMAT_ABGR8888 | DRM_FORMAT_XBGR8888 => Some(ash_vk::Format::R8G8B8A8_UNORM),
        DRM_FORMAT_ARGB8888 | DRM_FORMAT_XRGB8888 => Some(ash_vk::Format::B8G8R8A8_UNORM),
        _ => None,
    }
}

pub(crate) struct Plane {
    pub(crate) offset: u64,
    pub(crate) stride: u64,
}

pub(crate) struct Imported {
    shared: Rc<GraphicsShared>,
    image: ash_vk::Image,
    memory: ash_vk::DeviceMemory,
    pub(crate) texture: BackendTexture,
}

impl GraphicsShared {
    pub(crate) fn import_dmabuf(
        self: &Rc<Self>,
        fd: BorrowedFd<'_>,
        (width, height): (u32, u32),
        fourcc: u32,
        modifier: u64,
        planes: &[Plane],
    ) -> Option<Imported> {
        let dmabuf = self.dmabuf.as_ref()?;
        let format = vk_format(fourcc)?;
        let layouts: Vec<ash_vk::SubresourceLayout> = planes
            .iter()
            .map(|plane| ash_vk::SubresourceLayout {
                offset: plane.offset,
                row_pitch: plane.stride,
                ..Default::default()
            })
            .collect();
        let mut explicit = ash_vk::ImageDrmFormatModifierExplicitCreateInfoEXT::default()
            .drm_format_modifier(modifier)
            .plane_layouts(&layouts);
        let mut external = ash_vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(ash_vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let info = ash_vk::ImageCreateInfo::default()
            .image_type(ash_vk::ImageType::TYPE_2D)
            .format(format)
            .extent(ash_vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(ash_vk::SampleCountFlags::TYPE_1)
            .tiling(ash_vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(ash_vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(ash_vk::SharingMode::EXCLUSIVE)
            .initial_layout(ash_vk::ImageLayout::UNDEFINED)
            .push_next(&mut external)
            .push_next(&mut explicit);
        // SAFETY: the device is alive for `self`; the create info borrows locals only.
        let image = unsafe { self.device.create_image(&info, None) }.ok()?;

        let owned = fd.try_clone_to_owned().ok()?;
        let mut fd_properties = ash_vk::MemoryFdPropertiesKHR::default();
        // SAFETY: `owned` is a live dmabuf descriptor; the struct outlives the call.
        let queried = unsafe {
            dmabuf.fd.get_memory_fd_properties(
                ash_vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
                owned.as_raw_fd(),
                &mut fd_properties,
            )
        };
        // SAFETY: `image` was created on this device just above.
        let requirements = unsafe { self.device.get_image_memory_requirements(image) };
        let bits = requirements.memory_type_bits & fd_properties.memory_type_bits;
        let Some(memory_type) = (queried.is_ok() && bits != 0).then(|| bits.trailing_zeros())
        else {
            // SAFETY: nothing else references the image yet.
            unsafe { self.device.destroy_image(image, None) };
            return None;
        };
        let mut import = ash_vk::ImportMemoryFdInfoKHR::default()
            .handle_type(ash_vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
            .fd(owned.as_raw_fd());
        let mut dedicated = ash_vk::MemoryDedicatedAllocateInfo::default().image(image);
        let allocate = ash_vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type)
            .push_next(&mut import)
            .push_next(&mut dedicated);
        // SAFETY: the import struct names a live dmabuf; on success Vulkan owns the descriptor.
        let memory = match unsafe { self.device.allocate_memory(&allocate, None) } {
            Ok(memory) => {
                let _ = owned.into_raw_fd();
                memory
            }
            Err(_) => {
                // SAFETY: nothing else references the image yet.
                unsafe { self.device.destroy_image(image, None) };
                return None;
            }
        };
        // SAFETY: image and memory belong to this device and are unbound.
        if unsafe { self.device.bind_image_memory(image, memory, 0) }.is_err() {
            // SAFETY: neither handle is used anywhere else.
            unsafe {
                self.device.destroy_image(image, None);
                self.device.free_memory(memory, None);
            }
            return None;
        }

        // GENERAL keeps the decoder's pixels: a transition out of UNDEFINED may discard them.
        // SAFETY: `image` and `memory` live in `Imported`, which outlives every Skia image made from it.
        let mut skia_info = unsafe {
            vk::ImageInfo::new(
                image.as_raw() as _,
                vk::Alloc::default(),
                vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT,
                vk::ImageLayout::GENERAL,
                match format {
                    ash_vk::Format::R8G8B8A8_UNORM => vk::Format::R8G8B8A8_UNORM,
                    _ => vk::Format::B8G8R8A8_UNORM,
                },
                1,
                dmabuf
                    .foreign_queue
                    .then_some(ash_vk::QUEUE_FAMILY_FOREIGN_EXT),
                None,
                None,
                None,
            )
        };
        skia_info.image_usage_flags = ash_vk::ImageUsageFlags::SAMPLED.as_raw();
        // SAFETY: `skia_info` describes the live image above.
        let texture = unsafe {
            backend_textures::make_vk((width as i32, height as i32), &skia_info, "video")
        };
        Some(Imported {
            shared: self.clone(),
            image,
            memory,
            texture,
        })
    }
}

impl Drop for Imported {
    fn drop(&mut self) {
        self.shared.finish_gpu_work();
        // SAFETY: the GPU is idle, so no submitted work still samples the image.
        unsafe {
            self.shared.device.destroy_image(self.image, None);
            self.shared.device.free_memory(self.memory, None);
        }
    }
}
