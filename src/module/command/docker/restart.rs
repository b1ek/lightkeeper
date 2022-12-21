use std::collections::HashMap;
use crate::connection_manager::RequestMessage;
use crate::frontend;
use crate::module::connection::ResponseMessage;
use crate::module::{
    Module,
    command::CommandModule,
    command::Command,
    command::CommandResult,
    Metadata,
    ModuleSpecification,
};


#[derive(Clone)]
pub struct Restart;

impl Module for Restart {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-restart", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Restart { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Restart {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-containers"),
            parent_id: String::from("docker-containers"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("refresh"),
            display_text: String::from("Restart"),
            confirmation_text: String::from("Really restart container?"),
            ..Default::default()
        }
    }

    fn get_connector_message(&self, parameters: Vec<String>) -> RequestMessage {
        // TODO: filter out all but alphanumeric characters
        let target_id = parameters.first().expect("1 parameter is mandatory and should contain a container ID");

        RequestMessage::command(format!("sudo curl --unix-socket /var/run/docker.sock -X POST http://localhost/containers/{}/restart", target_id))
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}