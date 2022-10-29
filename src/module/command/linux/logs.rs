use std::collections::HashMap;
use crate::frontend;
use crate::module::command::CommandAction;
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
pub struct Logs;

impl Module for Logs {
    fn get_metadata() -> Metadata {
        // TODO: define dependnecy to systemd-service command
        Metadata {
            module_spec: ModuleSpecification::new("logs", "0.0.1"),
            description: String::from(""),
            url: String::from(""),
        }
    }

    fn new(_settings: &HashMap<String, String>) -> Self {
        Logs { }
    }

    fn get_module_spec(&self) -> ModuleSpecification {
        Self::get_metadata().module_spec
    }
}

impl CommandModule for Logs {
    fn clone_module(&self) -> Command {
        Box::new(self.clone())
    }

    fn get_connector_spec(&self) -> Option<ModuleSpecification> {
        Some(ModuleSpecification::new("ssh", "0.0.1"))
    }

    fn get_display_options(&self) -> frontend::DisplayOptions {
        frontend::DisplayOptions {
            category: String::from("host"),
            display_style: frontend::DisplayStyle::Icon,
            display_icon: String::from("view-document"),
            display_text: String::from("Show logs"),
            action: CommandAction::LogView,
            ..Default::default()
        }
    }

    fn get_connector_request(&self, target_id: String) -> String {
        // TODO: validate target_id?
        match target_id.as_str() {
            "" => String::from("sudo journalctl -q -n 500"),
            "all" => String::from("sudo journalctl -q -n 500"),
            "dmesg" => String::from("sudo journalctl -q -n 500 --dmesg"),
            _ => format!("sudo journalctl -q -n 500 -u {}", target_id)
        }
    }

    fn process_response(&self, response: &ResponseMessage) -> Result<CommandResult, String> {
        Ok(CommandResult::new(response.message.clone()))
    }
}