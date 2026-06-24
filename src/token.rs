use google_calendar::Client;
use std::io::{self, Write};
use crate::utils::token::Token;
use url::Url;
use std::time::Duration;
use tokio::time::timeout;

pub async fn get_token(client: &mut Client) -> Result<Token, String> {
    if let Ok(token) = client.refresh_access_token().await {
        let token = Token {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
        };
        token.write()?;
        return Ok(token);
    }

    let user_consent_url = format!(
        "{}&prompt=consent",
        client.user_consent_url(&["https://www.googleapis.com/auth/calendar".to_string()])
    );
    println!("Open this URL in a browser and authorize access:\n{user_consent_url}");

    let state = extract_query_param(&user_consent_url, "state")
        .ok_or_else(|| String::from("Could not read state from consent URL"))?;
    let code = read_input("Enter the authorization code: ")?;

    println!("Exchanging authorization code for tokens...");
    println!("Using state: {state}");

    let token_result = timeout(
        Duration::from_secs(30),
        client.get_access_token(code.trim(), state.trim()),
    )
    .await;

    match token_result {
        Ok(Ok(token)) => {
            println!("Token exchange completed.");
            let token = Token {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
            };
            token.write()?;
            Ok(token)
        }
        Ok(Err(err)) => Err(err.to_string()),
        Err(_) => Err(String::from("Timed out waiting for token exchange")),
    }
}

fn read_input(prompt: &str) -> Result<String, String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    Ok(input)
}

fn extract_query_param(input: &str, key: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    url.query_pairs()
        .find(|(param, _)| param == key)
        .map(|(_, value)| value.into_owned())
}