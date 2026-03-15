//! Utilities for integration tests: provides fakes and in-memory sim implementation for the
//! needed dependencies.

use async_trait::async_trait;
use crate::service::*;
use super::*;
#[derive(Default, Clone)]
/// Trivial InMemory implementation
pub struct InMemoryPokemonProvider(pub HashMap<String, Pokemon>);

impl InMemoryPokemonProvider {
    pub fn add(&mut self, p: Pokemon) {
        self.0.insert(p.name.clone(), p);
    }
}

#[async_trait]
impl PokemonProvider for InMemoryPokemonProvider {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound {
                name: name.to_string(),
            })
    }
}

#[derive(Clone)]
pub struct FakePokemonService {
    pub pokemon_provider: InMemoryPokemonProvider,
    pub language: Language,
    pub translate_ok: bool,
    pub provide_pokemon_error: bool,
}

impl Default for FakePokemonService {
    fn default() -> Self {
        FakePokemonService {
            pokemon_provider: Default::default(),
            language: Language::Shakespeare,
            translate_ok: true,
            provide_pokemon_error: false,
        }
    }
}

impl FakePokemonService {
    pub fn description(&self, name: &str) -> Option<String> {
        self.pokemon_provider
            .0
            .get(name)
            .map(|p| &p.description)
            .cloned()
    }
}

#[async_trait]
impl PokemonProvider for FakePokemonService {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        if self.provide_pokemon_error {
            return Err(ServiceError::Unknown {
                error: format!("Pokemon name: {}", name),
            });
        }
        self.pokemon_provider.pokemon(name).await
    }
}

#[async_trait]
impl TranslationProvider for FakePokemonService {
    async fn translate(&self, lang: Language, body: &str) -> Result<String, ServiceError> {
        self.translate_ok
            .then_some(format!("{lang:?},{body}"))
            .ok_or_else(|| ServiceError::Unknown {
                error: body.to_string(),
            })
    }
}
impl SelectLanguagePolicy for FakePokemonService {
    fn select(&self, _pokemon: &Pokemon) -> Language {
        self.language
    }
}

impl From<FakePokemonService> for Option<PokemonService> {
    fn from(fake: FakePokemonService) -> Self {
        Some(PokemonService::new(fake.clone(), fake.clone(), fake))
    }
}
