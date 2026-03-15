//! This module provide `SelectLanguagePolicy` implementations.
//!
//! Now just `CaveAndLegendarySpeakAsYoda` is available that select `Yoda` language for
//! every legendary pokemon or for the ones that come from "cave". Otherwise return `Shakespeare`
//!

use crate::{
    Pokemon,
    service::{Language, SelectLanguagePolicy},
};

/// Select Yoda language for each legendary pokemom or that come from "cave"
pub struct CaveAndLegendarySpeakAsYoda;

impl SelectLanguagePolicy for CaveAndLegendarySpeakAsYoda {
    fn select(&self, p: &Pokemon) -> Language {
        if p.is_legendary || p.habitat.as_ref().filter(|&h| "cave" == h).is_some() {
            Language::Yoda
        } else {
            Language::Shakespeare
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn pokefake() -> Pokemon {
        Pokemon {
            name: "pokefake".to_string(),
            description: "Fake Description".to_string(),
            habitat: None,
            is_legendary: false,
        }
    }

    #[fixture]
    fn legendary(mut pokefake: Pokemon) -> Pokemon {
        pokefake.is_legendary = true;
        pokefake
    }

    #[rstest]
    fn pokefake_use_shakespeare(
        mut pokefake: Pokemon,
        #[values(Some("rare"), Some("forest"), None)] habitat: Option<&str>,
        #[values("pikachu", "zubat", "nocare", "mewtwo")] name: &str,
    ) {
        pokefake.name = name.to_string();
        pokefake.habitat = habitat.map(|h| h.to_string());

        assert_eq!(
            Language::Shakespeare,
            CaveAndLegendarySpeakAsYoda.select(&pokefake)
        )
    }

    #[rstest]
    fn legendary_pokemon_use_yoda(
        mut legendary: Pokemon,
        #[values(Some("rare"), Some("forest"), None)] habitat: Option<&str>,
        #[values("pikachu", "zubat", "nocare")] name: &str,
    ) {
        legendary.name = name.to_string();
        legendary.habitat = habitat.map(|h| h.to_string());

        assert_eq!(
            Language::Yoda,
            CaveAndLegendarySpeakAsYoda.select(&legendary)
        )
    }

    #[rstest]
    fn pokemon_from_cave_use_yoda(
        mut pokefake: Pokemon,
        #[values("nocare", "zubat", "pokefake")] name: &str,
    ) {
        pokefake.name = name.to_string();

        pokefake.habitat = Some("cave".to_string());

        assert_eq!(
            Language::Yoda,
            CaveAndLegendarySpeakAsYoda.select(&pokefake)
        )
    }
}
