use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};

use crate::Host;
use crate::configuration::{CacheSettings, Hosts};
use crate::enums::Criticality;
use crate::module::connection::ResponseMessage;
use crate::module::{monitoring::*, ModuleSpecification};
use crate::module::ModuleFactory;
use crate::host_manager::{StateUpdateMessage, HostManager};
use crate::connection_manager::{ ConnectorRequest, ResponseHandlerCallback, RequestType, CachePolicy };
use crate::utils::ErrorMessage;


// Default needs to be implemented because of Qt QObject requirements.
#[derive(Default)]
pub struct MonitorManager {
    // Host name is the first key, monitor id is the second key.
    monitors: HashMap<String, HashMap<String, Monitor>>,
    /// For communication to ConnectionManager.
    request_sender: Option<Sender<ConnectorRequest>>,
    // Channel to send state updates to HostManager.
    state_update_sender: Option<Sender<StateUpdateMessage>>,
    /// Every refresh operation gets an invocation ID. Valid ID numbers begin from 1.
    invocation_id_counter: u64,
    cache_settings: CacheSettings,

    // Shared resources.
    host_manager: Rc<RefCell<HostManager>>,
    module_factory: Arc<ModuleFactory>,
}

impl MonitorManager {
    pub fn new(cache_settings: CacheSettings,
               host_manager: Rc<RefCell<HostManager>>,
               module_factory: Arc<ModuleFactory>) -> Self {

        MonitorManager {
            monitors: HashMap::new(),
            request_sender: None,
            state_update_sender: None,
            invocation_id_counter: 0,
            cache_settings: cache_settings,

            host_manager: host_manager.clone(),
            module_factory: module_factory,
        }
    }

    pub fn configure(&mut self,
                     hosts_config: &Hosts,
                     request_sender: mpsc::Sender<ConnectorRequest>,
                     state_update_sender: Sender<StateUpdateMessage>) {

        self.monitors.clear();
        self.request_sender = Some(request_sender);
        self.state_update_sender = Some(state_update_sender);

        for (host_id, host_config) in hosts_config.hosts.iter() {

            let mut new_monitors = Vec::<Monitor>::new();
            for (monitor_id, monitor_config) in host_config.monitors.iter() {
                let monitor_spec = ModuleSpecification::new(monitor_id.as_str(), monitor_config.version.as_str());
                let monitor = self.module_factory.new_monitor(&monitor_spec, &monitor_config.settings);
                new_monitors.push(monitor);
            }

            let base_modules = new_monitors.iter().filter(|monitor| monitor.get_metadata_self().parent_module.is_some())
                                                  .map(|monitor| monitor.get_metadata_self().parent_module.unwrap())
                                                  .collect::<Vec<_>>();

            for monitor in new_monitors {
                // Base modules won't get the initial NoData data point sent.
                let is_base = base_modules.contains(&monitor.get_module_spec());
                self.add_monitor(host_id.clone(), monitor, !is_base);
            }
        }
    }
        

    // Adds a monitor but only if a monitor with the same ID doesn't exist.
    fn add_monitor(&mut self, host_id: String, monitor: Monitor, send_initial_value: bool) {
        let monitor_collection = self.monitors.entry(host_id.clone()).or_insert(HashMap::new());
        let module_spec = monitor.get_module_spec();

        // Only add if missing.
        if !monitor_collection.contains_key(&module_spec.id) {

            if send_initial_value {
                // Add initial state value indicating no data as been received yet.
                self.state_update_sender.as_ref().unwrap().send(StateUpdateMessage {
                    host_name: host_id,
                    display_options: monitor.get_display_options(),
                    module_spec: monitor.get_module_spec(),
                    data_point: Some(DataPoint::no_data()),
                    command_result: None,
                    errors: Vec::new(),
                    stop: false,
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to state manager: {}", error);
                });
            }

            // Independent monitors are always executed first.
            // They don't depend on platform info or connectors.
            // TODO: needed?
            /*
            if monitor.get_connector_spec().is_none() {
                self.invocation_id_counter += 1;
                let ignore_cache = self.cache_settings.provide_initial_value == false;

                self.request_sender.send(ConnectorRequest {
                    connector_spec: None,
                    source_id: monitor.get_module_spec().id,
                    host: host.clone(),
                    messages: Vec::new(),
                    request_type: RequestType::Command,
                    response_handler: Self::get_response_handler(
                        host.clone(), vec![monitor.box_clone()], self.invocation_id_counter, self.request_sender.clone(),
                        self.state_update_sender.clone(), DataPoint::empty_and_critical(), ignore_cache
                    ),
                    ignore_cache: ignore_cache
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to connector: {}", error);
                });
            } */

            monitor_collection.insert(module_spec.id, monitor);
        }
    }

    /// Intended to be run only once in the beginning when possibly refreshing all host data.
    /// Returns list of host IDs that were refreshed.
    pub fn refresh_platform_info_all(&mut self, cache_policy: Option<CachePolicy>) -> Vec<String> {
        let cache_policy = if let Some(cache_policy) = cache_policy {
            cache_policy
        }
        else if !self.cache_settings.enable_cache {
            CachePolicy::BypassCache
        }
        else if self.cache_settings.provide_initial_value {
            CachePolicy::PreferCache
        }
        else {
            CachePolicy::BypassCache
        };
        let host_ids = self.monitors.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>();
        for host_id in &host_ids {
            self.refresh_platform_info(host_id, Some(cache_policy));
        }

        host_ids
    }

    /// Refreshes platform info and such in preparation for actual monitor refresh.
    pub fn refresh_platform_info(&mut self, host_id: &String, cache_policy: Option<CachePolicy>) {
        let monitors_for_host = self.monitors.iter().filter(|(name, _)| &host_id == name);
        for (host_name, monitor_collection) in monitors_for_host {

            let host = self.host_manager.borrow().get_host(host_name);
            let cache_policy = if let Some(cache_policy) = cache_policy {
                cache_policy
            }
            else if !self.cache_settings.enable_cache {
                CachePolicy::BypassCache
            }
            else if self.cache_settings.provide_initial_value {
                CachePolicy::PreferCache
            }
            else {
                CachePolicy::BypassCache
            };

            // Executed only if required connector is available.
            // TODO: remove hardcoding and execute once per connector type.
            if monitor_collection.iter().any(|(_, monitor)| monitor.get_connector_spec().unwrap_or_default().id == "ssh") {
                // Note that these do not increment the invocation ID counter.
                let info_provider = internal::PlatformInfoSsh::new_monitoring_module(&HashMap::new());

                let messages = match get_monitor_connector_messages(&host, &info_provider, &DataPoint::empty()) {
                    Ok(messages) => messages,
                    Err(error) => {
                        log::error!("Monitor \"{}\" failed: {}", info_provider.get_module_spec().id, error);
                        return;
                    }
                };

                self.request_sender.as_ref().unwrap().send(ConnectorRequest {
                    connector_spec: info_provider.get_connector_spec(),
                    source_id: info_provider.get_module_spec().id,
                    host: host.clone(),
                    messages: messages,
                    request_type: RequestType::Command,
                    response_handler: Self::get_response_handler(
                        host.clone(),
                        vec![info_provider],
                        0,
                        self.request_sender.as_ref().unwrap().clone(),
                        self.state_update_sender.as_ref().unwrap().clone(),
                        DataPoint::empty_and_critical(),
                        cache_policy
                    ),
                    cache_policy: cache_policy,
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to connector: {}", error);
                });
            }
        }
    }

    pub fn get_all_host_categories(&self, host_id: &String) -> Vec<String> {
        let mut categories = self.monitors.get(host_id).unwrap().iter()
                                          .map(|(_, monitor)| monitor.get_display_options().category.clone())
                                          .collect::<Vec<_>>();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_of_category_control(&mut self, host_id: &String, category: &String, cache_policy: CachePolicy) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitors_by_category = self.monitors.get(host_id).unwrap().iter()
                                                .filter(|(_, monitor)| &monitor.get_display_options().category == category)
                                                .collect();

        let invocation_ids = self.refresh_monitors(host, monitors_by_category, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_of_category(&mut self, host_id: &String, category: &String) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitors_by_category = self.monitors.get(host_id).unwrap().iter()
                                                .filter(|(_, monitor)| &monitor.get_display_options().category == category)
                                                .collect();

        let cache_policy = if !self.cache_settings.enable_cache {
            CachePolicy::BypassCache
        }
        else if self.cache_settings.prefer_cache {
            CachePolicy::PreferCache
        }
        else {
            CachePolicy::BypassCache
        };

        let invocation_ids = self.refresh_monitors(host, monitors_by_category, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    /// Refresh by monitor ID.
    /// Returns the invocation IDs of the refresh operations.
    pub fn refresh_monitors_by_id(&mut self, host_id: &String, monitor_id: &String, cache_policy: CachePolicy) -> Vec<u64> {
        let host = self.host_manager.borrow().get_host(host_id);
        let monitor = self.monitors.get(host_id).unwrap().iter()
                                   .filter(|(_, monitor)| &monitor.get_module_spec().id == monitor_id)
                                   .collect();

        let invocation_ids = self.refresh_monitors(host, monitor, cache_policy);
        self.invocation_id_counter += invocation_ids.len() as u64;
        invocation_ids
    }

    fn refresh_monitors(&self, host: Host, monitors: HashMap<&String, &Monitor>, cache_policy: CachePolicy) -> Vec<u64> {
        if !host.platform.is_set() {
            log::warn!("[{}] Refreshing monitors despite missing platform info", host.name);
        }

        let mut current_invocation_id = self.invocation_id_counter;
        let mut invocation_ids = Vec::new();

        // Split into 2: base modules and extension modules.
        let (extensions, bases): (Vec<&Monitor>, Vec<&Monitor>) = 
            monitors.values().partition(|monitor| monitor.get_metadata_self().parent_module.is_some());

        for monitor in bases {
            current_invocation_id += 1;
            invocation_ids.push(current_invocation_id);

            // Request will contain the base monitors and possible extensions modules.
            let mut request_monitors = vec![monitor.box_clone()];

            extensions.iter().filter(|ext| ext.get_metadata_self().parent_module.unwrap() == monitor.get_module_spec())
                             .for_each(|extension| request_monitors.push(extension.box_clone()));

            Self::send_connector_request(
                host.clone(),
                request_monitors,
                current_invocation_id,
                self.request_sender.as_ref().unwrap().clone(),
                self.state_update_sender.as_ref().unwrap().clone(),
                DataPoint::empty_and_critical(), cache_policy.clone()
            );
        }

        invocation_ids
    }

    // TODO: maybe refactor so there's less parameters to pass?
    /// Send a connector request to ConnectionManager.
    fn send_connector_request(host: Host, monitors: Vec<Monitor>, invocation_id: u64,
                              request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>,
                              parent_result: DataPoint, cache_policy: CachePolicy) {

        let monitor = monitors[0].box_clone();

        let messages = match get_monitor_connector_messages(&host, &monitor, &parent_result) {
            Ok(messages) => messages,
            Err(error) => {
                log::error!("Monitor \"{}\" failed: {}", monitor.get_module_spec().id, error);
                return;
            }
        };

        let response_handler = Self::get_response_handler(
            host.clone(), monitors, invocation_id, request_sender.clone(), state_update_sender, parent_result, cache_policy 
        );

        request_sender.send(ConnectorRequest {
            connector_spec: monitor.get_connector_spec(),
            source_id: monitor.get_module_spec().id,
            host: host.clone(),
            messages: messages,
            request_type: RequestType::Command,
            response_handler: response_handler,
            cache_policy: cache_policy,
        }).unwrap_or_else(|error| {
            log::error!("Couldn't send message to connector: {}", error);
        });
    }

    fn get_response_handler(host: Host, mut monitors: Vec<Monitor>, invocation_id: u64,
                            request_sender: Sender<ConnectorRequest>, state_update_sender: Sender<StateUpdateMessage>,
                            parent_datapoint: DataPoint, cache_policy: CachePolicy) -> ResponseHandlerCallback {

        Box::new(move |results| {
            let monitor = monitors.remove(0);
            let monitor_id = monitor.get_module_spec().id;

            let results_len = results.len();
            let (responses, errors): (Vec<_>, Vec<_>) =  results.into_iter().partition(Result::is_ok);
            let responses = responses.into_iter().map(Result::unwrap).collect::<Vec<_>>();
            let mut errors = errors.into_iter().map(|error| ErrorMessage::new(Criticality::Error, error.unwrap_err())).collect::<Vec<_>>();

            // If CachePolicy::OnlyCache is used and an entry is not found, don't continue.
            if responses.iter().any(|response| response.is_not_found()) {
                return;
            }

            let mut datapoint_result;
            if results_len == 0 {
                // Some special modules require no connectors and receive no response messages.
                // TODO: which modules?
                datapoint_result = monitor.process_response(host.clone(), ResponseMessage::empty(), parent_datapoint.clone())
            }
            else if responses.len() > 0 {
                datapoint_result = monitor.process_responses(host.clone(), responses.clone(), parent_datapoint.clone());
                if let Err(error) = datapoint_result {
                    if error.is_empty() {
                        // Was not implemented, so try the other method.
                        let response = responses[0].clone();
                        datapoint_result = monitor.process_response(host.clone(), response.clone(), parent_datapoint.clone())
                                                  .map(|mut data_point| { data_point.is_from_cache = response.is_from_cache; data_point });
                    }
                    else {
                        datapoint_result = Err(error);
                    }
                }
            }
            else {
                log::warn!("No response messages received for monitor {}", monitor_id);
                // This is just ignored below.
                datapoint_result = Err(String::new());
            }

            let new_data_point = match datapoint_result {
                Ok(mut data_point) => {
                    log::debug!("[{}] Data point received for monitor {}: {} {}", host.name, monitor_id, data_point.label, data_point);
                    data_point.invocation_id = invocation_id;
                    data_point
                },
                Err(error) => {
                    if !error.is_empty() {
                        errors.push(ErrorMessage::new(Criticality::Error, error));
                    }
                    // In case this was an extension module, retain the parents data point unmodified.
                    parent_datapoint
                }
            };

            for error in errors.iter() {
                log::error!("[{}] Error from monitor {}: {}", host.name, monitor_id, error.message);
            }

            if !monitors.is_empty() {
                // Process extension modules recursively until the final result is reached.
                Self::send_connector_request(host, monitors, invocation_id, request_sender, state_update_sender, new_data_point, cache_policy);
            }
            else {
                state_update_sender.send(StateUpdateMessage {
                    host_name: host.name.clone(),
                    display_options: monitor.get_display_options(),
                    module_spec: monitor.get_module_spec(),
                    data_point: Some(new_data_point),
                    command_result: None,
                    errors: errors,
                    stop: false,
                }).unwrap_or_else(|error| {
                    log::error!("Couldn't send message to state manager: {}", error);
                });
            }
        })
    }
}

fn get_monitor_connector_messages(host: &Host, monitor: &Monitor, parent_datapoint: &DataPoint) -> Result<Vec<String>, String> {
    let mut all_messages: Vec<String> = Vec::new();

    match monitor.get_connector_messages(host.clone(), parent_datapoint.clone()) {
        Ok(messages) => all_messages.extend(messages),
        Err(error) => {
            if !error.is_empty() {
                return Err(error);
            }
        }
    }

    match monitor.get_connector_message(host.clone(), parent_datapoint.clone()) {
        Ok(message) => all_messages.push(message),
        Err(error) => {
            if !error.is_empty() {
                return Err(error);
            }
        }
    }

    Ok(all_messages)
}
