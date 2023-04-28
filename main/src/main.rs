use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde_json::json;
use validator::validate_discord_signature;

mod commands;
use commands::Command;

mod discord;
use discord::InteractionObject;

mod validator;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await?;

    Ok(())
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    println!("{:?}", req);
    validate_discord_signature(req.headers(), req.body())?;
    println!("Successfully validated !");

    let data = req
        .payload::<InteractionObject>()
        .unwrap_or(None)
        .unwrap_or_default();
    println!("{:?}", data);

    let resp = match data.interaction_type {
        1 => {
            println!("ACK Ping.");
            Response::builder()
                .status(200)
                .body(
                    json!({
                        "type": 1,
                    })
                    .to_string()
                    .into(),
                )
                .map_err(Box::new)?
        }
        2 => {
            let data = data.data.unwrap();
            let command = Command::new(data).unwrap();
            println!("Responding to {:?}!", command);

            let content = command.run().await;

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "type": 4,
                        "data": {
                            "tts": false,
                            "content": content,
                            "embeds": [],
                            "allowed_mentions": { "parse": [] }
                        }
                    })
                    .to_string()
                    .into(),
                )
                .map_err(Box::new)?
        }
        _ => {
            println!("Default case.");
            Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .body(json!({"error": "Not implemented."}).to_string().into())
                .map_err(Box::new)?
        }
    };

    println!("Response sent :\n{:?}", resp);
    Ok(resp)
}
