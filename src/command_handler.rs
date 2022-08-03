
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use serde_derive::Serialize;

use crate::{
    Host,
    host_manager::StateUpdateMessage,
    connection_manager::ConnectorRequest, 
    connection_manager::ResponseHandlerCallback,
    frontend::DisplayOptions,
    module::command::CommandAction,
};

use crate::module::{
    command::Command,
    command::CommandResult,
};

#[derive(Default)]
pub struct CommandHandler {
    // Command id is the second key.
    commands: HashMap<Host, HashMap<String, Command>>,
    // For connector communication.
    request_sender: Option<Sender<ConnectorRequest>>,
    state_update_sender: Option<Sender<StateUpdateMessage>>,
}

impl CommandHandler {
    pub fn new(request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>) -> Self {
        CommandHandler {
            commands: HashMap::new(),
            request_sender: Some(request_sender),
            state_update_sender: Some(state_update_sender),
        }
    }

    pub fn add_command(&mut self, host: &Host, command: Command) {
        if !self.commands.contains_key(host) {
            self.commands.insert(host.clone(), HashMap::new());
        }

        let command_collection = self.commands.get_mut(host).unwrap();
        let module_spec = command.get_module_spec();

        // Only add if missing.
        if !command_collection.contains_key(&module_spec.id) {
            log::debug!("Adding command {}", module_spec.id);
            command_collection.insert(module_spec.id, command);
        }
    }

    pub fn execute(&mut self, host_id: String, command_id: String, target_id: String) -> CommandAction {
        // TODO: better solution for searching?
        let (host, command_collection) = self.commands.iter().filter(|(host, _)| host.name == host_id).next().unwrap();
        let command = command_collection.get(&command_id).unwrap();

        let state_update_sender = self.state_update_sender.as_ref().unwrap().clone();

        self.request_sender.as_ref().unwrap().send(ConnectorRequest {
            connector_id: command.get_connector_spec().unwrap().id,
            source_id: command.get_module_spec().id,
            host: host.clone(),
            message: command.get_connector_request(target_id),
            response_handler: Self::get_response_handler(host.clone(), command.clone_module(), state_update_sender),
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });

        command.get_action()
    }

    // Return value contains host's commands and command parameters as strings.
    pub fn get_host_commands(&self, host_id: String) -> HashMap<String, CommandData> {
        if let Some((_, command_collection)) = self.commands.iter().filter(|(host, _)| host.name == host_id).next() {
            command_collection.iter().map(|(command_id, command)| {
                (command_id.clone(),
                CommandData::new(command_id.clone(), command.get_action(), command.get_display_options()))
            }).collect()
        }
        else {
            HashMap::new()
        }
    }

    fn get_response_handler(host: Host, command: Command, state_update_sender: Sender<StateUpdateMessage>) -> ResponseHandlerCallback {
        Box::new(move |output, _connector_is_connected| {
            let command_result = match command.process_response(&output) {
                Ok(result) => {
                    log::debug!("Command result received: {}", result.message);
                    result
                },
                Err(error) => {
                    log::error!("Error from command: {}", error);
                    CommandResult::empty_and_critical()
                }
            };

            state_update_sender.send(StateUpdateMessage {
                host_name: host.name,
                display_options: command.get_display_options(),
                module_spec: command.get_module_spec(),
                data_point: None,
                command_result: Some(command_result),
            }).unwrap_or_else(|error| {
                log::error!("Couldn't send message to state manager: {}", error);
            });
        })

    }

}


#[derive(Clone, Serialize)]
pub struct CommandData {
    pub command_id: String,
    pub action: CommandAction,
    pub display_options: DisplayOptions,
}

impl CommandData {
    pub fn new(command_id: String, action: CommandAction, display_options: DisplayOptions) -> Self {
        CommandData {
            command_id: command_id,
            action: action,
            display_options: display_options,
        }
    }
}