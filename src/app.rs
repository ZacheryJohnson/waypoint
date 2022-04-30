use std::{collections::HashMap, process::Stdio, io::{Read, BufReader, BufRead}, sync::{Arc, Mutex, mpsc::Sender}, thread};
use uuid::Uuid;

pub struct ServiceConfig {
    pub path: String,
    pub instance_count: u32
}

pub type ServiceId = String;

struct ServiceInstance {
    process: std::process::Child,
    logs: Arc<Mutex<Vec<String>>>,
}

impl ServiceInstance {
    fn new(mut process: std::process::Child) -> ServiceInstance {
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
            logs: logs_mutex
        }
    }
}

pub struct WaypointApp {
    services: HashMap<String, ServiceConfig>,
    running_services: HashMap<ServiceId, ServiceInstance>
}

impl WaypointApp {

    pub fn new() -> WaypointApp {
        WaypointApp {
            services: HashMap::default(),
            running_services: HashMap::default()
        }
    }

    pub fn get_service_config(&self) -> &HashMap<String, ServiceConfig> {
        &self.services
    } 

    pub fn set_instance_count<S>(&mut self, service_name: S, count: u32) where S: Into<String>{
        match self.services.get_mut(&service_name.into()) {
            Some(service) => {
                service.instance_count = count;
            },
            None => {}
        }
    }

    pub fn get_service_logs(&mut self, service_id: &ServiceId) -> Option<Arc<Mutex<Vec<String>>>> {
        let process = self.running_services.get(service_id)?;
        Some(process.logs.clone())
    }

    pub fn start_service<S>(&mut self, service_name: S) -> ServiceId where S: Into<String> {
        let exe_path = "C:\\Users\\Zach\\dev\\dummy\\target\\debug\\dummy.exe";

        println!("Spawning process {}", exe_path);
        
        let child_process = std::process::Command::new(exe_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(format!("failed to start process {}", exe_path).as_str());

        let id = Uuid::new_v4().to_string();

        let new_instance = ServiceInstance::new(child_process);

        self.running_services.insert(id.clone(), new_instance);

        println!("Spun up new service {}", id);
        id
    }
}