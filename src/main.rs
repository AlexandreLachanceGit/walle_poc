use lambda_runtime::handler_fn;

mod lib;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    let func = handler_fn(handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn handler(req: lib::Request, _ctx: lambda_runtime::Context) -> lib::Response {
    Ok(lib::SuccessResponse {
        body: "Pong!".to_string(),
    })
}
