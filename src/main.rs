use std::io;

use actix_web;

#[actix_web::main]
async fn main() -> io::Result<()> {
    match actix_web::HttpServer::new(|| {
        actix_web::App::new().service(actix_web::web::resource("/").to(|| async { "hello world" }))
    })
    .bind(("0.0.0.0", 1234))
    {
        Ok(server) => {
            println!("Server running!");
            server.run().await
        }
        Err(err) => {
            println!("Error: {}", err);
            Err(err)
        }
    }
}
