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

    pub async fn run(&self) -> Result<String, String> {
        if let Some(content) = self.data.get_content() {
            match self.command_type {
                CommandType::Ping => Ok(String::from("Pong!")),
                CommandType::Pong => Ok(String::from("Ping!")),
                CommandType::Run => run_code_command(&content).await,
            }
        } else {
            Err(String::from("ERROR: This command can only be used through the context menu on a message containing a code block. Use the ... in the upper right corner of the message, Apps -> Run."))
        }
    }
}

async fn run_code_command(content: &str) -> Result<String, String> {
    let re = Regex::new(r"```(\w*)\n([\w\W]*)```").unwrap();

    let cap = re.captures_iter(content).next().unwrap();
    let language = &cap[1];
    let code = &cap[2];

    let code_response = run_code(language, code).await?;

    let validated_response = if code_response.matches('\n').count() < 25 {
        format!("```\n{code_response}```\n")
    } else {
        return Err(String::from("ERROR: Output contained too many lines."));
    };

    if validated_response.len() < 2000 {
        Ok(validated_response)
    } else {
        Err(String::from("ERROR: Output contained too many characters."))
    }
}

async fn run_code(language: &str, code: &str) -> Result<String, String> {
    match language.to_lowercase().as_str() {
        "rust" => Ok(run_rust(code).await),
        "c" | "go" | "cpp" | "java" | "cs" | "r" => run_other(language, code).await,
        "js" | "javascript" => run_other("node", code).await,
        "ts" | "typescript" => run_other("ts", code).await,
        "py" | "python" => run_other("py", code).await,
        "" => Err("ERROR: No language specified.\nHint: ```<language>".into()),
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

async fn run_other(language: &str, code: &str) -> Result<String, String> {
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
            Ok(response.data.output.unwrap())
        } else {
            Err(String::from("ERROR: Code failed."))
        }
    } else {
        Err(String::from("ERROR: API Error."))
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
            String::from("ERROR: No language specified.\nHint: ```<language>"),
            run_code_command(&code).await.err().unwrap()
        );
    }

    #[tokio::test]
    async fn run_unsupported_language() {
        let code = String::from("```random_lang\nfn main() {\nprintln!(\"Hello\");\n}\n```\n");
        assert_eq!(
            String::from("ERROR: Unsupported language."),
            run_code_command(&code).await.err().unwrap()
        );
    }
}
