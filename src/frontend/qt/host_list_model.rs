use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;

extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;
use super::monitor_data_model::MonitorDataModel;


#[derive(QObject, Default)]
pub struct HostListModel {
    base: qt_base_class!(trait QAbstractTableModel),
    headers: Vec<QString>,
    hosts: HashMap<String, HostData>,
    hosts_index: HashMap<usize, String>,

    receive_updates: qt_method!(fn(&self)),
    update_receiver: Option<mpsc::Receiver<frontend::HostDisplayData>>,
    update_receiver_thread: Option<thread::JoinHandle<()>>,

    // Couldn't get custom types to work for return types,
    // so for now methods are used to get the monitoring data.
    get_monitor_data: qt_method!(fn(&self, host_id: QString) -> QVariantList),

    // For table row selection.
    selected_row: qt_property!(i32; NOTIFY selected_row_changed),
    selected_row_changed: qt_signal!(),
}

impl HostListModel {
    pub fn new(display_data: &frontend::DisplayData) -> (Self, mpsc::Sender<frontend::HostDisplayData>) {
        let (sender, receiver) = mpsc::channel::<frontend::HostDisplayData>();
        let mut model = HostListModel {
            headers: Vec::new(),
            hosts: HashMap::new(),
            hosts_index: HashMap::new(),
            update_receiver: Some(receiver),
            update_receiver_thread: None,
            selected_row: -1,
            ..Default::default()
        };

        for header in &display_data.table_headers {
            model.headers.push(header.clone().into());
        }

        for (host_id, host_data) in display_data.hosts.iter() {
            model.hosts_index.insert(model.hosts.len(), host_id.clone());
            model.hosts.insert(host_id.clone(), HostData::from(&host_data));
        }

        (model, sender)
    }

    fn receive_updates(&mut self) {
        // Shouldn't be run more than once.
        if self.update_receiver_thread.is_none() {
            let self_ptr = QPointer::from(&*self);
            let set_data = qmetaobject::queued_callback(move |host_display_data: frontend::HostDisplayData| {
                self_ptr.as_pinned().map(|self_pinned| {
                    // HostData cannot be passed between threads so parsing happens in set_data().
                    let host_data = HostData::from(&host_display_data);

                    let _old_value = std::mem::replace(
                        self_pinned.borrow_mut().hosts.get_mut(&host_data.name.to_string()).unwrap(),
                        host_data,
                    );

                    // TODO:
                    // let index = self_pinned.borrow().hosts.data.iter().position(|&item| item.name == host_data.name).unwrap();
                    let top_left = self_pinned.borrow().index(0, 0);
                    let bottom_right = self_pinned.borrow().index(
                        self_pinned.borrow().hosts.len() as i32 - 1,
                        self_pinned.borrow().column_count() as i32 - 1
                    );
                    self_pinned.borrow_mut().data_changed(top_left, bottom_right);
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

    fn get_monitor_data(&self, host_id: QString) -> QVariantList {
        let host = self.hosts.get(&host_id.to_string()).unwrap();
        host.monitor_data.clone().data
    }
}


impl QAbstractTableModel for HostListModel {
    fn row_count(&self) -> i32 {
        self.hosts.len() as i32
    }

    fn column_count(&self) -> i32 {
        6
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        if role != USER_ROLE {
            return QString::from(format!("Unknown role: {}", role)).to_qvariant();
        }

        let host_id = self.hosts_index.get(&(index.row() as usize)).unwrap();
        let row = self.hosts.get(host_id).unwrap();

        match index.column() {
            0 => row.status.to_qvariant(),
            1 => row.name.to_qvariant(),
            2 => row.fqdn.to_qvariant(),
            3 => row.ip_address.to_qvariant(),
            // Return host id to use with get_monitor_data().
            4 => row.name.to_qvariant(),
            5 => row.name.to_qvariant(),
            _ => panic!(),
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        vec![(USER_ROLE, QByteArray::from("value"))].into_iter().collect()
    }
}


#[derive(QGadget, Default, Clone)]
struct HostData {
    status: qt_property!(QString),
    name: qt_property!(QString),
    fqdn: qt_property!(QString),
    ip_address: qt_property!(QString),
    monitor_data: qt_property!(MonitorDataModel),
}

impl HostData {
    pub fn from(host_display_data: &frontend::HostDisplayData) -> Self {

        HostData {
            status: host_display_data.status.clone().to_string().into(),
            name: host_display_data.name.clone().into(),
            fqdn: host_display_data.domain_name.clone().into(),
            ip_address: host_display_data.ip_address.to_string().into(),
            monitor_data: MonitorDataModel::new(&host_display_data),
        }
    }
}