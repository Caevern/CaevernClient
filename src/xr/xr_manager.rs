use ash::vk::{self, Handle};
use openxr::ViewConfigurationType;
use wgpu_hal::Instance;

pub struct XRManager {
    pub instance: openxr::Instance,
    pub system: openxr::SystemId,
    pub session: openxr::Session<openxr::Vulkan>,
    pub frame_waiter: openxr::FrameWaiter,
    pub frame_stream: openxr::FrameStream<openxr::Vulkan>,
    pub views: Vec<openxr::ViewConfigurationView>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub view_config: ViewConfigurationType,
    pub swapchains: Vec<openxr::Swapchain<openxr::Vulkan>>,
    pub swapchain_images: Vec<Vec<vk::Image>>,
    pub swapchain_textures: Vec<Vec<wgpu::Texture>>,
    pub swapchain_views: Vec<Vec<wgpu::TextureView>>,
    pub depth_views: Vec<wgpu::TextureView>,
    pub reference_space: openxr::Space,
    event_buffer: openxr::EventDataBuffer,
    session_running: bool,
}
impl XRManager {
    pub fn new() -> Result<Self, openxr::sys::Result> {
        let entry = unsafe {
            match openxr::Entry::load() {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!("OpenXR Entry::load() failed: {err:?}");
                    return Err(openxr::sys::Result::ERROR_INITIALIZATION_FAILED);
                }
            }
        };

        let mut extensions = openxr::ExtensionSet::default();
        extensions.khr_vulkan_enable2 = true;
        #[cfg(target_os = "android")]
        {
            extensions.khr_android_create_instance = true;
        }

        let instance = entry.create_instance(
            &openxr::ApplicationInfo {
                application_name: "Caevern",
                application_version: 1,
                engine_name: "Caevern",
                engine_version: 1,
                api_version: openxr::CURRENT_API_VERSION,
            },
            &extensions,
            &[],
        )?;

        let system = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

        let properties = instance.system_properties(system)?;

        println!("HMD: {}", properties.system_name);
        println!("Vendor: {}", properties.vendor_id);
        println!(
            "Max swapchain width: {}",
            properties.graphics_properties.max_swapchain_image_width
        );
        println!(
            "Max swapchain height: {}",
            properties.graphics_properties.max_swapchain_image_height
        );

        let view_config = instance
            .enumerate_view_configurations(system)?
            .into_iter()
            .next()
            .ok_or(openxr::sys::Result::ERROR_VIEW_CONFIGURATION_TYPE_UNSUPPORTED)?;

        println!("View config: {:?}", view_config);

        let views = instance.enumerate_view_configuration_views(system, view_config)?;

        for (i, view) in views.iter().enumerate() {
            println!(
                "View {i}: {}x{} samples={}",
                view.recommended_image_rect_width,
                view.recommended_image_rect_height,
                view.recommended_swapchain_sample_count,
            );
        }

        let vk_entry = unsafe { ash::Entry::load().unwrap() };

        let vk_app_info = vk::ApplicationInfo::default()
            .application_version(0)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 2, 0));

        let vk_instance = unsafe {
            let vk_instance = instance
                .create_vulkan_instance(
                    system,
                    std::mem::transmute(vk_entry.static_fn().get_instance_proc_addr),
                    &vk::InstanceCreateInfo::default().application_info(&vk_app_info) as *const _
                        as *const _,
                )
                .expect("XR error creating Vulkan instance")
                .map_err(vk::Result::from_raw)
                .expect("Vulkan error creating Vulkan instance");
            ash::Instance::load(
                vk_entry.static_fn(),
                vk::Instance::from_raw(vk_instance as _),
            )
        };

        let requirements = instance.graphics_requirements::<openxr::Vulkan>(system)?;
        println!("Got graphics requirements");

        let enabled_extensions = unsafe {
            vk_entry
                .enumerate_instance_extension_properties(None)
                .expect("Failed to enumerate Vulkan instance extensions")
                .into_iter()
                .map(|ext| {
                    let name = std::ffi::CStr::from_ptr(ext.extension_name.as_ptr());
                    name.to_owned()
                })
                .map(|name| Box::leak(name.into_boxed_c_str()) as &'static std::ffi::CStr)
                .collect::<Vec<_>>()
        };
        println!("Got enabled extensions");

        let hal_instance = unsafe {
            wgpu_hal::vulkan::Instance::from_raw(
                vk_entry.clone(),
                vk_instance.clone(),
                vk_app_info.api_version,
                0,
                None,
                enabled_extensions,
                wgpu::InstanceFlags::empty(),
                wgpu::MemoryBudgetThresholds::default(),
                false,
                None,
            )
            .expect("Failed to create hal instance")
        };
        println!("Created hal instance");

        let hal_adapters = unsafe { hal_instance.enumerate_adapters(None) };
        let wgpu_instance =
            unsafe { wgpu::Instance::from_hal::<wgpu_hal::api::Vulkan>(hal_instance) };

        let vk_physical_device = unsafe {
            vk::PhysicalDevice::from_raw(
                instance
                    .vulkan_graphics_device(system, vk_instance.handle().as_raw() as _)
                    .unwrap() as _,
            )
        };

        let hal_adapter = hal_adapters
            .into_iter()
            .find(|adapter| adapter.adapter.raw_physical_device() == vk_physical_device)
            .expect("Failed to find hal adapter");

        let required_device_extensions = hal_adapter
            .adapter
            .required_device_extensions(wgpu::Features::empty());

        let device_extension_names: Vec<*const std::ffi::c_char> = required_device_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();

        let queue_family_index = unsafe {
            vk_instance
                .get_physical_device_queue_family_properties(vk_physical_device)
                .into_iter()
                .enumerate()
                .find_map(|(queue_family_index, info)| {
                    if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        Some(queue_family_index as u32)
                    } else {
                        None
                    }
                })
                .expect("Vulkan device has no graphics queue")
        };

        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(std::slice::from_ref(&1.0f32));
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_extension_names(&device_extension_names);

        let vk_device_raw = unsafe {
            instance
                .create_vulkan_device(
                    system,
                    std::mem::transmute(vk_entry.static_fn().get_instance_proc_addr),
                    vk_physical_device.as_raw() as _,
                    &device_create_info as *const _ as *const _,
                )
                .expect("Failed to create Vulkan device")
        };

        let vk_device = unsafe {
            ash::Device::load(
                vk_instance.fp_v1_0(),
                vk::Device::from_raw(vk_device_raw.expect("Failed to create device ID") as _),
            )
        };

        let (session, frame_waiter, frame_stream) = unsafe {
            instance.create_session::<openxr::Vulkan>(
                system,
                &openxr::vulkan::SessionCreateInfo {
                    instance: vk_instance.handle().as_raw() as _,
                    physical_device: vk_physical_device.as_raw() as _,
                    device: vk_device.handle().as_raw() as _,
                    queue_family_index,
                    queue_index: 0,
                },
            )?
        };
        println!("Created session");

        let hal_device = unsafe {
            hal_adapter
                .adapter
                .device_from_raw(
                    vk_device,
                    None,
                    &required_device_extensions,
                    wgpu::Features::empty(),
                    &wgpu::Limits::default(),
                    &wgpu::MemoryHints::default(),
                    queue_family_index,
                    0,
                )
                .expect("Failed to create hal device")
        };

        let wgpu_adapter =
            unsafe { wgpu_instance.create_adapter_from_hal::<wgpu_hal::api::Vulkan>(hal_adapter) };
        let (device, queue) = unsafe {
            wgpu_adapter.create_device_from_hal(
                hal_device,
                &wgpu::DeviceDescriptor {
                    label: Some("OpenXR Vulkan Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    experimental_features: wgpu::ExperimentalFeatures::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    trace: wgpu::Trace::Off,
                },
            )
        }
        .expect("Failed to create device from hal");

        println!(
            "OpenXR Vulkan: {} -> {}",
            requirements.min_api_version_supported, requirements.max_api_version_supported,
        );

        let reference_space = session.create_reference_space(
            openxr::ReferenceSpaceType::LOCAL,
            openxr::Posef::IDENTITY,
        )?;

        Ok(Self {
            instance: instance,
            system: system,
            session: session,
            frame_waiter: frame_waiter,
            frame_stream: frame_stream,
            views: views,
            device: device,
            queue: queue,
            view_config: view_config,
            swapchains: Vec::new(),
            swapchain_images: Vec::new(),
            swapchain_textures: Vec::new(),
            swapchain_views: Vec::new(),
            depth_views: Vec::new(),
            reference_space: reference_space,
            event_buffer: openxr::EventDataBuffer::new(),
            session_running: false,
        })
    }

    pub fn get_session_running(&self) -> bool {
        self.session_running
    }

    fn create_xr_views(&mut self) -> Result<(), openxr::sys::Result> {
        for (i, swapchain) in self.swapchains.iter().enumerate() {
            let images = swapchain.enumerate_images()?;

            let mut views = Vec::new();

            let swapchain_width = self.views[i].recommended_image_rect_width;
            let swapchain_height = self.views[i].recommended_image_rect_height;

            for raw_image in images {
                let vk_image = vk::Image::from_raw(raw_image);

                let hal_desc = wgpu_hal::TextureDescriptor {
                    label: Some("OpenXR Swapchain Image"),
                    size: wgpu::Extent3d {
                        width: swapchain_width,
                        height: swapchain_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUses::COLOR_TARGET,
                    view_formats: Vec::new(),
                    memory_flags: wgpu_hal::MemoryFlags::empty(),
                };
                let desc = wgpu::TextureDescriptor {
                    label: Some("OpenXR Swapchain Image"),
                    size: wgpu::Extent3d {
                        width: swapchain_width,
                        height: swapchain_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                };

                let hal_texture = unsafe {
                    let hal_device = self
                        .device
                        .as_hal::<wgpu_hal::api::Vulkan>()
                        .expect("wgpu device is not using Vulkan");

                    hal_device.texture_from_raw(
                        vk_image,
                        &hal_desc,
                        Some(Box::new(|| {})),
                        wgpu_hal::vulkan::TextureMemory::External,
                    )
                };

                let texture = unsafe {
                    self.device.create_texture_from_hal::<
                        wgpu_hal::api::Vulkan
                    >(
                        hal_texture,
                        &desc,
                        wgpu::TextureUses::UNINITIALIZED,
                    )
                };

                let view = texture.create_view(
                    &wgpu::TextureViewDescriptor::default()
                );

                views.push(view);
            }

            self.swapchain_views.push(views);

            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("XR Depth"),
                size: wgpu::Extent3d {
                    width: swapchain_width,
                    height: swapchain_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let depth_view = depth_texture.create_view(
                &wgpu::TextureViewDescriptor::default()
            );

            self.depth_views.push(depth_view);
        }

        Ok(())
    }

    pub fn create_swapchains(&mut self) -> Result<(), openxr::sys::Result> {
        let views = self.instance
            .enumerate_view_configuration_views(
                self.system,
                self.view_config,
            )?;

        let format = vk::Format::B8G8R8A8_SRGB.as_raw();

        for view in &views {
            let swapchain = self.session.create_swapchain(
                &openxr::SwapchainCreateInfo {
                    create_flags: openxr::SwapchainCreateFlags::EMPTY,
                    usage_flags: openxr::SwapchainUsageFlags::COLOR_ATTACHMENT,
                    format: format as u32,
                    sample_count: view.recommended_swapchain_sample_count,
                    width: view.recommended_image_rect_width,
                    height: view.recommended_image_rect_height,
                    face_count: 1,
                    array_size: 1,
                    mip_count: 1,
                },
            )?;

            self.swapchains.push(swapchain);
        }

        Ok(())
    }

    pub fn poll_events(&mut self) -> Result<(), openxr::sys::Result> {
        while let Some(event) = self.instance.poll_event(&mut self.event_buffer)? {
            match event {
                openxr::Event::SessionStateChanged(event) => {
                    match event.state() {
                        openxr::SessionState::READY => {
                            self.session.begin(self.view_config)?;
                            println!("XR session started");
                            self.session_running = true;
                            self.create_swapchains()?;
                            self.create_xr_views()?;
                            println!("Swapchains created");
                        }

                        openxr::SessionState::STOPPING => {
                            self.session.end()?;
                            println!("XR session stopped");
                            self.session_running = false;
                        }

                        state => {
                            println!("XR session state: {state:?}");
                        }
                    }
                }

                _ => {}
            }
        }

        Ok(())
    }
}
