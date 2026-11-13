pub struct EngineArguments {
    pub no_vr: bool,
    pub server_url: String,
}
impl Default for EngineArguments {
    fn default() -> Self {
        Self {
            no_vr: false,
            server_url: "178.128.158.197:5000".to_string(),
        }
    }
}
impl EngineArguments {
    pub fn from(args: &[String]) -> Self {
        let mut engine_arguments = Self::default();

        for arg in args {
            if arg.starts_with("--no-vr") {
                engine_arguments.set_no_vr(true);
            } else if arg.starts_with("--server=") {
                engine_arguments.set_server_url(arg[9..].to_string());
            }
        }

        engine_arguments
    }

    pub fn set_no_vr(&mut self, no_vr: bool) {
        self.no_vr = no_vr;

        if no_vr {
            println!("Running desktop mode");
        } else {
            println!("Running VR mode");
        }
    }
    pub fn set_server_url(&mut self, server_url: String) {
        self.server_url = server_url;
        println!("Custom server set to: {}", self.server_url);
    }
}
