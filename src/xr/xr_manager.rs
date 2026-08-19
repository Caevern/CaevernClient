pub struct XRManager {
    _instance: openxr::Instance,
    _system: Option<openxr::SystemId>,
    _session: Option<openxr::Session<openxr::Vulkan>>,
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

        let extensions = openxr::ExtensionSet::default();
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

        Ok(Self {
            _instance: instance,
            _system: Some(system),
            _session: None,
            _frame_waiter: None,
            _frame_stream: None,
            _views: Vec::new(),
        })
    }
}
