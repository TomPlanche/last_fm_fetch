#[path = "url_builder.rs"]
mod url_builder;

use dotenv::dotenv;
use reqwest::Error;
use std::env;
use url_builder::Url;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let last_fm_api_key =
        env::var("LAST_FM_API_KEY").expect("LAST_FM_API_KEY must be set in .env file");

    println!("Creating base URL");
    let base_url = Url::new("https://ws.audioscrobbler.com/2.0/").add_args(vec![
        ("api_key", &last_fm_api_key),
        ("format", "json"),
        ("user", "tom_planche"),
    ]);
    println!("Base URL created: {}", base_url.build());

    println!("Creating URL to get loved tracks");
    let get_top_tracks = base_url
        .add_args(vec![
            ("method", "user.getlovedtracks"),
            ("limit", "1"),
            ("page", "1"),
        ])
        .build();
    println!("URL to get loved tracks created: {}", get_top_tracks);

    Ok(())
}
