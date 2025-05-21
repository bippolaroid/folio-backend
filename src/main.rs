use actix_cors::Cors;
use actix_web::{
    web::{self, resource, scope, Json},
    App, HttpResponse, HttpServer,
};
use std::{
    fs::File,
    io::{BufReader, Write},
    net::{Ipv4Addr, SocketAddr},
};
use std::{io::Result, net::IpAddr};
use types::Project;

mod types;

const HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const PORT: u16 = 1234;
const ADDR: SocketAddr = SocketAddr::new(HOST, PORT);

#[actix_web::main]
async fn main() -> Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .service(
                scope("/api").service(
                    resource("/projects")
                        .route(web::get().to(project_handler))
                        .route(web::post().to(project_handler)),
                ),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method(),
            )
    })
    .bind(ADDR)?;
    println!("Server started at {}", ADDR);
    server.run().await?;
    Ok(())
}

async fn project_handler(project: Json<Project>) -> HttpResponse {
    let project = types::Project {
        id: project.id.clone(),
        client: project.client.clone(),
        client_logo: project.client_logo.clone(),
        accent_color: project.accent_color.clone(),
        title: project.title.clone(),
        tags: project.tags.clone(),
        featured: project.featured.clone(),
        keypoints: project.keypoints.clone(),
        summary: project.summary.clone()
    };

    let filename = "projects.json";
    let file_path = format!("data/{}", filename);

    match File::open(&file_path) {
        Ok(file) => {
            let project_data = handle_project_update(file, project);
            let file = File::create(&file_path).expect("Failed to reopen file for writing");
            write_data(project_data, file).await
        }
        Err(_) => {
            let file = File::create(&file_path).expect("Failed to create file");
            println!("File created.");
            write_data(vec![project], file).await
        }
    }
}

fn handle_project_update(file: File, new_project_data: Project) -> Vec<Project> {
    let reader = BufReader::new(file);
    let mut project_data: Vec<Project> = serde_json::from_reader(reader).unwrap();

    let new_project_id: usize = new_project_data
        .id
        .as_u64()
        .and_then(|unsigned| unsigned.try_into().ok())
        .unwrap();

    if new_project_id < project_data.len() {
        project_data[new_project_id] = new_project_data;
    } else {
        project_data.push(new_project_data);
    }

    project_data
}

async fn write_data(project: Vec<Project>, mut file: File) -> HttpResponse {
    match serde_json::to_string_pretty(&project) {
        Ok(json) => {
            if let Err(err) = file.write_all(json.as_bytes()) {
                eprintln!("Error writing to file {}:", err);
                return HttpResponse::InternalServerError()
                    .body("Failed to write project data to file.");
            }
            let message = format!("Project data written.");
            println!("{}", message);
            println!("Size: {}", json.len());
            HttpResponse::Ok().body(message)
        }
        Err(err) => {
            eprintln!("Error serializing project data: {}", err);
            HttpResponse::InternalServerError().body("Failed to serialize project data.")
        }
    }
}
