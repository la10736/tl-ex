use super::*;
use rstest::{fixture, rstest};
use std::collections::HashMap;

pub mod utils;

mod integration {
    use super::*;
    use actix_web::{
        test,
        http::StatusCode
    };
    use crate::{
        tests::utils::InMemoryPokemonProvider,
        tests::utils::ErrorPokemonProvider
    };

    #[fixture]
    fn empty() -> InMemoryPokemonProvider {
        Default::default()
    }

    #[fixture]
    fn mewtwo(mut empty: InMemoryPokemonProvider) -> InMemoryPokemonProvider {
        empty.add(Pokemon {
            name: "mewtwo".to_string(),
            description: "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments".to_string(),
            habitat: Some("rare").map(str::to_string),
            is_legendary: true,
        });
        empty
    }

    mod pokemon {
        use super::*;

        fn req(name: &str) -> actix_http::Request {
            test::TestRequest::get()
                .uri(&format!("/pokemon/{name}"))
                .to_request()
        }

        #[rstest]
        #[actix_web::test]
        async fn exists(mewtwo: InMemoryPokemonProvider) {
            let app = test::init_service(app(mewtwo.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert!(res.status().is_success());
        }

        #[rstest]
        #[actix_web::test]
        async fn does_not_exist(empty: InMemoryPokemonProvider) {
            // Here we use an empty PokemonService
            let app = test::init_service(app(empty.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert_eq!(StatusCode::NOT_FOUND, res.status());
        }

        #[rstest]
        #[actix_web::test]
        async fn provider_error() {
            // Here we use an empty PokemonService
            let app = test::init_service(app(ErrorPokemonProvider.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, res.status());
        }
    }
}
