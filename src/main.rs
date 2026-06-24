mod config;
mod jobs;
mod token;
mod utils;

use crate::config::Config;
use crate::jobs::start;
use crate::utils::token::Token;
use google_calendar::Client;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let config = Config::load().expect("Error loading config");
    let client_config = config.client.clone();

    let token = Token::load().ok();

    let mut client = if let Some(token) = token.as_ref() {
        Client::new(
            client_config.client_id.clone(),
            client_config.client_secret.clone(),
            client_config.redirect_uri.clone(),
            token.access_token.clone(),
            token.refresh_token.clone(),
        )
    } else {
        Client::new(
            client_config.client_id.clone(),
            client_config.client_secret.clone(),
            client_config.redirect_uri.clone(),
            String::new(),
            String::new(),
        )
    };

    client.set_auto_access_token_refresh(true);

    let token = if let Some(token) = token {
        token
    } else {
        let token = match token::get_token(&mut client).await {
            Ok(token) => token,
            Err(err) => panic!("Error getting token: {err}"),
        };

        if let Err(err) = token.write() {
            eprintln!("Warning: could not save token: {err}");
        }
        token
    };

    println!("Token ready, starting job loop...");
    let _ = token;

    start(client, config).await;
}
