mod app;
mod gui;

fn main() {
    gui::WaypointGui::new(app::WaypointApp::new());
} 
