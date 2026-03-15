use super::*;
#[derive(Default, Clone)]
/// Trivial InMemory implementation
pub struct InMemoryPokemonProvider(HashMap<String, Pokemon>);

impl From<InMemoryPokemonProvider> for Option<PokemonService> {
    fn from(provider: InMemoryPokemonProvider) -> Self {
        Some(PokemonService {
            provider: Box::new(provider),
        })
    }
}

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

#[derive(Default, Clone)]
/// Just return error every time
pub struct ErrorPokemonProvider;

impl From<ErrorPokemonProvider> for Option<PokemonService> {
    fn from(provider: ErrorPokemonProvider) -> Self {
        Some(PokemonService {
            provider: Box::new(provider),
        })
    }
}

#[async_trait]
impl PokemonProvider for ErrorPokemonProvider {
    async fn pokemon(&self, name: &str) -> Result<Pokemon, ServiceError> {
        Err(ServiceError::Unknown {
            error: format!("Pokemon name: {}", name),
        })
    }
}
