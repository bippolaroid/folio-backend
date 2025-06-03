use std::{
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Read, Result},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use awc::Client;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Collection {
    pub id: u32,
    pub client: String,
    pub client_logo: String,
    pub accent_color: String,
    pub title: String,
    pub tags: Vec<String>,
    pub featured: String,
    pub keypoints: Vec<Keypoint>,
    pub summary: String,
    pub text_fields: Vec<TextField>,
    pub last_modified: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Keypoint {
    pub id: u32,
    pub featured: Vec<String>,
    pub title: String,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextField {
    pub id: u32,
    pub name: String,
    pub value: String,
}

impl TextField {
    pub fn new(id: u32, name: String, value: String) -> Self {
        TextField {
            id: id,
            name: name,
            value: value,
        }
    }
}

impl Collection {
    fn new(
        id: u32,
        client: String,
        client_logo: String,
        accent_color: String,
        title: String,
        tags: Vec<String>,
        featured: String,
        keypoints: Vec<Keypoint>,
        summary: String,
        last_modified: String,
    ) -> Self {
        Collection {
            id: id,
            client: client,
            client_logo: client_logo,
            accent_color: accent_color,
            title: title,
            tags: tags,
            featured: featured,
            keypoints: keypoints,
            summary: summary,
            text_fields: Vec::new(),
            last_modified: last_modified,
        }
    }
    pub fn default(projects_data: Vec<Collection>) -> Self {
        let id: u32 = projects_data.len().try_into().unwrap_or(0);
        let keypoint = Keypoint {
            id: 0,
            featured: vec!["n/a".to_string()],
            title: format!("New Keypoint 1 - {}", id),
            summary: format!("New Summary 1 - {}", id),
        };
        Collection {
            id: id,
            client: format!("New Client {}", id),
            client_logo: "n/a".to_string(),
            accent_color: "#cacaca".to_string(),
            title: format!("New Title {}", id),
            tags: vec!["Default".to_string()],
            featured: "n/a".to_string(),
            keypoints: vec![keypoint],
            summary: format!("New Summary {}", id),
            text_fields: Vec::new(),
            last_modified: Local::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}

pub fn load_from_storage(local_projects_path: &str) -> Result<Vec<Collection>> {
    match File::open(local_projects_path) {
        Ok(local_projects_file) => {
            let mut buffer: Vec<u8> = Vec::new();
            let mut reader = BufReader::new(local_projects_file);
            match reader.read_to_end(&mut buffer) {
                Ok(size) => {
                    println!("Local projects data size: {}", size);
                    match serde_json::from_slice::<Vec<Collection>>(&buffer) {
                        Ok(local_projects_data) => {
                            println!("Successfully loaded local projects data.");
                            return Ok(local_projects_data);
                        }
                        Err(error) => {
                            eprintln!("Local projects data structure is incorrect: {}", error);
                            return Err(error.into());
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Local projects data could not be read: {}", error);
                    return Err(error);
                }
            }
        }
        Err(error) => {
            eprintln!("Error opening local projects data file: {}", error);
            Err(error)
        }
    }
}

pub async fn load_from_cdn(remote_projects_path: &str) -> Result<Vec<Collection>> {
    let client = Client::default();
    match client.get(remote_projects_path).send().await {
        Ok(mut response) => match response.body().await {
            Ok(body) => {
                println!("Remote projects data size: {}", body.len());
                match serde_json::from_slice::<Vec<Collection>>(&body) {
                    Ok(projects_data) => {
                        println!("Remote projects data loaded!");
                        Ok(projects_data)
                    }
                    Err(error) => {
                        eprintln!("Remote projects data structure is incorrect: {}", error);
                        Err(error.into())
                    }
                }
            }
            Err(_) => {
                let error = Error::new(
                    ErrorKind::InvalidData,
                    "Failed to read remote projects data from CDN.",
                );
                eprintln!("Remote projects data could not be read: {}", error);
                Err(error)
            }
        },
        Err(_) => {
            let error = Error::new(ErrorKind::NotConnected, "Failed to connect to CDN.");
            eprintln!("Error requesting remote projects data from CDN: {}", error);
            Err(error)
        }
    }
}

pub fn write_local_db(path: &str, projects: Vec<Collection>) -> Result<Vec<Collection>> {
    match File::create(&path) {
        Ok(file) => {
            let writer = BufWriter::new(file);
            let _ = serde_json::to_writer_pretty(writer, &projects);
            Ok(projects)
        }
        Err(error) => {
            eprintln!("Could not create local projects database: {}", error);
            Err(error)
        }
    }
}
