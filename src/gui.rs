use crate::app::{*};

use eframe::egui;

pub struct WaypointGui {
    app: WaypointApp,
    service_id: ServiceId,
    logs_enabled: bool,
    realtime_logs: bool,
    config_service_name: String,
    config_service_path: String,
    selected_config: Option<String>,
}

impl WaypointGui {
    pub fn new(app: WaypointApp) -> WaypointGui {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Waypoint",
            options,
            Box::new(|_cc| Box::new(WaypointGui {
                app,
                service_id: ServiceId::default(),
                logs_enabled: true,
                realtime_logs: true,
                config_service_name: Default::default(),
                config_service_path: String::from("C:\\Users\\Zach\\dev\\dummy\\target\\debug\\dummy.exe"),
                selected_config: Default::default(),
            })),
        );
    }
}

impl WaypointGui {
    /// This view allows users to configure services.
    fn draw_service_configuration(&mut self, ui: &mut egui::Ui) {
        ui.label("Service Configuration");

        ui.horizontal(|ui| {
            ui.label("Service Name:");
            ui.text_edit_singleline(&mut self.config_service_name);
        });

        ui.horizontal(|ui| {
            ui.label("Service Path:");
            ui.text_edit_singleline(&mut self.config_service_path);
        });

        if ui.button("Add").clicked() {
            self.app.add_service_config(&self.config_service_path, &self.config_service_name);
        }
    }

    /// This view allows users to interact with service runtimes, including starting new services, reading logs, or killing running services.
    fn draw_service_management(&mut self, ui: &mut egui::Ui) {
        ui.label("Service Management");

        
        let service_configs = self.app.get_service_config();
        egui::ComboBox::from_label("Configured Services")
            .selected_text(if self.selected_config.is_some() { self.selected_config.to_owned().unwrap() } else { String::default() })
            .show_ui(ui, |ui| {
                for cfg in service_configs {
                    ui.selectable_value(&mut self.selected_config, Some(cfg.0.to_string()), cfg.0.to_string());
                }
            }
        );

        if self.selected_config.is_some() && ui.button("Start").clicked() {                
            let service_id = self.app.start_service(self.selected_config.to_owned().unwrap());
            self.service_id = service_id.unwrap_or_default();
        }

        let services = self.app.get_running_services();

        let selected_service = self.app.get_service_instance(&self.service_id);
        egui::ComboBox::from_label("Running Services")
            .selected_text(if selected_service.is_some() { selected_service.unwrap().display_name() } else { String::default() })
            .show_ui(ui, |ui| {
                for serv in services {
                    ui.selectable_value(&mut self.service_id, serv.0.to_string(), serv.1.display_name());
                }
            }
        );

        if self.service_id != ServiceId::default() {
            if ui.button("Kill").clicked() {
                self.app.kill(&self.service_id);
            }
            
            ui.label(format!("Status: {}", self.app.get_service_status(&self.service_id)));
            ui.label(format!("Selected service ID: '{}'", self.service_id));

            ui.horizontal(|ui| {
                
                ui.horizontal(|ui| {
                    if ui.button("Toggle Logs").clicked() {
                        self.logs_enabled = !self.logs_enabled;
                    }
    
                    ui.checkbox(&mut self.realtime_logs, "Realtime Logs");
                });
                if self.logs_enabled {
                    let mut logstr = String::default();
                    if let Some(logs) = self.app.get_service_logs(&self.service_id) {
                        let logs = &*logs.lock().unwrap();
                        for logline in logs {
                            logstr = logstr + logline;
                        }
                    } else {
                        println!("Failed to get logs...");
                    }

                    let logbox = egui::TextEdit::multiline(&mut logstr).interactive(false).desired_rows(20).desired_width(800.0);

                    ui.add(logbox);
                }
            });
        }
    }
}

impl eframe::App for WaypointGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> () {
        egui::CentralPanel::default().show(ctx, |ui| {    
            if self.realtime_logs {
                ctx.request_repaint();
            }

            self.draw_service_configuration(ui);
            ui.separator();
            self.draw_service_management(ui);
        });
    }
}