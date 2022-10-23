use std::collections::HashMap;
use serde_derive::Deserialize;
use serde_json;
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
pub struct Prune;

impl Module for Prune {
    fn get_metadata() -> Metadata {
        Metadata {
            module_spec: ModuleSpecification::new("docker-image-prune", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Prune { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Prune {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("docker-images"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("clear"),
            display_text: String::from("Prune"),
            confirmation_text: String::from("Really prune all unused images?"),
            ..Default::default()
        }
    }

    fn get_connector_request(&self, _target_id: String) -> String {
        String::from("sudo curl --unix-socket /var/run/docker.sock -X POST http://localhost/images/prune")
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        let result: PruneResult = serde_json::from_str(response.message.as_str()).map_err(|e| e.to_string())?;
        Ok(CommandResult::new_info(format!("Total reclaimed space: {} B", result.space_reclaimed)))
    }
}


#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PruneResult {
    // images_deleted: Option<Vec<String>>,
    space_reclaimed: i64,
}
