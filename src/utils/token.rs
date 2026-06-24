use google_calendar::AccessToken;

#[derive(Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

impl Token {
    pub fn write(&self) -> Result<(), String> {
        if self.access_token.trim().is_empty() {
            return Err(String::from("Refusing to write empty access token"));
        }

        let content = format!("{}\n{}", self.access_token, self.refresh_token);
        std::fs::write("./token.txt", content).map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        let content = std::fs::read_to_string("./token.txt").map_err(|err| err.to_string())?;
        let mut lines = content.lines();
        let access_token = lines.next().unwrap_or_default().trim().to_string();
        let refresh_token = lines.next().unwrap_or_default().trim().to_string();

        if access_token.is_empty() {
            return Err(String::from("No token found"));
        }

        Ok(Self {
            access_token,
            refresh_token,
        })
    }

    pub fn refresh(refreshed: AccessToken) {
        let existing = Self::load().expect("Error loading token");
        let token = Self {
            access_token: refreshed.access_token,
            refresh_token: if refreshed.refresh_token.is_empty() {
                existing.refresh_token
            } else { refreshed.refresh_token },
        };
        token.write().expect("Error writing token");
    }
}
