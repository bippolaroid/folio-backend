use std::io::Result;

use actix_cors::Cors;
use actix_web::{
    web::{self, resource, scope, Json},
    App, HttpResponse, HttpServer,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::{
    auth::check_auth,
    core::data::{load_from_storage, write_local_db, Collection},
    init_local_files,
};

const LOCAL_PROJECTS_PATH: &str = "data";
const PROJECTS_FILE_NAME: &str = "projects";

pub async fn start_server(addr: String) -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                scope("/v1")
                    .service(
                        resource("/projects")
                            .route(web::get().to(get_handler))
                            .route(web::put().to(update_handler))
                            .route(web::post().to(create_handler))
                            .route(web::delete().to(del_handler)),
                    )
                    .service(resource("/folio").route(web::get().to(status_handler))),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
    })
    .bind(&addr)?
    .run()
    .await
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

async fn create_handler(collection: Json<Collection>, remote_key: BearerAuth) -> HttpResponse {
    if let Ok(_) = check_auth(remote_key.token().to_string()) {
        let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
        let collection_title = collection.title.clone();
        match load_from_storage(&local_projects_path) {
            Ok(mut collections) => {
                let collection = collection.into_inner();
                collections.push(collection);
                match write_local_db(&local_projects_path, collections) {
                    Ok(_) => {
                        println!("Added \"{}\"", collection_title);
                        HttpResponse::Ok().body(format!("Added \"{}\"", collection_title))
                    }
                    Err(error) => {
                        eprintln!("Failed to add \"{}\"", collection_title);
                        HttpResponse::from_error(error)
                    }
                }
            }
            Err(error) => {
                eprintln!("Failed to add \"{}\"", collection_title);
                HttpResponse::from_error(error)
            }
        }
    } else {
        HttpResponse::Unauthorized().body("Unauthorized token.")
    }
}

async fn update_handler(collection: Json<Collection>, remote_key: BearerAuth) -> HttpResponse {
    if let Ok(_) = check_auth(remote_key.token().to_string()) {
        let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
        let collection_title = collection.title.clone();
        match load_from_storage(&local_projects_path) {
            Ok(collections) => {
                let collection = collection.into_inner();
                let collections = collections
                    .iter()
                    .map(|item| {
                        if item.id == collection.id {
                            return collection.clone();
                        } else {
                            return item.clone();
                        }
                    })
                    .collect();
                match write_local_db(&local_projects_path, collections) {
                    Ok(_) => {
                        println!("Updated \"{}\"", collection_title);
                        HttpResponse::Ok().body(format!("Updated \"{}\"", collection_title))
                    }
                    Err(error) => HttpResponse::from_error(error),
                }
            }
            Err(error) => HttpResponse::from_error(error),
        }
    } else {
        HttpResponse::Unauthorized().body("Unauthorized token.")
    }
}

async fn del_handler(project: Json<Collection>, remote_key: BearerAuth) -> HttpResponse {
    if let Ok(_) = check_auth(remote_key.token().to_string()) {
        let local_projects_path = format!("{}/{}.json", LOCAL_PROJECTS_PATH, PROJECTS_FILE_NAME);
        let mut projects = load_from_storage(&local_projects_path).unwrap();
        let project_id: usize = project.id.try_into().unwrap();
        projects.remove(project_id);
        let mut i: u32 = 0;
        let projects = projects
            .into_iter()
            .map(|mut item| {
                item.id = i;
                i += 1;
                item
            })
            .collect();
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
    } else {
        HttpResponse::Unauthorized().body("Unauthorized token.")
    }
}

async fn status_handler() -> HttpResponse {
    HttpResponse::Ok().body("folio is running")
}
