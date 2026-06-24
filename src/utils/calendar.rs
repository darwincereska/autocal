use google_calendar::{Client, ClientError, Response};
use google_calendar::types::{Event, OrderBy};

pub async fn list_all_events(client: &Client, time_min: impl AsRef<str>) -> Result<Response<Vec<Event>>, ClientError> {
    match client.events().list_all(
        "primary",
        Default::default(),
        Default::default(),
        OrderBy::StartTime,
        Default::default(),
        Default::default(),
        Default::default(),
        false,
        false,
        true,
        Default::default(),
        time_min.as_ref(),
        Default::default(),
        Default::default(),
    ).await {
        Ok(events) => Ok(events),
        Err(e) => Err(e),
    }
}