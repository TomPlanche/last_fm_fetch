use dotenv::dotenv;
use reqwest::Error;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let last_fm_api_key =
        env::var("LAST_FM_API_KEY").expect("LAST_FM_API_KEY must be set in .env file");

    Ok(())
}
