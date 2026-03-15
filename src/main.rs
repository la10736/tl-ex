//! A simple Pokedex REST API server.

use actix_web::{get, web, App, Error, HttpResponse, HttpServer, Responder};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use async_trait::async_trait;
use clap::Parser;
use log::debug;
use serde::{Deserialize, Serialize};

/// Pokemon data type
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Pokemon {
    pub name: String,
    pub description: String,
    pub habitat: Option<String>,
    pub is_legendary: bool,
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ServiceError {
    #[error("not found error")]
    NotFound { name: String },
    #[error("unknown service error: {error:?}")]
    Unknown { error: String },
}


/// The trait that abstract the async pokemon info provider.
#[async_trait]
trait PokemonProvider {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError>;
}

pub struct PokemonService {
    provider: Box<dyn PokemonProvider>,
}

impl PokemonService {
    fn new(
        pokemon_provider: impl PokemonProvider + 'static,
    ) -> PokemonService {
        PokemonService {
            provider: Box::new(pokemon_provider),
        }
    }

    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        self.provider.pokemon(name).await
    }
}

/// Empty provider
struct VoidPokemonProvider;

#[async_trait]
impl PokemonProvider for VoidPokemonProvider {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        Err(ServiceError::NotFound { name: name.to_string() })
    }
}

impl Default for PokemonService {
    fn default() -> Self {
        Self::new(
            VoidPokemonProvider,
        )
    }
}

#[get("/pokemon/{name}")]
async fn pokemon(
    core: web::Data<PokemonService>,
    name: web::Path<(String,)>,
) -> impl Responder {
    let name = name.into_inner().0;
    debug!("New pokemon request '{name}'");
    match core.pokemon(&name).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(err) => HttpResponse::from(err).into(),
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
