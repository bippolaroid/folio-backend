use core::settings::Settings;
use std::io::Result;

use crate::{auth::get_local_passkey, core::data::{load_from_cdn, load_from_storage, write_local_db, Collection}};

mod core;
mod server;
mod auth;

//implement periodic core checks and creation if they dont exist (folders, etc)

#[actix_web::main]
async fn main() -> Result<()> {
    let settings = Settings::load().unwrap();
    let _ = init_local_files().await;
    println!("\nStarting administrative server...");
    let server_addr = format!("{}:{}", settings.ipv4_addr.value, settings.port.value);
    let server = server::start_server(server_addr);
    server.await?;
    Ok(())
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
