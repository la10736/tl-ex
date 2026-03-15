//! A simple Pokedex REST API server.

use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get, web,
};
use clap::Parser;
use log::debug;
use serde::{Deserialize, Serialize};
use crate::service::{PokemonService, ServiceError};

mod funtranslation_provider;
mod language_policies;
mod rustemon_provider;

#[cfg(test)]
mod tests;
mod service;

/// Pokemon data type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon {
    pub name: String,
    pub description: String,
    pub habitat: Option<String>,
    pub is_legendary: bool,
}


#[get("/pokemon/{name}")]
async fn pokemon(core: web::Data<PokemonService>, name: web::Path<(String,)>) -> impl Responder {
    let name = name.into_inner().0;
    debug!("New pokemon request '{name}'");
    match core.pokemon(&name).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(err) => HttpResponse::from(err),
    }
}
#[get("/pokemon/translated/{name}")]
async fn translated(core: web::Data<PokemonService>, name: web::Path<(String,)>) -> impl Responder {
    let name = name.into_inner().0;
    debug!("New pokemon translated request '{name}'");
    match core.translated(&name).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(err) => HttpResponse::from(err),
    }
}

impl From<ServiceError> for HttpResponse {
    fn from(e: ServiceError) -> Self {
        match e {
            ServiceError::NotFound { .. } => HttpResponse::NotFound().into(),
            ServiceError::Unknown { .. } => HttpResponse::InternalServerError().into(),
        }
    }
}

pub fn app(
    core: Option<PokemonService>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = Error,
        InitError = (),
    >,
> {
    let core = core.unwrap_or_default();
    App::new()
        .app_data(web::Data::new(core))
        .service(pokemon)
        .service(translated)
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
    HttpServer::new(move || app(None))
        .bind(("0.0.0.0", cli.port))?
        .run()
        .await
}
