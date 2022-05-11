use std::{collections::HashMap, process::Stdio, io::{BufReader, BufRead}, sync::{Arc, Mutex}, thread};
use uuid::Uuid;

pub struct ServiceConfig {
    pub path: String,
    pub display_name: String,
}

pub type ServiceId = String;

pub struct ServiceInstance {
    process: std::process::Child,
    logs: Arc<Mutex<Vec<String>>>,
    display_name: String,
}

impl ServiceInstance {
    fn new(mut process: std::process::Child, display_name: String) -> ServiceInstance {
        let stdout = process.stdout.take().unwrap();

        let logs_mutex = Arc::new(Mutex::from(Vec::new()));
        let mutex_clone = logs_mutex.clone();

        thread::spawn(move || {
            let mut out = BufReader::new(stdout);

            loop {
                let mut buf = String::new();
                match out.read_line(&mut buf) {
                    Ok(0) => {
                        continue;
                    }
                    Ok(_) => {
                        mutex_clone.lock().unwrap().push(buf);
                        continue;
                    },
                    Err(err) => {
                        println!("{:?}", err);
                        break;
                    }
                }
            }
        });

        ServiceInstance {
            process,
            logs: logs_mutex,
            display_name
        }
    }

    pub fn display_name(&self) -> String {
        self.display_name.clone()
    }
}

pub struct WaypointApp {
    service_config: HashMap<String, ServiceConfig>,
    running_services: HashMap<ServiceId, ServiceInstance>
}

impl WaypointApp {
    pub fn new() -> WaypointApp {
        WaypointApp {
            service_config: HashMap::default(),
            running_services: HashMap::default()
        }
    }

    pub fn add_service_config(&mut self, exe_path: String, display_name: String) {
        self.service_config.insert(display_name.clone(), ServiceConfig {
            path: exe_path,
            display_name
        });
    }

    pub fn get_service_config(&self) -> &HashMap<String, ServiceConfig> {
        &self.service_config
    }

    pub fn kill(&mut self, service_id: &ServiceId) -> bool {
        if let Some(svc) = self.running_services.get_mut(service_id) {
            svc.process.kill().is_ok()
        } else {
            false
        }
    }

    pub fn get_running_services(&self) -> &HashMap<ServiceId, ServiceInstance> {
        &self.running_services
    }

    pub fn get_service_instance(&self, service_id: &ServiceId) -> Option<&ServiceInstance> {
        self.running_services.get(service_id)
    }

    pub fn get_service_logs(&mut self, service_id: &ServiceId) -> Option<Arc<Mutex<Vec<String>>>> {
        let process = self.running_services.get(service_id)?;
        Some(process.logs.clone())
    }

    pub fn start_service<S>(&mut self, service_name: S) -> ServiceId where S: Into<String> + std::fmt::Display {
        let exe_path = "C:\\Users\\Zach\\dev\\dummy\\target\\debug\\dummy.exe";

        println!("Spawning process {}", exe_path);
        
        let child_process = std::process::Command::new(exe_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(format!("failed to start process {}", exe_path).as_str());

        let id = Uuid::new_v4().to_string();

        let new_instance = ServiceInstance::new(child_process, format!("{} ({})", service_name, id));

        self.running_services.insert(id.clone(), new_instance);

        println!("Spun up new service {}", id);
        id
    }
}