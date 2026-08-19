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
        let entry = unsafe { openxr::Entry::load().expect("openxr not found") };

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

        Ok(Self {
            _instance: instance,
            _system: None,
            _session: None,
            _frame_waiter: None,
            _frame_stream: None,
            _views: Vec::new(),
        })
    }
}
