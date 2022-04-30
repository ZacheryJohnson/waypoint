use std::process::{Command, Stdio};

mod app;
mod gui;

fn main() {
    gui::WaypointGui::start().unwrap();
} 
