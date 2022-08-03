use std::collections::HashMap;
use crate::frontend;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::CommandResult,
    Metadata,
    ModuleSpecification,
};


#[derive(Clone)]
pub struct Shell;

impl Module for Shell {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-shell", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Shell { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Shell {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker"),
            parent_id: String::from("docker"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("terminal"),
            display_priority: 3,
            ..Default::default()
        }
    }

    fn get_connector_request(&self, target_id: String) -> String {
        if target_id.is_empty() {
            panic!("target_id is mandatory and should contain a container ID");
        }

        String::from("TODO")
    }

    fn process_response(&self, response: &String) -> Result<CommandResult, String> {
        log::debug!("Got response: {}", response);
        Ok(CommandResult::new(String::from(response)))
    }
}