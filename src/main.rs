mod app;
mod gui;

fn main() {
    let mut gui = gui::WaypointGui::new(app::WaypointApp::new());
} 
