use regex::Regex;
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
        if let Some(content) = self.data.get_content() {
            match self.command_type {
                CommandType::Ping => String::from("Pong!"),
                CommandType::Pong => String::from("Ping!"),
                CommandType::Run => run_code_command(&content)
                    .await
                    .unwrap_or(String::from("Error")),
            }
        } else {
            String::from("ERROR: Couldn't get message content.")
        }
    }
}

async fn run_code_command(content: &str) -> Option<String> {
    let re = Regex::new(r"```(\w*)\n([\w\W]*)```").unwrap();
    let mut reply = String::new();

    for cap in re.captures_iter(content) {
        let language = &cap[1];
        let code = &cap[2];

        let code_response = run_code(language, code).await;

        let validated_response = if let Some(response) = code_response.clone().ok() {
            if response.matches('\n').count() < 25 {
                format!("```\n{response}```\n")
            } else {
                String::from("ERROR: Output contained too many lines.")
            }
        } else {
            code_response.err()?
        };

        reply.push_str(&validated_response);
    }

    if reply.len() < 2000 {
        Some(reply)
    } else {
        Some(String::from("ERROR: Output contained too many characters."))
    }
}

async fn run_code(language: &str, code: &str) -> Result<String, String> {
    match language.to_lowercase().as_str() {
        "rust" => Ok(run_rust(code).await),
        "c" | "go" | "cpp" | "java" | "cs" | "r" => Ok(run_other(language, code).await),
        "js" | "javascript" => Ok(run_other("node", code).await),
        "ts" | "typescript" => Ok(run_other("ts", code).await),
        "py" | "python" => Ok(run_other("py", code).await),
        "" => Err("ERROR: No language specified.\nHint: '```<language>'".into()),
        _ => Err("ERROR: Unsupported language.".into()),
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct OtherApiResponse {
    success: Option<bool>,
    data: OtherData,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct OtherData {
    output: Option<String>,
}

async fn run_other(language: &str, code: &str) -> String {
    let mut map = serde_json::Map::new();
    map.insert("code".into(), code.into());
    map.insert("codeId".into(), "".into());
    map.insert("input".into(), "".into());
    map.insert("language".into(), language.into());

    let client = reqwest::Client::new();
    let response = client
        .post("https://api2.sololearn.com/v2/codeplayground/v2/compile")
        .json(&map)
        .send()
        .await
        .unwrap()
        .json::<OtherApiResponse>()
        .await
        .unwrap();

    if let Some(success) = response.success {
        if success {
            response
                .data
                .output
                .unwrap_or(String::from("ERROR: API Error."))
        } else {
            String::from("ERROR: Code failed.")
        }
    } else {
        String::from("ERROR: API Error.")
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct RustApiResponse {
    success: Option<bool>,
    stdout: Option<String>,
    stderr: Option<String>,
    error: Option<String>,
}

async fn run_rust(code: &str) -> String {
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
        .json::<RustApiResponse>()
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

#[cfg(test)]
mod tests {
    use super::run_code_command;

    #[tokio::test]
    async fn run_valid_rust() {
        let code = String::from("```rust\nfn main() {\nprintln!(\"Hello\");\n}\n```\n");
        assert_eq!(
            String::from("```\nHello\n```\n"),
            run_code_command(&code).await.unwrap()
        );
    }

    #[tokio::test]
    async fn run_valid_python() {
        let code = String::from("```py\nprint(\"Hello\")\n```\n");
        assert_eq!(
            String::from("```\nHello\n```\n"),
            run_code_command(&code).await.unwrap()
        );
    }

    #[tokio::test]
    async fn run_no_language() {
        let code = String::from("```\nfn main() {\nprintln!(\"Hello\");\n}\n```\n");
        assert_eq!(
            String::from("ERROR: No language specified.\nHint: '```<language>'"),
            run_code_command(&code).await.unwrap()
        );
    }

    #[tokio::test]
    async fn run_unsupported_language() {
        let code = String::from("```random_lang\nfn main() {\nprintln!(\"Hello\");\n}\n```\n");
        assert_eq!(
            String::from("ERROR: Unsupported language."),
            run_code_command(&code).await.unwrap()
        );
    }
}
