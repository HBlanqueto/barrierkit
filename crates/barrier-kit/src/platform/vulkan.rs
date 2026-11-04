use super::RunError;
use ash::{
    Device, Entry, Instance,
    khr::{surface, swapchain, wayland_surface},
    vk::{self as ash_vk, Handle},
};
use freya_engine::prelude::{
    ColorType, DirectContext, FontCollection, FontMgr, Surface as SkiaSurface, SurfaceOrigin,
    backend_render_targets, direct_contexts, vk, wrap_backend_render_target,
};
use sctk::reexports::client::{Proxy, protocol::wl_surface::WlSurface};
use skia_safe::{gpu::FlushInfo, surfaces::BackendSurfaceAccess};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    ffi::{CStr, c_void},
    mem::ManuallyDrop,
    rc::Rc,
    time::Duration,
};

const INSTANCE_EXTENSIONS: [&CStr; 2] = [surface::NAME, wayland_surface::NAME];
const DEVICE_EXTENSIONS: [&CStr; 1] = [swapchain::NAME];
const SURFACE_MAINTENANCE_EXTENSIONS: [&CStr; 2] = [
    ash::khr::get_surface_capabilities2::NAME,
    ash::ext::surface_maintenance1::NAME,
];
#[cfg(feature = "video")]
const DMABUF_EXTENSIONS: [&CStr; 4] = [
    ash::khr::external_memory_fd::NAME,
    ash::ext::external_memory_dma_buf::NAME,
    ash::ext::image_drm_format_modifier::NAME,
    ash::khr::image_format_list::NAME,
];

#[cfg(feature = "video")]
pub(crate) struct DmabufDevice {
    pub(crate) fd: ash::khr::external_memory_fd::Device,
    pub(crate) foreign_queue: bool,
}
const SWAPCHAIN_FORMAT: ash_vk::Format = ash_vk::Format::B8G8R8A8_UNORM;

fn failed(call: &'static str) -> impl Fn(ash_vk::Result) -> RunError {
    move |_| RunError::Vulkan(call)
}

pub(crate) struct GraphicsShared {
    _entry: Entry,
    instance: Instance,
    display: *mut c_void,
    physical_device: ash_vk::PhysicalDevice,
    pub(crate) device: Device,
    #[cfg(feature = "video")]
    pub(crate) dmabuf: Option<DmabufDevice>,
    queue: ash_vk::Queue,
    surface_fns: surface::Instance,
    wayland_fns: wayland_surface::Instance,
    swapchain_fns: swapchain::Device,
    gr: RefCell<ManuallyDrop<DirectContext>>,
    pub(crate) font_collection: FontCollection,
    pub(crate) font_manager: FontMgr,
    pub(crate) swapchain_creations: Cell<usize>,
    present_fences: bool,
}

impl GraphicsShared {
    pub(crate) fn new(display: *mut c_void) -> Result<Self, RunError> {
        // SAFETY: libvulkan is the system loader; its initialisers have no preconditions.
        let entry = unsafe { Entry::load() }
            .map_err(|_| RunError::Vulkan("libvulkan could not be loaded"))?;
        // SAFETY: `entry` holds a loaded libvulkan.
        let api_version = unsafe { entry.try_enumerate_instance_version() }
            .ok()
            .flatten()
            .unwrap_or(ash_vk::API_VERSION_1_0);
        let app_info = ash_vk::ApplicationInfo::default().api_version(api_version);
        // SAFETY: `entry` holds a loaded libvulkan.
        let instance_available: Vec<_> =
            unsafe { entry.enumerate_instance_extension_properties(None) }
                .unwrap_or_default()
                .iter()
                .filter_map(|ext| ext.extension_name_as_c_str().ok().map(CStr::to_owned))
                .collect();
        let surface_maintenance = api_version >= ash_vk::API_VERSION_1_1
            && SURFACE_MAINTENANCE_EXTENSIONS
                .iter()
                .all(|name| instance_available.iter().any(|ext| ext.as_c_str() == *name));
        let mut instance_extensions = INSTANCE_EXTENSIONS.to_vec();
        if surface_maintenance {
            instance_extensions.extend(SURFACE_MAINTENANCE_EXTENSIONS);
        }
        let extensions: Vec<_> = instance_extensions
            .iter()
            .map(|name| name.as_ptr())
            .collect();
        // SAFETY: the create info only borrows locals that outlive the call.
        let instance = unsafe {
            entry.create_instance(
                &ash_vk::InstanceCreateInfo::default()
                    .application_info(&app_info)
                    .enabled_extension_names(&extensions),
                None,
            )
        }
        .map_err(failed("vkCreateInstance"))?;
        let surface_fns = surface::Instance::new(&entry, &instance);
        let wayland_fns = wayland_surface::Instance::new(&entry, &instance);
        let (physical_device, queue_family_index) = pick_device(&instance, &wayland_fns, display)
            .ok_or(RunError::Vulkan(
            "no GPU can present to this Wayland display",
        ))?;

        let priorities = [1.0];
        let queue_info = ash_vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);
        #[cfg(feature = "video")]
        let (dmabuf_supported, foreign_queue) = {
            // SAFETY: `physical_device` comes from `instance`.
            let available: Vec<_> =
                unsafe { instance.enumerate_device_extension_properties(physical_device) }
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|ext| ext.extension_name_as_c_str().ok().map(CStr::to_owned))
                    .collect();
            let has = |name: &CStr| available.iter().any(|ext| ext.as_c_str() == name);
            (
                DMABUF_EXTENSIONS.iter().all(|name| has(name)),
                has(ash::ext::queue_family_foreign::NAME),
            )
        };
        #[cfg(not(feature = "video"))]
        let enabled: Vec<&CStr> = DEVICE_EXTENSIONS.to_vec();
        #[cfg(feature = "video")]
        let enabled: Vec<&CStr> = {
            let mut enabled = DEVICE_EXTENSIONS.to_vec();
            if dmabuf_supported {
                enabled.extend(DMABUF_EXTENSIONS);
                if foreign_queue {
                    enabled.push(ash::ext::queue_family_foreign::NAME);
                }
            }
            enabled
        };
        let present_fences =
            surface_maintenance && supports_present_fences(&instance, physical_device);
        let mut enabled = enabled;
        if present_fences {
            enabled.push(ash::ext::swapchain_maintenance1::NAME);
        }
        let extensions: Vec<_> = enabled.iter().map(|name| name.as_ptr()).collect();
        let mut maintenance = ash_vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default()
            .swapchain_maintenance1(true);
        let mut device_info = ash_vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions);
        if present_fences {
            device_info = device_info.push_next(&mut maintenance);
        }
        // SAFETY: `physical_device` comes from `instance`; the create info borrows locals.
        let device = unsafe { instance.create_device(physical_device, &device_info, None) }
            .map_err(failed("vkCreateDevice"))?;
        // SAFETY: the device was created with one queue in `queue_family_index`.
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let swapchain_fns = swapchain::Device::new(&instance, &device);

        let get_proc = |of: vk::GetProcOf| {
            // SAFETY: Skia asks only for handles it received from this instance and device.
            unsafe {
                match of {
                    vk::GetProcOf::Instance(handle, name) => {
                        entry.get_instance_proc_addr(ash_vk::Instance::from_raw(handle as _), name)
                    }
                    vk::GetProcOf::Device(handle, name) => {
                        (instance.fp_v1_0().get_device_proc_addr)(
                            ash_vk::Device::from_raw(handle as _),
                            name,
                        )
                    }
                }
            }
            .map_or(std::ptr::null(), |f| f as _)
        };
        let instance_extensions = INSTANCE_EXTENSIONS.map(|name| name.to_str().unwrap_or_default());
        let device_extensions: Vec<_> = enabled
            .iter()
            .map(|name| name.to_str().unwrap_or_default())
            .collect();
        // SAFETY: instance, device and queue live in `Self` and outlive the Skia context.
        let mut backend = unsafe {
            vk::BackendContext::new_with_extensions(
                instance.handle().as_raw() as _,
                physical_device.as_raw() as _,
                device.handle().as_raw() as _,
                (queue.as_raw() as _, queue_family_index as usize),
                &get_proc,
                &instance_extensions,
                &device_extensions,
            )
        };
        backend.set_max_api_version(vk::Version::new(
            ash_vk::api_version_major(api_version) as usize,
            ash_vk::api_version_minor(api_version) as usize,
            ash_vk::api_version_patch(api_version) as usize,
        ));
        let mut gr = direct_contexts::make_vulkan(&backend, None).ok_or(RunError::Skia)?;
        drop(backend);
        gr.set_resource_cache_limit(128 * 1024 * 1024);
        #[cfg(feature = "video")]
        let dmabuf = dmabuf_supported.then(|| DmabufDevice {
            fd: ash::khr::external_memory_fd::Device::new(&instance, &device),
            foreign_queue,
        });
        #[cfg(feature = "video")]
        if dmabuf.is_none() {
            tracing::info!(
                "vídeo: el dispositivo Vulkan no importa dmabuf; los fotogramas se suben desde la CPU"
            );
        }

        Ok(Self {
            _entry: entry,
            instance,
            display,
            physical_device,
            device,
            #[cfg(feature = "video")]
            dmabuf,
            queue,
            surface_fns,
            wayland_fns,
            swapchain_fns,
            gr: RefCell::new(ManuallyDrop::new(gr)),
            font_collection: crate::fonts::collection(),
            font_manager: crate::fonts::manager(),
            swapchain_creations: Cell::new(0),
            present_fences,
        })
    }
}

// Skia keeps up to its cache limit of GPU memory alive; anything idle this long is released.
pub(crate) const IDLE_RESOURCE_RELEASE: Duration = Duration::from_secs(10);

impl GraphicsShared {
    pub(crate) fn release_idle_resources(&self) {
        self.gr
            .borrow_mut()
            .perform_deferred_cleanup(IDLE_RESOURCE_RELEASE, None);
    }

    #[cfg(feature = "video")]
    pub(crate) fn finish_gpu_work(&self) {
        self.gr.borrow_mut().flush_and_submit();
        // SAFETY: the device is alive for `self`.
        let _ = unsafe { self.device.device_wait_idle() };
    }
}

fn supports_present_fences(instance: &Instance, device: ash_vk::PhysicalDevice) -> bool {
    // SAFETY: `device` comes from `instance`.
    let has_extension = unsafe { instance.enumerate_device_extension_properties(device) }
        .unwrap_or_default()
        .iter()
        .any(|ext| ext.extension_name_as_c_str() == Ok(ash::ext::swapchain_maintenance1::NAME));
    if !has_extension {
        return false;
    }
    let mut maintenance = ash_vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default();
    let mut features = ash_vk::PhysicalDeviceFeatures2::default().push_next(&mut maintenance);
    // SAFETY: the instance is Vulkan 1.1, so vkGetPhysicalDeviceFeatures2 exists.
    unsafe { instance.get_physical_device_features2(device, &mut features) };
    maintenance.swapchain_maintenance1 == ash_vk::TRUE
}

// Mesa's device-select layer lists first the GPU the Wayland compositor uses:
// https://gitlab.freedesktop.org/mesa/mesa/-/tree/main/src/vulkan/device-select-layer
fn pick_device(
    instance: &Instance,
    wayland_fns: &wayland_surface::Instance,
    display: *mut c_void,
) -> Option<(ash_vk::PhysicalDevice, u32)> {
    // SAFETY: `instance` is alive; `display` is the connection's live wl_display.
    unsafe { instance.enumerate_physical_devices() }
        .ok()?
        .into_iter()
        .find_map(|device| {
            // SAFETY: `device` was enumerated from `instance`.
            let families = unsafe { instance.get_physical_device_queue_family_properties(device) };
            families
                .iter()
                .enumerate()
                .find(|&(index, family)| {
                    family.queue_flags.contains(ash_vk::QueueFlags::GRAPHICS)
                        // SAFETY: `display` is the connection's live wl_display.
                        && unsafe {
                            wayland_fns.get_physical_device_wayland_presentation_support(
                                device,
                                index as u32,
                                &mut *display.cast(),
                            )
                        }
                })
                .map(|(index, _)| (device, index as u32))
        })
}

impl Drop for GraphicsShared {
    fn drop(&mut self) {
        // SAFETY: no swapchain outlives this struct (each `Graphics` holds an `Rc` to it);
        // Skia is dropped before the device it uses.
        unsafe {
            let _ = self.device.device_wait_idle();
            ManuallyDrop::drop(self.gr.get_mut());
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

pub(crate) struct Graphics {
    shared: Rc<GraphicsShared>,
    surface: ash_vk::SurfaceKHR,
    swapchain: ash_vk::SwapchainKHR,
    images: Vec<ash_vk::Image>,
    targets: Vec<SkiaSurface>,
    rendered: Vec<ash_vk::Semaphore>,
    acquired: ash_vk::Fence,
    presents: VecDeque<ash_vk::Fence>,
    spare_fences: Vec<ash_vk::Fence>,
    retired: Vec<Retired>,
    image_index: u32,
    pub(crate) skia: Option<SkiaSurface>,
    size: (u32, u32),
}

struct Retired {
    swapchain: ash_vk::SwapchainKHR,
    targets: Vec<SkiaSurface>,
    rendered: Vec<ash_vk::Semaphore>,
    presents: VecDeque<ash_vk::Fence>,
}

impl Graphics {
    pub(crate) fn new(shared: Rc<GraphicsShared>) -> Self {
        Self {
            shared,
            surface: ash_vk::SurfaceKHR::null(),
            swapchain: ash_vk::SwapchainKHR::null(),
            images: Vec::new(),
            targets: Vec::new(),
            rendered: Vec::new(),
            acquired: ash_vk::Fence::null(),
            presents: VecDeque::new(),
            spare_fences: Vec::new(),
            retired: Vec::new(),
            image_index: 0,
            skia: None,
            size: (0, 0),
        }
    }

    pub(crate) fn configure(
        &mut self,
        surface: &WlSurface,
        size: (u32, u32),
    ) -> Result<(), RunError> {
        if size.0 == 0 || size.1 == 0 {
            return Ok(());
        }
        if self.surface.is_null() {
            self.create_surface(surface)?;
        }
        if self.swapchain.is_null() || self.size != size {
            self.recreate_swapchain(size)?;
        }
        Ok(())
    }

    fn create_surface(&mut self, surface: &WlSurface) -> Result<(), RunError> {
        let shared = &self.shared;
        let info = ash_vk::WaylandSurfaceCreateInfoKHR::default()
            .display(shared.display.cast())
            .surface(surface.id().as_ptr().cast());
        // SAFETY: display and wl_surface are alive while this `Graphics` lives.
        self.surface = unsafe { shared.wayland_fns.create_wayland_surface(&info, None) }
            .map_err(failed("vkCreateWaylandSurfaceKHR"))?;
        // SAFETY: `device` is alive; the create info has no pointers.
        self.acquired = unsafe {
            shared
                .device
                .create_fence(&ash_vk::FenceCreateInfo::default(), None)
        }
        .map_err(failed("vkCreateFence"))?;
        Ok(())
    }

    fn recreate_swapchain(&mut self, size: (u32, u32)) -> Result<(), RunError> {
        let shared = self.shared.clone();
        // SAFETY: the surface belongs to `physical_device`'s instance.
        let capabilities = unsafe {
            shared
                .surface_fns
                .get_physical_device_surface_capabilities(shared.physical_device, self.surface)
        }
        .map_err(failed("vkGetPhysicalDeviceSurfaceCapabilitiesKHR"))?;
        // SAFETY: as above.
        let present_modes = unsafe {
            shared
                .surface_fns
                .get_physical_device_surface_present_modes(shared.physical_device, self.surface)
        }
        .map_err(failed("vkGetPhysicalDeviceSurfacePresentModesKHR"))?;
        // Frame callbacks already pace redraws; FIFO would throttle a second time (freya#2260).
        let present_mode = if present_modes.contains(&ash_vk::PresentModeKHR::MAILBOX) {
            ash_vk::PresentModeKHR::MAILBOX
        } else {
            ash_vk::PresentModeKHR::FIFO
        };
        let wanted_images = if present_mode == ash_vk::PresentModeKHR::MAILBOX {
            3
        } else {
            2
        };
        let mut image_count = capabilities.min_image_count.max(wanted_images);
        if capabilities.max_image_count > 0 {
            image_count = image_count.min(capabilities.max_image_count);
        }
        let info = ash_vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(image_count)
            .image_format(SWAPCHAIN_FORMAT)
            .image_color_space(ash_vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(ash_vk::Extent2D {
                width: size.0,
                height: size.1,
            })
            .image_array_layers(1)
            .image_usage(
                ash_vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | ash_vk::ImageUsageFlags::SAMPLED
                    | ash_vk::ImageUsageFlags::TRANSFER_SRC
                    | ash_vk::ImageUsageFlags::TRANSFER_DST,
            )
            .image_sharing_mode(ash_vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(ash_vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(self.swapchain);
        // SAFETY: without present fences the device is idle before the old swapchain and its
        // semaphores go away; with them the old swapchain waits in `retired` for its last present.
        unsafe {
            if !shared.present_fences {
                shared
                    .device
                    .device_wait_idle()
                    .map_err(failed("vkDeviceWaitIdle"))?;
            }
            let swapchain = shared
                .swapchain_fns
                .create_swapchain(&info, None)
                .map_err(failed("vkCreateSwapchainKHR"))?;
            if shared.present_fences {
                self.retire_swapchain();
            } else {
                self.destroy_swapchain();
            }
            self.swapchain = swapchain;
            self.images = shared
                .swapchain_fns
                .get_swapchain_images(swapchain)
                .map_err(failed("vkGetSwapchainImagesKHR"))?;
            self.rendered = self
                .images
                .iter()
                .map(|_| {
                    shared
                        .device
                        .create_semaphore(&ash_vk::SemaphoreCreateInfo::default(), None)
                })
                .collect::<Result<_, _>>()
                .map_err(failed("vkCreateSemaphore"))?;
        }
        self.targets = self
            .images
            .iter()
            .map(|image| wrap_swapchain_image(&shared, *image, size))
            .collect::<Option<_>>()
            .ok_or(RunError::Vulkan("Skia could not wrap the swapchain image"))?;
        shared
            .swapchain_creations
            .set(shared.swapchain_creations.get() + 1);
        self.size = size;
        Ok(())
    }

    fn retire_swapchain(&mut self) {
        self.skia = None;
        self.images.clear();
        if self.swapchain.is_null() {
            return;
        }
        self.retired.push(Retired {
            swapchain: std::mem::replace(&mut self.swapchain, ash_vk::SwapchainKHR::null()),
            targets: std::mem::take(&mut self.targets),
            rendered: std::mem::take(&mut self.rendered),
            presents: std::mem::take(&mut self.presents),
        });
    }

    // VK_EXT_swapchain_maintenance1: present fences signal in present order, and a swapchain
    // may be destroyed once the fences of all its presents have signaled.
    fn reclaim_presented(&mut self) {
        let shared = self.shared.clone();
        // SAFETY: every fence here belongs to `device`.
        let signaled =
            |fence: &ash_vk::Fence| unsafe { shared.device.get_fence_status(*fence) } == Ok(true);
        while self.presents.front().is_some_and(signaled) {
            if let Some(fence) = self.presents.pop_front() {
                self.recycle_fence(fence);
            }
        }
        let (done, pending): (Vec<_>, Vec<_>) = std::mem::take(&mut self.retired)
            .into_iter()
            .partition(|retired| retired.presents.back().is_none_or(signaled));
        self.retired = pending;
        for retired in done {
            // SAFETY: the last present of this swapchain finished, so no queue operation
            // still uses its images or semaphores.
            for fence in unsafe { destroy_retired(&shared, retired) } {
                self.recycle_fence(fence);
            }
        }
    }

    fn recycle_fence(&mut self, fence: ash_vk::Fence) {
        // SAFETY: the fence is signaled, so no queue operation still uses it.
        match unsafe { self.shared.device.reset_fences(&[fence]) } {
            Ok(()) => self.spare_fences.push(fence),
            // SAFETY: as above.
            Err(_) => unsafe { self.shared.device.destroy_fence(fence, None) },
        }
    }

    fn present_fence(&mut self) -> Result<ash_vk::Fence, RunError> {
        if let Some(fence) = self.spare_fences.pop() {
            return Ok(fence);
        }
        // SAFETY: `device` is alive; the create info has no pointers.
        unsafe {
            self.shared
                .device
                .create_fence(&ash_vk::FenceCreateInfo::default(), None)
        }
        .map_err(failed("vkCreateFence"))
    }

    // SAFETY: callers ensure the device is idle.
    unsafe fn destroy_swapchain(&mut self) {
        self.skia = None;
        self.targets.clear();
        let shared = &self.shared;
        for semaphore in self.rendered.drain(..) {
            // SAFETY: the device is idle, so no queue operation waits on it.
            unsafe { shared.device.destroy_semaphore(semaphore, None) };
        }
        // SAFETY: the device is idle and the images are no longer acquired.
        unsafe { shared.swapchain_fns.destroy_swapchain(self.swapchain, None) };
        self.swapchain = ash_vk::SwapchainKHR::null();
        self.images.clear();
    }

    pub(crate) fn acquire(&mut self) -> Result<(), RunError> {
        if self.swapchain.is_null() {
            return Err(RunError::Vulkan("no swapchain"));
        }
        let shared = self.shared.clone();
        // SAFETY: the swapchain and the unsignalled fence belong to `device`.
        let acquired = unsafe {
            shared.swapchain_fns.acquire_next_image(
                self.swapchain,
                u64::MAX,
                ash_vk::Semaphore::null(),
                self.acquired,
            )
        };
        let index = match acquired {
            Ok((index, _)) => index,
            Err(ash_vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(self.size)?;
                return Err(RunError::Vulkan("swapchain out of date"));
            }
            Err(_) => return Err(RunError::Vulkan("vkAcquireNextImageKHR")),
        };
        // SAFETY: the fence was handed to the successful acquire above.
        unsafe {
            shared
                .device
                .wait_for_fences(&[self.acquired], true, u64::MAX)
                .and_then(|()| shared.device.reset_fences(&[self.acquired]))
        }
        .map_err(failed("vkWaitForFences"))?;
        self.image_index = index;
        self.skia = Some(self.targets[index as usize].clone());
        Ok(())
    }

    pub(crate) fn present(&mut self) -> Result<(), RunError> {
        let Some(mut skia) = self.skia.take() else {
            return Err(RunError::Vulkan("no acquired image"));
        };
        let shared = self.shared.clone();
        let mut gr = shared.gr.borrow_mut();
        gr.flush_surface_with_access(
            &mut skia,
            BackendSurfaceAccess::Present,
            &FlushInfo::default(),
        );
        gr.submit(None);
        drop(gr);
        let rendered = [self.rendered[self.image_index as usize]];
        let signal = ash_vk::SubmitInfo::default().signal_semaphores(&rendered);
        let swapchains = [self.swapchain];
        let indices = [self.image_index];
        // SAFETY: the empty submit orders after Skia's on the same queue.
        unsafe {
            shared
                .device
                .queue_submit(shared.queue, &[signal], ash_vk::Fence::null())
                .map_err(failed("vkQueueSubmit"))?;
        }
        self.reclaim_presented();
        let present_fence = if shared.present_fences {
            Some(self.present_fence()?)
        } else {
            None
        };
        let fences = [present_fence.unwrap_or_default()];
        let mut fence_info = ash_vk::SwapchainPresentFenceInfoEXT::default().fences(&fences);
        let mut present_info = ash_vk::PresentInfoKHR::default()
            .wait_semaphores(&rendered)
            .swapchains(&swapchains)
            .image_indices(&indices);
        if present_fence.is_some() {
            present_info = present_info.push_next(&mut fence_info);
        }
        // SAFETY: the semaphore of this image is free because its previous present finished
        // before it was acquired.
        let presented = unsafe {
            shared
                .swapchain_fns
                .queue_present(shared.queue, &present_info)
        };
        if let Some(fence) = present_fence {
            // An out-of-date present is still enqueued, so its fence signals.
            match presented {
                Ok(_) | Err(ash_vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.presents.push_back(fence)
                }
                Err(_) => self.spare_fences.push(fence),
            }
        }
        // VK_SUBOPTIMAL_KHR still presented; recreating on it loops when nothing can fix it.
        match presented {
            Ok(_) => Ok(()),
            Err(ash_vk::Result::ERROR_OUT_OF_DATE_KHR) => self.recreate_swapchain(self.size),
            Err(_) => Err(RunError::Vulkan("vkQueuePresentKHR")),
        }
    }
}

// SAFETY: callers ensure no queue operation still uses the swapchain or its semaphores.
unsafe fn destroy_retired(shared: &GraphicsShared, retired: Retired) -> VecDeque<ash_vk::Fence> {
    let Retired {
        swapchain,
        targets,
        rendered,
        presents,
    } = retired;
    drop(targets);
    for semaphore in rendered {
        // SAFETY: as the caller ensures.
        unsafe { shared.device.destroy_semaphore(semaphore, None) };
    }
    // SAFETY: as the caller ensures.
    unsafe { shared.swapchain_fns.destroy_swapchain(swapchain, None) };
    presents
}

// Skia's VulkanWindowContext keeps one SkSurface per swapchain image as well.
fn wrap_swapchain_image(
    shared: &GraphicsShared,
    image: ash_vk::Image,
    size: (u32, u32),
) -> Option<SkiaSurface> {
    // SAFETY: the image belongs to the swapchain and outlives the surface, which
    // destroy_swapchain drops first.
    let image_info = unsafe {
        vk::ImageInfo::new(
            image.as_raw() as _,
            vk::Alloc::default(),
            vk::ImageTiling::OPTIMAL,
            vk::ImageLayout::UNDEFINED,
            vk::Format::B8G8R8A8_UNORM,
            1,
            None,
            None,
            None,
            vk::SharingMode::EXCLUSIVE,
        )
    };
    let render_target =
        backend_render_targets::make_vk((size.0 as i32, size.1 as i32), &image_info);
    wrap_backend_render_target(
        &mut shared.gr.borrow_mut(),
        &render_target,
        SurfaceOrigin::TopLeft,
        ColorType::BGRA8888,
        None,
        None,
    )
}

impl Drop for Graphics {
    fn drop(&mut self) {
        self.skia = None;
        let shared = self.shared.clone();
        // SAFETY: waiting for idle first lets every handle of this window go.
        unsafe {
            let _ = shared.device.device_wait_idle();
            self.destroy_swapchain();
            let retired_fences: Vec<_> = std::mem::take(&mut self.retired)
                .into_iter()
                .flat_map(|retired| destroy_retired(&shared, retired))
                .collect();
            for fence in retired_fences
                .into_iter()
                .chain(self.presents.drain(..))
                .chain(self.spare_fences.drain(..))
            {
                shared.device.destroy_fence(fence, None);
            }
            shared.device.destroy_fence(self.acquired, None);
            shared.surface_fns.destroy_surface(self.surface, None);
        }
    }
}
