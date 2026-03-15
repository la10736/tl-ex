use super::*;
use rstest::{fixture, rstest};
use std::collections::HashMap;

pub mod utils;

mod integration {
    use super::*;
    use crate::tests::utils::FakePokemonService;
    use actix_web::{http::StatusCode, test};

    #[fixture]
    fn empty() -> FakePokemonService {
        Default::default()
    }

    #[fixture]
    fn mewtwo(mut empty: FakePokemonService) -> FakePokemonService {
        empty.pokemon_provider.add(Pokemon {
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
        async fn exists(mewtwo: FakePokemonService) {
            let app = test::init_service(app(mewtwo.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert!(res.status().is_success());
        }

        #[rstest]
        #[actix_web::test]
        async fn does_not_exist(empty: FakePokemonService) {
            // Here we use an empty PokemonService
            let app = test::init_service(app(empty.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert_eq!(StatusCode::NOT_FOUND, res.status());
        }

        #[rstest]
        #[actix_web::test]
        async fn provider_error(#[from(empty)] mut error: FakePokemonService) {
            error.provide_pokemon_error = true;
            let app = test::init_service(app(error.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, res.status());
        }
    }

    mod pokemon_translated {
        use super::*;

        fn req(name: &str) -> actix_http::Request {
            test::TestRequest::get()
                .uri(&format!("/pokemon/translated/{name}"))
                .to_request()
        }

        #[rstest]
        #[actix_web::test]
        async fn exists(mewtwo: FakePokemonService) {
            let app = test::init_service(app(mewtwo.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert!(res.status().is_success());
        }

        #[rstest]
        #[actix_web::test]
        async fn does_not_exist(empty: FakePokemonService) {
            // Here we use an empty PokemonService
            let app = test::init_service(app(empty.into())).await;

            let req = req("mewtwo");
            let res = test::call_service(&app, req).await;

            assert_eq!(StatusCode::NOT_FOUND, res.status());
        }

        #[rstest]
        #[actix_web::test]
        async fn should_translate(mewtwo: FakePokemonService,
                                  #[values(Language::Shakespeare, Language::Yoda)] language: Language,
        ) {
            let mewtwo_desc = mewtwo.description("mewtwo").unwrap();
            let app = test::init_service(app(mewtwo.into())).await;

            let req = req("mewtwo");
            let res: Pokemon = test::call_and_read_body_json(&app, req).await;

            assert_eq!(format!("{language:?}-{mewtwo_desc}"), res.description);
        }

        #[rstest]
        #[actix_web::test]
        async fn translate_error_should_return_orig_description(
            mut mewtwo: FakePokemonService,
            #[values(Language::Shakespeare, Language::Yoda)] language: Language,
        ) {
            mewtwo.language = language;
            mewtwo.translate_ok = false;
            let mewtwo_desc = mewtwo.description("mewtwo").unwrap();
            let app = test::init_service(app(mewtwo.into())).await;

            let req = req("mewtwo");
            let res: Pokemon = test::call_and_read_body_json(&app, req).await;

            assert_eq!(mewtwo_desc, res.description);
        }
    }
}
