use std::{env, fs::File};

#[tokio::main]
async fn main() {
    dotenv::from_filename("../.env").ok();

    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1]).expect("File not found");
    let json: serde_json::Value =
        serde_json::from_reader(file).expect("File should be proper JSON");

    for command in json["commands"]
        .as_array()
        .expect("File not structured correctly")
    {
        let url = format!(
            "https://discord.com/api/v10/applications/{}/commands",
            dotenv::var("APP_ID").expect("APP_ID env variable not found")
        );

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .json(command)
            .header(
                "Authorization",
                format!(
                    "Bot {}",
                    dotenv::var("BOT_TOKEN").expect("BOT_TOKEN env variable not found")
                ),
            )
            .send()
            .await
            .expect("Post request failed");

        if !resp.status().is_success() {
            println!("Registration failed");
        }
    }
}
