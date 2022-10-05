use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::HostDataModel;


// TODO: use camelcase with qml models?
#[derive(QObject, Default)]
pub struct HostDataManagerModel {
    base: qt_base_class!(trait QObject),
    hosts: HashMap<String, HostDataModel>,

    receive_updates: qt_method!(fn(&self)),
    update_received: qt_signal!(host_id: QString),

    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    // NOTE: Couldn't get custom types to work for return types,
    // so for now methods are used to get the data in JSON and parsed in QML side.
    get_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_monitor_data_map: qt_method!(fn(&self, host_id: QString) -> QVariantMap),
    get_command_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),
    get_host_data: qt_method!(fn(&self, host_id: QString) -> QVariantMap),
}

impl HostDataManagerModel {
    pub fn new(display_data: &frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let mut model = HostDataManagerModel {
            hosts: HashMap::new(),
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            ..Default::default()
        };

        for (host_id, host_data) in display_data.hosts.iter() {
            model.hosts.insert(host_id.clone(), HostDataModel::from(&host_data));
        }

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't (and can't) be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);

            let set_data = qmetaobject::queued_callback(move |host_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostDataModel cannot be passed between threads so parsing happens here.
                    let host_data = HostDataModel::from(&host_display_data);

                    let old_data = std::mem::replace(
                        self_pinned.borrow_mut().hosts.get_mut(&host_data.name.to_string()).unwrap(),
                        host_data,
                    );

                    self_pinned.borrow().update_received(old_data.name);
                });
            });

            let receiver = self.update_receiver.take().unwrap();
            let thread = std::thread::spawn(move || {
                loop {
                    let received_data = receiver.recv().unwrap();
                    set_data(received_data);
                }
            });

            self.update_receiver_thread = Some(thread);
        }
    }

    // Returns list of MonitorData structs in JSON. Empty if host doesn't exist.
    fn get_monitor_data(&self, host_id: QString) -> QVariantList {
        match self.hosts.get(&host_id.to_string()) {
            Some(host) => host.monitor_data.data.into_iter().map(|(_, data)| data).collect(),
            None => QVariantList::default(),
        }
    }

    // Returns map of MonitorData structs in JSON with monitor id as key. Empty if host doesn't exist.
    fn get_monitor_data_map(&self, host_id: QString) -> QVariantMap {
        match self.hosts.get(&host_id.to_string()) {
            Some(host) => host.monitor_data.clone().data,
            None => QVariantMap::default(),
        }
    }

    // Returns CommandResults from executed commands in JSON. Empty if host doesn't exist.
    // TODO: create a command invocation id to get specific results?
    fn get_command_data(&self, host_id: QString) -> QVariantList {
        match self.hosts.get(&host_id.to_string()) {
            Some(host) => host.command_data.clone().data,
            None => QVariantList::default()
        }
    }

    fn get_host_data(&self, host_id: QString) -> QVariantMap {
        let mut result = QVariantMap::default();
    
        if let Some(host_data) = self.hosts.get(&host_id.to_string()) {
            // TODO: use table headers from display data in init.
            result.insert(QString::from("status"), host_data.status.to_qvariant());
            result.insert(QString::from("name"), host_data.name.to_qvariant());
            result.insert(QString::from("fqdn"), host_data.fqdn.to_qvariant());
            result.insert(QString::from("ip"), host_data.ip_address.to_qvariant());
        }
    
        result
    }
}