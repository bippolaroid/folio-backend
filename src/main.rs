use actix_cors::Cors;
use actix_web::{
    web::{self, resource, scope, Json},
    App, HttpResponse, HttpServer,
};
use awc::Client;
use core::settings::Settings;
use serde::{Deserialize, Serialize};
use std::io::Result;
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read},
};

mod core;
mod server;

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
        }
    }
    fn default(projects_data: Vec<Collection>) -> Self {
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
        }
    }
}

const LOCAL_PROJECTS_PATH: &str = "data";
const PROJECTS_FILE_NAME: &str = "projects";

//implement periodic core checks and creation if they dont exist (folders, etc)

#[actix_web::main]
async fn main() -> Result<()> {
    let settings = Settings::load().unwrap();
    let _ = init_local_files().await;
    println!("\nStarting administrative server...");
    let server_addr = format!("{}:{}", settings.ipv4_addr.value, settings.port.value);
    let server = HttpServer::new(|| {
        App::new()
            .service(
                scope("/v1").service(
                    resource("/projects")
                        .route(web::get().to(get_handler))
                        .route(web::post().to(post_handler))
                        .route(web::delete().to(del_handler)),
                ),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
    })
    .bind(&server_addr)?;
    println!("Server listening at {}...\n", server_addr);
    server.run().await?;
    Ok(())
}

async fn get_handler() -> HttpResponse {
    let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
    match load_from_storage(&local_projects_path) {
        Ok(projects) => {
            let value = serde_json::to_value(projects).unwrap();
            HttpResponse::Ok().json(value)
        }
        Err(error) => {
            eprintln!("Failed to load projects data: {}", error);
            println!("Re-initializing files...");
            let _ = init_local_files();
            HttpResponse::Ok().json("Failed to load local data. Please refresh.")
        }
    }
}

async fn post_handler(new_project_data: Json<Collection>) -> HttpResponse {
    let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
    let mut current_projects: Vec<Collection>;
    let new_project = Collection::new(
        new_project_data.id,
        new_project_data.client.clone(),
        new_project_data.client_logo.clone(),
        new_project_data.accent_color.clone(),
        new_project_data.title.clone(),
        new_project_data.tags.clone(),
        new_project_data.featured.clone(),
        new_project_data.keypoints.clone(),
        new_project_data.summary.clone(),
    );
    match load_from_storage(&local_projects_path) {
        Ok(projects) => {
            current_projects = projects;
            println!("Projects data loaded.");
            match current_projects
                .iter()
                .find(|project| project.id == new_project.id)
            {
                Some(project) => {
                    let project_index: usize = project.id.try_into().unwrap();
                    current_projects[project_index] = new_project;
                    match write_local_db(&local_projects_path, current_projects) {
                        Ok(current_projects) => {
                            println!(
                                "\nProject id \"{}\" updated to \"{}\"\n",
                                &current_projects[project_index].id,
                                &current_projects[project_index].title
                            );
                            HttpResponse::Ok().body("Project updated successfully!")
                        }
                        Err(error) => {
                            eprintln!("Error writing to local projects data: {}", error);
                            HttpResponse::from_error(error)
                        }
                    }
                }
                None => {
                    println!("\nProject doesn't exist. Creating project...");
                    let project_index: usize = new_project.id.try_into().unwrap();
                    current_projects.push(new_project);
                    match write_local_db(&local_projects_path, current_projects) {
                        Ok(current_projects) => {
                            println!(
                                "\nProject \"{}\" added with id \"{}\"\n",
                                &current_projects[project_index].title,
                                &current_projects[project_index].id
                            );
                            HttpResponse::Ok()
                                .body(format!("Project {} added successfully!", project_index))
                        }
                        Err(error) => {
                            eprintln!("Error writing to local projects data: {}", error);
                            HttpResponse::from_error(error)
                        }
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("Error fetching projects data: {}", error);
            let _ = init_local_files().await;
            HttpResponse::from_error(error)
        }
    }
}

async fn del_handler(project: Json<Collection>) -> HttpResponse {
    let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
    let mut projects = load_from_storage(&local_projects_path).unwrap();
    let project_id: usize = project.id.try_into().unwrap();
    projects.remove(project_id);
    let mut i: u32 = 0;
    let projects = projects.into_iter().map(|mut item| {
        item.id = i;
        i += 1;
        item
    }).collect();
    match write_local_db(&local_projects_path, projects) {
        Ok(_) => {
            println!("{} deleted!", project.title);
            HttpResponse::Ok().body("test")
        }
        Err(error) => {
            eprintln!("Failed to delete project: {}", error);
            HttpResponse::from_error(error)
        }
    }
}

fn load_from_storage(local_projects_path: &str) -> Result<Vec<Collection>> {
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

async fn load_from_cdn(remote_projects_path: &str) -> Result<Vec<Collection>> {
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
                let error = io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Failed to read remote projects data from CDN.",
                );
                eprintln!("Remote projects data could not be read: {}", error);
                Err(error)
            }
        },
        Err(_) => {
            let error = io::Error::new(io::ErrorKind::NotConnected, "Failed to connect to CDN.");
            eprintln!("Error requesting remote projects data from CDN: {}", error);
            Err(error)
        }
    }
}

fn write_local_db(path: &str, projects: Vec<Collection>) -> Result<Vec<Collection>> {
    match File::create(&path) {
        Ok(file) => {
            println!("Creating file...");
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

async fn get_current_projects(
    local_projects_path: &str,
    remote_projects_path: &str,
) -> Result<Vec<Collection>> {
    println!(
        "\nLoading local projects data from \"{}\"...",
        &local_projects_path
    );
    match load_from_storage(local_projects_path) {
        Ok(projects) => Ok(projects),
        Err(_) => {
            eprintln!("Failed to load local projects file.");
            println!(
                "\nLoading remote projects data from \"{}\"...",
                &remote_projects_path
            );
            match load_from_cdn(remote_projects_path).await {
                Ok(projects) => Ok(projects),
                Err(error) => {
                    eprintln!("Could not load remote projects file: {}", error);
                    Err(error)
                }
            }
        }
    }
}

fn init_paths() -> [String; 3] {
    let settings = Settings::load().unwrap();
    return [
        format!(
            "{}/{}.json",
            settings.local_projects_path.value, settings.projects_file_name.value
        ),
        format!(
            "{}/{}.json",
            settings.remote_url.value, settings.projects_file_name.value
        ),
        format!(
            "{}/{}.json",
            settings.local_backup_path.value, settings.projects_file_name.value
        ),
    ];
}

async fn init_local_files() {
    let [local_projects_path, remote_projects_path, local_backup_path] = init_paths();
    let mut current_projects: Vec<Collection> = Vec::new();

    match get_current_projects(&local_projects_path, &remote_projects_path).await {
        Ok(projects) => {
            current_projects = projects;
            // refactor to handle better i.e. if local projects are loaded, don't overwrite.
            println!("\nSyncing local files...");
            match write_local_db(&local_projects_path, current_projects.clone()) {
                Ok(_) => {
                    println!("Local working file created successfully!\n");
                    match write_local_db(&local_backup_path, current_projects) {
                        Ok(_) => {
                            println!("Local backup file created successfully!\n");
                        }
                        Err(error) => {
                            eprintln!("Failed to create backup file: {}", error);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Failed to created working file: {}", error);
                }
            }
        }
        Err(error) => {
            eprintln!("Could not initialize projects data: {}", error);
            println!("Create new local projects file? (Y/N) ");
            println!("Creating new file...");
            let new_collections = vec![Collection::default(current_projects)];
            match write_local_db(&local_projects_path, new_collections) {
                Ok(current_projects) => {
                    println!("Database created successfully!");
                    match write_local_db(&local_backup_path, current_projects) {
                        Ok(_) => {
                            println!("Local backup file created successfully!\n");
                        }
                        Err(error) => {
                            eprintln!("Failed to create backup file: {}", error);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Could not create database: {}", error);
                }
            }
        }
    }
}
