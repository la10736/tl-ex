//! A simple Pokedex REST API server.

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use log::debug;

#[get("/pokemon/{name}")]
async fn pokemon(
    name: web::Path<(String,)>,
) -> impl Responder {
    let name = name.into_inner().0;
    debug!("New pokemon request '{name}'");
    HttpResponse::NotFound()
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Binding port
    #[arg(long, short, default_value_t = 5000)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    env_logger::init();
    println!("Starting server...");
    HttpServer::new(move || App::new()
        .service(pokemon)
    )
        .bind(("0.0.0.0", cli.port))?
        .run()
        .await
}
