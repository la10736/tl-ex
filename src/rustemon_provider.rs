//! This module provide a `PokemonProvider` implementation using `rustemon` crate.
//!
//! It's a base implementation and till doesn't expose any `rustemon` configuration.
//!

use super::{Pokemon, PokemonProvider, ServiceError};
use crate::ServiceError::{NotFound, Unknown};
use async_trait::async_trait;
use log::warn;
use rustemon::model::pokemon::PokemonSpecies;
use rustemon::{client::RustemonClient, error::Error};

#[derive(Default)]
/// Implement `PokemonProvider` via rustemon crate
pub struct Rustemon {
    client: RustemonClient,
}

fn sanitize_description(desc: &str) -> String {
    desc.replace("\r\n", " ")
        .replace(['\n', '\r', '\u{c}'], " ")
        .trim()
        .to_string()
}

#[async_trait]
impl PokemonProvider for Rustemon {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        let data = self.fetch_pokemon_data(name).await?;
        let species = self.fetch_pokemon_species(&data.species.name).await?;
        let description = Self::extract_description(&species);
        let habitat = species.habitat.map(|h| h.name);

        Ok(Pokemon {
            name: data.name,
            description,
            habitat,
            is_legendary: species.is_legendary,
        })
    }
}

impl Rustemon {
    /// Get the first english `flavor_text` and sanitize it
    fn extract_description(species: &PokemonSpecies) -> String {
        species
            .flavor_text_entries
            .iter()
            .filter(|d| d.language.name == "en")
            .map(|t| &t.flavor_text)
            .map(|desc| sanitize_description(&desc))
            .nth(0)
            .unwrap_or_default()
    }

    async fn fetch_pokemon_data(
        &self,
        name: &str,
    ) -> Result<rustemon::model::pokemon::Pokemon, ServiceError> {
        rustemon::pokemon::pokemon::get_by_name(name, &self.client)
            .await
            .map_err(|e| match e {
                Error::Reqwest(_) => NotFound {
                    name: name.to_string(),
                },
                _ => {
                    warn!("Failed to fetch Pokemon {name}: {e:?}");
                    Unknown {
                        error: format!("{e:?}"),
                    }
                }
            })
    }

    async fn fetch_pokemon_species(
        &self,
        species_name: &String,
    ) -> Result<PokemonSpecies, ServiceError> {
        rustemon::pokemon::pokemon_species::get_by_name(&species_name, &self.client)
            .await
            .map_err(|e| match e {
                _ => {
                    warn!("Failed to fetch species {species_name}: {e:?}");
                    Unknown {
                        error: format!("{e:?}"),
                    }
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::base("mewtwo", "mewtwo")]
    #[case::accept_also_upper_case("MEWTWO", "mewtwo")]
    #[case::mixed("ChArizArD", "charizard")]
    #[actix_rt::test]
    async fn should_return_correct_name(#[case] name: &str, #[case] expected: &str) {
        let service = Rustemon::default();

        assert_eq!(expected, service.pokemon(name).await.unwrap().name);
    }

    #[rstest]
    #[case("not_a_pokemon")]
    #[case::empty("")]
    #[case::invalid("a/s/d")]
    #[case::invalid("#$@")]
    #[case::invalid(",")]
    #[actix_rt::test]
    async fn should_return_not_found_error(#[case] name: &str) {
        let service = Rustemon::default();

        assert_eq!(
            NotFound {
                name: name.to_string()
            },
            service.pokemon(name).await.unwrap_err()
        );
    }

    #[rstest]
    #[case::base("mewtwo", Some("rare"))]
    #[case::base("charizard", Some("mountain"))]
    #[case::base("pikachu", Some("forest"))]
    #[case::no_habitat("quaxly", None)]
    #[actix_rt::test]
    async fn should_return_correct_habitat(#[case] name: &str, #[case] expected: Option<&str>) {
        let expected = expected.map(str::to_string);

        let service = Rustemon::default();

        assert_eq!(expected, service.pokemon(name).await.unwrap().habitat);
    }

    #[rstest]
    #[case::base("mewtwo", true)]
    #[case::base("pikachu", false)]
    #[case::no_habitat("quaxly", false)]
    #[actix_rt::test]
    async fn should_return_correct_legendary_state(#[case] name: &str, #[case] expected: bool) {
        let service = Rustemon::default();

        assert_eq!(expected, service.pokemon(name).await.unwrap().is_legendary);
    }

    #[rstest]
    #[case::base(
        "mewtwo",
        "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments."
    )]
    #[case::base(
        "pikachu",
        "When several of these POKéMON gather, their electricity could build and cause lightning storms."
    )]
    #[actix_rt::test]
    async fn should_return_correct_description(#[case] name: &str, #[case] expected: &str) {
        let service = Rustemon::default();

        assert_eq!(expected, service.pokemon(name).await.unwrap().description);
    }

    #[rstest]
    #[case::general(
        "hello\r\nworld\nciao\u{c}other\rmonster",
        "hello world ciao other monster"
    )]
    #[case::dont_change("hello world ciao other monster", "hello world ciao other monster")]
    #[case::boundary("\nhello world ciao other monster\r", "hello world ciao other monster")]
    #[case::stripping(
        "     hello world ciao other monster   ",
        "hello world ciao other monster"
    )]
    fn expected_sanitize_description(#[case] desc: &str, #[case] expected: &str) {
        assert_eq!(expected, &sanitize_description(desc));
    }
}
