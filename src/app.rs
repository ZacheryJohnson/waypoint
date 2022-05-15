use std::{collections::HashMap, process::Stdio, io::{BufReader, BufRead}, sync::{Arc, Mutex}, thread};
use uuid::Uuid;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Contains the configuration of service, namely how to run it and with what arguments.
/// Actual processes are spawned from this configuration.
pub struct ServiceConfig {
    pub path: String,
    pub display_name: String,
}

pub type ServiceId = String;

/// An instance of a configured service, either currently or previously running.
/// Logs are captured from the spawned process.
pub struct ServiceInstance {
    process: std::process::Child,
    logs: Arc<Mutex<Vec<String>>>,
    display_name: String,
}

impl ServiceInstance {
    fn new(mut process: std::process::Child, display_name: String) -> ServiceInstance {
        let stdout = process.stdout.take().unwrap();
        let stderr = process.stderr.take().unwrap();

        let logs_mutex = Arc::new(Mutex::from(Vec::new()));
        let stdout_mutex_clone = logs_mutex.clone();

        thread::spawn(move || {
            let mut out = BufReader::new(stdout);

            loop {
                let mut outbuf = String::new();
                match out.read_line(&mut outbuf) {
                    Ok(0) => {}
                    Ok(_) => stdout_mutex_clone.lock().unwrap().push(outbuf),
                    Err(err) => println!("{:?}", err)
                }
            }
        });
        
        let stderr_mutex_clone = logs_mutex.clone();

        thread::spawn(move || {
            let mut out = BufReader::new(stderr);

            loop {
                let mut outbuf = String::new();
                match out.read_line(&mut outbuf) {
                    Ok(0) => {}
                    Ok(_) => stderr_mutex_clone.lock().unwrap().push(outbuf),
                    Err(err) => println!("{:?}", err)
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

/// Instance of the application, containing all necessary state.
pub struct WaypointApp {
    service_config: HashMap<String, ServiceConfig>,
    running_services: HashMap<ServiceId, ServiceInstance>
}

impl WaypointApp {
    /// Creates a new WaypointApp instance.
    /// There should only be one running at any given time.
    pub fn new() -> WaypointApp {
        let mut app = WaypointApp {
            service_config: HashMap::default(),
            running_services: HashMap::default()
        };

        app.read_service_config_from_disk();

        app
    }

    /// Reads previously configured services from disk, if any.
    pub fn read_service_config_from_disk(&mut self) {
        if let Ok(bytes) = std::fs::read("service_cfg.json") {
            let services: Vec<ServiceConfig> = serde_json::de::from_slice(bytes.as_slice()).unwrap_or_default();
            for svc in services {
                self.service_config.insert(svc.display_name.clone(), svc);
            }

            println!("Service config loaded!");
        } else {
            println!("No service config loaded!");
        }
    }

    /// Writes currently configured services to disk, if any.
    pub fn write_service_config_to_disk(&self) {
        let mut configs = vec!();
        for cfg in self.get_service_config().values() {
            configs.push(cfg);
        } 
        let serialized = serde_json::to_string(&configs);
        
        if std::fs::write("service_cfg.json", serialized.unwrap_or(String::from("[]"))).is_err() {
            println!("Failed to write service config to disk");
        } else {
            println!("Successfully wrote service config to disk");
        }
    }

    /// Adds a service to our service configuration.
    pub fn add_service_config(&mut self, exe_path: &String, display_name: &String) {
        self.service_config.insert(display_name.clone(), ServiceConfig {
            path: exe_path.to_owned(),
            display_name: display_name.to_owned()
        });

        // Once we've added the service, update our local config.
        self.write_service_config_to_disk();
    }

    /// Returns the current service configuration.
    pub fn get_service_config(&self) -> &HashMap<String, ServiceConfig> {
        &self.service_config
    }

    /// Returns the current status of a ServiceInstance.
    pub fn get_service_status(&mut self, service_id: &ServiceId) -> String {
        if let Some(svc) = self.running_services.get_mut(service_id) {
            match svc.process.try_wait() {
                Ok(None) => String::from("Running"),
                _ => String::from("Stopped")
            }
        } else {
            String::from("Stopped")
        }        
    }

    /// Attempts to kill a ServiceInstance, returning true if successful and false if not.
    pub fn kill(&mut self, service_id: &ServiceId) -> bool {
        if let Some(svc) = self.running_services.get_mut(service_id) {
            svc.process.kill().is_ok()
        } else {
            false
        }
    }

    /// Returns all ServiceInstances.
    pub fn get_running_services(&self) -> &HashMap<ServiceId, ServiceInstance> {
        &self.running_services
    }

    /// Returns a specific ServiceInstance, given it's ServiceId.
    pub fn get_service_instance(&self, service_id: &ServiceId) -> Option<&ServiceInstance> {
        self.running_services.get(service_id)
    }

    /// Returns the logs collected from the ServiceInstance over it's lifetime.
    pub fn get_service_logs(&mut self, service_id: &ServiceId) -> Option<Arc<Mutex<Vec<String>>>> {
        let process = self.running_services.get(service_id)?;
        Some(process.logs.clone())
    }

    /// Starts a new ServiceInstance given the display name of the desired service.
    /// 
    /// TODO: This should instead take a ServiceConfig, not a string.
    pub fn start_service<S>(&mut self, service_name: S) -> Option<ServiceId> 
    where S: Into<String> + std::fmt::Display {
        let cfg = self.get_service_config().get(&service_name.to_string())?;
        let exe_path = &cfg.path;

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
        Some(id)
    }
}