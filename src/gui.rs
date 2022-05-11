use crate::app::{*};

use eframe::egui;

pub struct WaypointGui {
    app: WaypointApp,
    service_id: ServiceId,
    logs_enabled: bool,
    realtime_logs: bool,
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
            })),
        );
    }
}

impl WaypointGui {
    /// This view allows users to configure services.
    fn draw_service_configuration(&mut self, ui: &mut egui::Ui) {

    }

    /// This view allows users to interact with service runtimes, including starting new services, reading logs, or killing running services.
    fn draw_service_management(&mut self, ui: &mut egui::Ui) {
        let services = self.app.get_running_services();

        let selected_service = self.app.get_service_instance(&self.service_id);
        egui::ComboBox::from_label("Services")
            .selected_text(if selected_service.is_some() { selected_service.unwrap().display_name() } else { String::default() })
            .show_ui(ui, |ui| {
                for serv in services {
                    ui.selectable_value(&mut self.service_id, serv.0.to_string(), serv.1.display_name());
                }
            }
        );

        if ui.button("Start").clicked() {                
            let service_id = self.app.start_service("foobar");
            self.service_id = service_id;
        }
        if self.service_id != ServiceId::default() {
            if ui.button("Kill").clicked() {
                self.app.kill(&self.service_id);
            }
            ui.horizontal(|ui| {
                ui.label(format!("Selected service ID: '{}'", self.service_id));
                
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
            self.draw_service_management(ui);
        });
    }
}