use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Error, Read, Write},
    net::{Ipv4Addr, SocketAddr},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub ipv4_addr: Ipv4Setting,
    pub port: U16Setting,
    pub remote_url: StrSetting,
    pub local_projects_path: StrSetting,
    pub local_backup_path: StrSetting,
    pub projects_file_name: StrSetting,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrSetting {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct U16Setting {
    pub name: String,
    pub value: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ipv4Setting {
    pub name: String,
    pub value: Ipv4Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocketAddrSetting {
    pub name: String,
    pub value: SocketAddr,
}

impl Settings {
    // TODO: Settings import/export
    pub fn load() -> Result<Self, Error> {
        match File::open("core/settings.json") {
            Ok(file) => {
                let mut buffer = Vec::new();
                let mut reader = BufReader::new(file);
                match reader.read_to_end(&mut buffer) {
                    Ok(_) => match serde_json::from_slice::<Settings>(&buffer) {
                        Ok(settings) => Ok(settings),
                        Err(error) => {
                            let error: Error = error.try_into().unwrap();
                            eprintln!("Failed to parse settings file: {}", error);
                            if let Err(error) = Settings::new() {
                                fatal_load_error(&error);
                            }
                            Ok(Settings::load().unwrap())
                        }
                    },
                    Err(error) => {
                        eprintln!("Failed to read settings file: {}", error);
                        if let Err(error) = Settings::new() {
                            fatal_load_error(&error);
                        }
                        Ok(Settings::load().unwrap())
                    }
                }
            }
            Err(_) => {
                if let Err(error) = Settings::new() {
                    fatal_load_error(&error);
                }
                Ok(Settings::load().unwrap())
            }
        }
    }

    pub fn new() -> Result<(), Error> {
        println!("Creating settings file...");
        match File::create("core/settings.json") {
            Ok(mut file) => {
                let settings = Settings::new_list();
                let settings_string = serde_json::to_string(&settings).unwrap();
                match file.write_all(settings_string.as_bytes()) {
                    Ok(_) => {
                        println!("Settings file successfully created!");
                        Ok(())
                    }
                    Err(error) => {
                        eprintln!("Failed to write to settings file: {}", error);
                        Err(error)
                    }
                }
            }
            Err(error) => {
                eprintln!("Failed to create settings file: {}", error);
                Err(error)
            }
        }
    }

    pub fn new_list() -> Self {
        Settings {
            ipv4_addr: Ipv4Setting {
                name: "Ipv4 Address".to_string(),
                value: Ipv4Addr::from([127, 0, 0, 1]),
            },
            port: U16Setting {
                name: "Port".to_string(),
                value: 1234,
            },
            remote_url: StrSetting {
                name: "Remote URL".to_string(),
                value: "http://cdn.mikeangelo.art".to_string(),
            },
            local_projects_path: StrSetting {
                name: "Local Projects Path".to_string(),
                value: "data".to_string(),
            },
            local_backup_path: StrSetting {
                name: "Local Backup Path".to_string(),
                value: "backup".to_string(),
            },
            projects_file_name: StrSetting {
                name: "Projects File Name".to_string(),
                value: "projects".to_string(),
            },
        }
    }
}

fn fatal_load_error(error: &Error) {
    eprintln!("Settings load error: {}", error);
    std::process::exit(1);
}
