//! Pokedex API acceptance tests. Every test run a server on a given port and execute the request.
//!

use assert_cmd::cargo_bin;
use reqwest::StatusCode;
use rstest::{fixture, rstest};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::process;

const SERVER_STARTING_PORT: u16 = 5000;

struct CommandGuard(process::Child);

struct Server {
    port: u16,
    _cmd: CommandGuard,
}

impl Server {
    fn started(port: u16) -> Self {
        let mut cmd = CommandGuard(
            process::Command::new(cargo_bin!("tl-ex"))
                .arg("--port")
                .arg(port.to_string())
                .stdout(process::Stdio::piped())
                .stderr(process::Stdio::piped())
                .spawn()
                .unwrap(),
        );

        cmd.wait_start();

        Server { port, _cmd: cmd }
    }

    async fn pokemon(&self, name: &str) -> PokemonRequest {
        let res = reqwest::get(format!("http://localhost:{}/pokemon/{name}", self.port))
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        res.json().await.unwrap()
    }

    async fn translated(&self, name: &str) -> PokemonRequest {
        let res = reqwest::get(format!(
            "http://localhost:{}/pokemon/translated/{name}",
            self.port
        ))
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        res.json().await.unwrap()
    }
}

impl Drop for CommandGuard {
    fn drop(&mut self) {
        self.0.kill().unwrap()
    }
}

impl CommandGuard {
    fn wait_start(&mut self) {
        let out = self.0.stdout.take().unwrap();
        if !BufReader::new(out)
            .lines()
            .map(|line| line.expect("failed to obtain next line"))
            .find(|line| line.starts_with("Starting server"))
            .is_some()
        {
            panic!("server did not respond to start");
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct PokemonRequest {
    name: String,
    description: String,
    habitat: String,
    is_legendary: bool,
}

#[fixture]
fn server() -> Server {
    static PORT: std::sync::Mutex<u16> = std::sync::Mutex::new(SERVER_STARTING_PORT);

    let mut p = PORT.lock().unwrap();
    let server_port = *p;

    *p += 1;
    Server::started(server_port)
}

#[rstest]
#[case::mewtwo(
    "mewtwo",
    "It was created by a scientist after years of horrific gene splicing and DNA engineering experiments.",
    "rare",
    true
)]
#[case::pikachu(
    "pikachu",
    "When several of these POKéMON gather, their electricity could build and cause lightning storms.",
    "forest",
    false
)]
#[actix_web::test]
async fn pokemon(
    server: Server,
    #[case] name: &str,
    #[case] description: &str,
    #[case] habitat: &str,
    #[case] is_legendary: bool,
) {
    let answer = server.pokemon(name).await;

    assert_eq!(
        PokemonRequest {
            name: name.to_string(),
            description: description.to_string(),
            habitat: habitat.to_string(),
            is_legendary,
        },
        answer
    );
}

#[rstest]
#[case::legendary(
    "mewtwo",
    "By a one of powerful knowledge after years of terrible gene fusing and DNA crafting trials, made it was, young one.",
    "rare",
    true
)]
#[case::in_cave(
    "zubat",
    "Forms colonies in perpetually dark places. Approach targets, uses ultrasonic waves to identify. Curious, this is.",
    "cave",
    false
)]
#[case::not_cave_or_legendary(
    "pikachu",
    "Lo! when sundry of these POKéMON assemble, their galvanic force couldst amass and bring forth wondrous tempests of thunder.",
    "forest",
    false
)]
#[actix_web::test]
async fn translated(
    server: Server,
    #[case] name: &str,
    #[case] description: &str,
    #[case] habitat: &str,
    #[case] is_legendary: bool,
) {
    let answer = server.translated(name).await;

    let fallback_description = server.pokemon(name).await.description;

    // Use the not translated description as fallback if some error occurred during the
    // translation: this can be an issue if translation doesn't work and integration tests
    // are not implemented.
    let expected_description = if answer.description != fallback_description {
        description.to_string()
    } else {
        fallback_description
    };

    assert_eq!(
        PokemonRequest {
            name: name.to_string(),
            description: expected_description,
            habitat: habitat.to_string(),
            is_legendary,
        },
        answer
    );
}
