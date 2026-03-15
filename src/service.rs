//! Service: Traits and implementation.

use crate::{Pokemon, funtranslation_provider, language_policies, rustemon_provider};
// Even we can use _native_ async trait we prefer to use this macro because the native one are
// not object safe (AKA dynamic compatible)
use async_trait::async_trait;
use serde::Deserialize;

/// The trait that abstract the async pokemon info provider.
#[async_trait]
pub trait PokemonProvider {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Yoda,
    Shakespeare,
}

/// The trait that models the translation language selection.
pub trait SelectLanguagePolicy {
    fn select(&self, pokemon: &Pokemon) -> Language;
}

/// The trait that models the async translations service.
#[async_trait]
pub trait TranslationProvider {
    async fn translate(&self, lang: Language, body: &str) -> Result<String, ServiceError>;
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ServiceError {
    #[error("not found error")]
    NotFound { name: String },
    #[error("unknown service error: {error:?}")]
    Unknown { error: String },
}

/// The sevice implementation: use the provided implementation for fetch pokemon data, select
/// language and translate description.
pub struct PokemonService {
    provider: Box<dyn PokemonProvider>,
    language_policy: Box<dyn SelectLanguagePolicy>,
    translator: Box<dyn TranslationProvider>,
}

impl PokemonService {
    pub fn new(
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

    pub async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        self.provider.pokemon(name).await
    }

    pub async fn translated(&self, name: &str) -> Result<Pokemon, ServiceError> {
        let mut p = self.pokemon(name).await?;
        let language = self.language_policy.select(&p);
        p.description = self
            .translator
            .translate(language, &p.description)
            .await
            .unwrap_or(p.description);
        Ok(p)
    }
}

impl Default for PokemonService {
    fn default() -> Self {
        Self::new(
            rustemon_provider::Rustemon::default(),
            language_policies::CaveAndLegendarySpeakAsYoda,
            funtranslation_provider::FunTranslator::default(),
        )
    }
}
