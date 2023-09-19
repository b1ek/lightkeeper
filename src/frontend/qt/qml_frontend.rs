use std::sync::mpsc;
use std::env;
extern crate qmetaobject;
use qmetaobject::*;
use super::{
    resources,
    models::*,
};
use crate::{
    frontend,
    command_handler::CommandHandler,
    monitor_manager::MonitorManager,
    configuration,
    module::Metadata,
    ExitReason,
};


pub struct QmlFrontend {
    theme: Option<ThemeModel>,
    update_sender_prototype: mpsc::Sender<frontend::HostDisplayData>,
    host_data_manager: Option<HostDataManagerModel>,
    command_handler: Option<CommandHandlerModel>,
    config_manager: Option<ConfigManagerModel>,
}

impl QmlFrontend {
    pub fn new(display_data: frontend::DisplayData,
               config_dir: String,
               main_config: configuration::Configuration,
               hosts_config: configuration::Hosts,
               group_config: configuration::Groups,
               module_metadatas: Vec<Metadata>
            ) -> QmlFrontend {

        qmetaobject::log::init_qt_to_rust();
        resources::init_resources();

        let style = main_config.display_options.as_ref().unwrap().qtquick_style.as_str();
        if !style.is_empty() {
            if std::env::var("QT_QUICK_CONTROLS_STYLE").is_err() && std::env::var("QT_STYLE_OVERRIDE").is_err() {
                std::env::set_var("QT_STYLE_OVERRIDE", style);
            }
        }

        let theme_model = ThemeModel::new(main_config.display_options.clone().unwrap());
        let (host_data_manager, update_sender) = HostDataManagerModel::new(display_data, main_config.clone());
        let config_manager = ConfigManagerModel::new(config_dir, main_config, hosts_config, group_config, module_metadatas);

        QmlFrontend {
            theme: Some(theme_model),
            update_sender_prototype: update_sender,
            host_data_manager: Some(host_data_manager),
            command_handler: None,
            config_manager: Some(config_manager),
        }
    }

    pub fn setup_command_handler(&mut self, command_handler: CommandHandler, monitor_manager: MonitorManager, config: configuration::Configuration) {
        self.command_handler = Some(CommandHandlerModel::new(command_handler, monitor_manager, config));
    }

    pub fn start(&mut self) -> ExitReason {
        let sandboxed = env::var("FLATPAK_ID").is_ok();
        let main_qml_path = match sandboxed {
            // Inside flatpak.
            true => "/app/qml/main.qml",
            // If running from the source directory, use the QML file from there.
            false => "src/frontend/qt/qml/main.qml",
        };

        qml_register_type::<PropertyTableModel>(cstr::cstr!("PropertyTableModel"), 1, 0, cstr::cstr!("PropertyTableModel"));
        qml_register_type::<HostTableModel>(cstr::cstr!("HostTableModel"), 1, 0, cstr::cstr!("HostTableModel"));

        let qt_data_theme = QObjectBox::new(self.theme.take().unwrap());
        let qt_data_host_data_manager = QObjectBox::new(self.host_data_manager.take().unwrap());
        let qt_data_command_handler = QObjectBox::new(self.command_handler.take().unwrap());
        let qt_data_desktop_portal = QObjectBox::new(DesktopPortalModel::new());
        let qt_data_config_manager = QObjectBox::new(self.config_manager.take().unwrap());
        let sandboxed_updated = qt_data_config_manager.pinned().borrow_mut().setSandboxed(sandboxed);

        if sandboxed_updated {
            // Currently needs a restart so configuration is updated everywhere. Should happen only on first start.
            return ExitReason::Restart;
        }
        else {
            let mut engine = QmlEngine::new();
            engine.set_object_property(QString::from("Theme"), qt_data_theme.pinned());
            engine.set_object_property(QString::from("HostDataManager"), qt_data_host_data_manager.pinned());
            engine.set_object_property(QString::from("CommandHandler"), qt_data_command_handler.pinned());
            engine.set_object_property(QString::from("ConfigManager"), qt_data_config_manager.pinned());
            engine.set_object_property(QString::from("DesktopPortal"), qt_data_desktop_portal.pinned());
            engine.load_file(QString::from(main_qml_path));
            engine.exec();
        }

        if qt_data_config_manager.pinned().borrow().restart_required {
            ExitReason::Restart
        }
        else {
            ExitReason::Quit
        }
    }

    pub fn new_update_sender(&self) -> mpsc::Sender<frontend::HostDisplayData> {
        self.update_sender_prototype.clone()
    }
}