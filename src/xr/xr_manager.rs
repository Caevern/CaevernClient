use ash::vk::{self, Handle};

pub struct XRManager {
    _instance: openxr::Instance,
    _system: Option<openxr::SystemId>,
    _session: Option<openxr::Session<openxr::Vulkan>>,
    frame_waiter: openxr::FrameWaiter,
    frame_stream: openxr::FrameStream<openxr::Vulkan>,
    _frame_waiter: Option<openxr::FrameWaiter>,
    _frame_stream: Option<openxr::FrameStream<openxr::Vulkan>>,
    _views: Vec<openxr::View>,
}
impl XRManager {
    pub fn new() -> Result<Self, openxr::sys::Result> {
        let entry;
        unsafe {
            if let Ok(temp_entry) = openxr::Entry::load() {
                entry = temp_entry;
            } else {
                return Err(openxr::sys::Result::ERROR_SESSION_NOT_RUNNING);
            }
        }

        let mut extensions = openxr::ExtensionSet::default();
        extensions.khr_vulkan_enable2 = true;
        #[cfg(target_os = "android")]
        {
            enabled_extensions.khr_android_create_instance = true;
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
            .api_version(vk::make_api_version(0, 1, 1, 0));

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

        let vk_physical_device = unsafe {
            vk::PhysicalDevice::from_raw(
                instance
                    .vulkan_graphics_device(system, vk_instance.handle().as_raw() as _)
                    .unwrap() as _,
            )
        };

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
            .queue_create_infos(std::slice::from_ref(&queue_create_info));

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

        let requirements = instance.graphics_requirements::<openxr::Vulkan>(system)?;

        println!(
            "OpenXR Vulkan: {} -> {}",
            requirements.min_api_version_supported, requirements.max_api_version_supported,
        );

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

        Ok(Self {
            _instance: instance,
            _system: Some(system),
            _session: Some(session),
            frame_waiter,
            frame_stream,
            _frame_waiter: None,
            _frame_stream: None,
            _views: Vec::new(),
        })
    }

    pub fn run_frame_loop(&mut self) -> Result<(), openxr::sys::Result> {
        loop {
            /*let frame_state = self.frame_waiter.wait()?;

            self.frame_stream.begin()?;

            if frame_state.should_render {
                // locate views
                // acquire swapchain images
                // render left eye
                // render right eye
                // release images
            }

            self.frame_stream.end(
                frame_state.predicted_display_time,
                openxr::EnvironmentBlendMode::OPAQUE,
                &layers,
            )?;*/
        }
    }
}
