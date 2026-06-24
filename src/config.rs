use log::{error, info};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct Config {
    /// Refresh interval (in seconds)
    pub interval: i64,
    
    /// Client config
    pub client: ClientConfig,

    /// Notification settings
    pub notifications: Option<NotificationSettings>,

    /// Events
    pub events: Option<Vec<Event>>,
}

#[derive(Deserialize, Clone)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize, Clone)]
pub struct NotificationSettings {
    pub logging: Option<bool>,

    #[allow(dead_code)]
    pub gotify: Option<GotifySettings>,
}

pub enum NotificationType {
    Normal,
    Error,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct GotifySettings {
    pub url: String,
    pub token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Event {
    /// Event name
    pub name: String,

    /// Optional label
    #[allow(dead_code)]
    pub label: Option<String>,

    /// Toggle regex
    pub use_regex: Option<bool>,

    #[serde(skip_deserializing)]
    pub regex_pattern: Option<Regex>,

    /// Toggle exact name or contains
    pub exact_name: Option<bool>,

    pub participants: Option<Vec<String>>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = Path::new("./config.toml");

        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut config: Config = toml::from_str(&content).map_err(|e| e.to_string())?;

        // Precompile regex expressions
        config.precompile_regex()?;

        Ok(config)
    }

    fn precompile_regex(&mut self) -> Result<(), String> {
        if let Some(events) = self.events.as_mut() {
            for event in events {
                if event.use_regex == Some(true) {
                    let re = Regex::new(&event.name).map_err(|e| e.to_string())?;
                    event.regex_pattern = Some(re);
                }
            }
        }

        Ok(())
    }
}

impl NotificationSettings {
    pub fn notify(&self, message: impl AsRef<str>, notification_type: NotificationType) {
        if self.logging == Some(true) {
            match notification_type {
                NotificationType::Normal => {
                    info!("{}", message.as_ref());
                }

                NotificationType::Error => {
                    error!("{}", message.as_ref());
                }
            }
        }

        // TODO: Implement gotify
    }
}
