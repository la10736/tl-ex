//! A simple Pokedex REST API server.

use actix_web::{
    App, Error, HttpResponse, HttpServer, Responder,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    get, web,
};
use async_trait::async_trait;
use clap::Parser;
use log::debug;
use serde::{Deserialize, Serialize};

mod rustemon_provider;

#[cfg(test)]
mod tests;

/// Pokemon data type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Language {
    Yoda,
    Shakespeare,
}

/// The trait that models the translation language selection.
trait SelectLanguagePolicy {
    fn select(&self, pokemon: &Pokemon) -> Language;
}

struct FixedLanguageSelector(Language);

#[async_trait]
impl SelectLanguagePolicy for FixedLanguageSelector {
    fn select(&self, _pokemon: &Pokemon) -> Language {
        self.0
    }
}

/// The trait that models the async translations service.
#[async_trait]
trait TranslationProvider {
    async fn translate(&self, lang: Language, body: &str) -> Result<String, ServiceError>;
}

struct FakeTranslator;

#[async_trait]
impl TranslationProvider for FakeTranslator {
    async fn translate(&self, lang: Language, body: &str) -> Result<String, ServiceError> {
        Ok(format!("{lang:?}--{body}"))
    }
}


pub struct PokemonService {
    provider: Box<dyn PokemonProvider>,
    language_policy: Box<dyn SelectLanguagePolicy>,
    translator: Box<dyn TranslationProvider>,
}

impl PokemonService {
    fn new(
        pokemon_provider: impl PokemonProvider + 'static,
        language_policy: impl SelectLanguagePolicy + 'static,
        translator: impl TranslationProvider + 'static,
    ) -> PokemonService {
        PokemonService {
            provider: Box::new(pokemon_provider),
            language_policy: Box::new(language_policy),
            translator: Box::new(translator),
        }
    }

    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        self.provider.pokemon(name).await
    }

    async fn translated(&self, name: &str) -> Result<Pokemon, ServiceError> {
        // Placeholder: don't translate anything, just return the pokemon like it is
        self.pokemon(name).await
    }
}

impl Default for PokemonService {
    fn default() -> Self {
        Self::new(rustemon_provider::Rustemon::default(), FixedLanguageSelector(Language::Shakespeare), FakeTranslator)
    }
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
