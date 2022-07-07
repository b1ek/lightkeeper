use std::collections::HashMap;
use crate::frontend;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    Metadata,
    ModuleSpecification,
};

use super::CommandResult;

#[derive(Clone)]
pub struct Docker;

impl Module for Docker {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Docker { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Docker {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_parameters(&self) -> Vec<String> {
        vec![
            String::from(""),
            String::from("ps"),
            String::from("images")
        ]
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            display_name: String::from("test123"),
            display_style: frontend::DisplayStyle::CriticalityLevel,
            category: String::from("docker"),
            use_multivalue: true,
            parent_id: String::from("docker"),
            ..Default::default()
        }
    }

    fn get_connector_request(&self, parameter: Option<String>) -> String {
        let param_string = parameter.unwrap_or_else(|| String::new());
        match param_string.as_str() {
            "ps" => String::from("sudo curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true"),
            "images" => String::from("sudo curl --unix-socket /var/run/docker.sock http://localhost/images/json?all=true"),
            "" => String::from("sudo curl --unix-socket /var/run/docker.sock http://localhost/containers/json?all=true"),
            _ => panic!("Unknown command parameter"),
        }
    }

    fn process_response(&self, response: &String) -> Result<CommandResult, String> {
        log::debug!("TEST");
        Ok(CommandResult::new(String::from("test")))
    }
}