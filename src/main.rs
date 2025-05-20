use actix_cors::Cors;
use actix_web::{
    web::{self, resource, scope},
    App, HttpResponse, HttpServer,
};
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    net::{Ipv4Addr, SocketAddr},
};
use std::{io::Result, net::IpAddr};

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

async fn project_handler(project: web::Json<types::ProjectData>) -> HttpResponse {
    let project = types::Project {
        id: project.id.clone(),
        client: project.client.clone(),
        client_logo: project.client_logo.clone(),
        accent_color: project.accent_color.clone(),
        title: project.title.clone(),
        tags: project.tags.clone(),
        featured: project.featured.clone(),
        keypoints: project.keypoints.clone(),
    };

    let filename = "projects.json";
    let file_path = format!("data/{}", filename);

    match File::open(&file_path) {
        Ok(file) => {
            let mut buffer: Vec<u8> = vec![];
            let mut reader = BufReader::new(file);
            let _ = reader.read_to_end(&mut buffer);
            println!("Current: {}", buffer.len());
            send_data(&project, &file_path).await
        }
        Err(_) => {
            File::create(&file_path).expect("Failed to create file");
            send_data(&project, &file_path).await
        }
    }
}

async fn send_data(project: &types::Project, file: &String) -> HttpResponse {
    match serde_json::to_string_pretty(&project) {
        Ok(json) => {
            if let Err(err) = fs::write(file, &json) {
                eprintln!("Error writing to file {}: {}", file, err);
                return HttpResponse::InternalServerError()
                    .body("Failed to write project data to file.");
            }
            let message = format!("Project data written to {}", project.id);
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
