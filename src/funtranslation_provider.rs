//! A simple implementation that access to funtranslations.mercxry.me API to
//! implement a TranslationProvider

use crate::{Language, ServiceError, TranslationProvider};
use async_trait::async_trait;
use log::warn;
use reqwest::{Error, Response};
use serde::Deserialize;
use std::collections::HashMap;

const FUNTRANSLATE_ENDPOINT_URL: &str = "https://api.funtranslations.mercxry.me/v1/translate";

#[derive(Deserialize)]
#[allow(dead_code)]
struct TranslatedContent {
    translated: String,
    text: String,
    #[serde(rename = "translation")]
    language: Language,
}
#[derive(Deserialize)]
struct TranslationResponse {
    contents: TranslatedContent,
}

pub struct FunTranslator {
    client: reqwest::Client,
}

impl Default for FunTranslator {
    fn default() -> Self {
        Self {
            client: Default::default(),
        }
    }
}

#[async_trait]
impl TranslationProvider for FunTranslator {
    async fn translate(&self, lang: Language, body: &str) -> Result<String, ServiceError> {
        let lang = match lang {
            Language::Yoda => "yoda",
            Language::Shakespeare => "shakespeare",
        };
        let mut request = HashMap::new();
        request.insert("text", body);

        let response = self
            .fetch_translation(lang, &mut request)
            .await
            .inspect_err(|e| warn!("Fetch translation error: {:?}", e))
            .map_err(|e| ServiceError::Unknown {
                error: e.to_string(),
            })?
            .json::<TranslationResponse>()
            .await
            .inspect_err(|e| warn!("Deserialize translation error: {:?}", e))
            .map_err(|e| ServiceError::Unknown {
                error: e.to_string(),
            })?;

        Ok(response.contents.translated)
    }
}

impl FunTranslator {
    async fn fetch_translation(
        &self,
        lang: &str,
        request: &mut HashMap<&str, &str>,
    ) -> Result<Response, Error> {
        self.client
            .post(format!("{FUNTRANSLATE_ENDPOINT_URL}/{lang}"))
            .json(&request)
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::yoda(
        Language::Yoda,
        "By a one of powerful knowledge after years of terrible gene fusing and DNA crafting trials, made it was, young one."
    )]
    #[case::shakespeare(
        Language::Shakespeare,
        "Hark! it wast fashioned by a alchemist after many a year of most dreadful alchemical fusion and DNA alchemy trials."
    )]
    #[actix_rt::test]
    async fn translate(#[case] language: Language, #[case] expected: &str) {
        let service = FunTranslator::default();
        let message = "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.";

        let translated = service.translate(language, &message).await.unwrap();

        assert_eq!(expected, translated);
    }
}
