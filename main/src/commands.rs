use serde::Deserialize;

use crate::discord::InteractionData;

#[derive(Debug)]
enum CommandType {
    Ping,
    Pong,
    Run,
}

#[derive(Debug)]
pub struct Command {
    command_type: CommandType,
    data: InteractionData,
}

impl Command {
    pub fn new(data: InteractionData) -> Option<Command> {
        Some(Command {
            command_type: match data.name.as_str() {
                "ping" => CommandType::Ping,
                "pong" => CommandType::Pong,
                "run" => CommandType::Run,
                _ => return None,
            },
            data,
        })
    }

    pub async fn run(&self) -> String {
        match self.command_type {
            CommandType::Ping => String::from("Pong!"),
            CommandType::Pong => String::from("Ping!"),
            CommandType::Run => run_code_command(&self.data)
                .await
                .unwrap_or(String::from("Error")),
        }
    }
}

async fn run_code_command(data: &InteractionData) -> Option<String> {
    Some(run_code(&data.get_content()?).await)
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ApiResponse {
    success: Option<bool>,
    stdout: Option<String>,
    stderr: Option<String>,
    error: Option<String>,
}

async fn run_code(code: &str) -> String {
    let mut map = serde_json::Map::new();
    map.insert("channel".into(), "stable".into());
    map.insert("mode".into(), "debug".into());
    map.insert("edition".into(), "2021".into());
    map.insert("crateType".into(), "bin".into());
    map.insert("tests".into(), false.into());
    map.insert("code".into(), code.into());
    map.insert("backtrace".into(), false.into());

    let client = reqwest::Client::new();
    let response = client
        .post("https://play.rust-lang.org/execute")
        .json(&map)
        .send()
        .await
        .unwrap()
        .json::<ApiResponse>()
        .await
        .unwrap();

    if response.error.is_some() {
        response.error.unwrap_or(String::from("error is some"))
    } else if response.success.unwrap() {
        response.stdout.unwrap_or(String::from("stdout"))
    } else {
        response.stderr.unwrap_or(String::from("stderr"))
    }
}
