use std::collections::HashMap;
use std::time::Duration;
use crate::config::{Config, NotificationSettings, NotificationType};
use google_calendar::types::{Event, EventAttendee, SendUpdates};
use crate::utils::calendar::list_all_events;
use chrono::{TimeZone, Utc};
use tokio::sync::mpsc::{Sender, channel};
use crate::utils::token::Token;

pub async fn start(client: google_calendar::Client, config: Config) {
    // Setup time
    let t = Utc::now().to_rfc3339();

    // Create events channel
    let (events_tx, mut events_rx) = channel::<Event>(10);

    // Indefinitely check for new events with timeout
    let checker_config = config.clone();
    let checker_srv = client.clone();
    let checker_notifications = config.notifications.clone();
    let event_checker = tokio::spawn(async move {
        if let Err(err) = check_for_new_events(checker_srv, checker_config, t, events_tx).await {
            if let Some(notifications) = checker_notifications {
                notifications.notify(format!("Event checker stopped: {err}"), NotificationType::Error);
            }
        } else if let Some(notifications) = checker_notifications {
            notifications.notify("Event checker stopped unexpectedly", NotificationType::Error);
        }
    });

    let event_rules = config.events.as_deref().unwrap_or_default();

    // Wait for events
    while let Some(event) = events_rx.recv().await {
        // Iterate through specified events in the config
        for rule in event_rules {
            let use_regex = rule.use_regex == Some(true);
            let exact_name = rule.exact_name == Some(true);

            // Store if the event name matches the rule
            let is_match: bool;

            // Check if the event matches the rule
            if use_regex {
                is_match = rule.regex_pattern.clone().unwrap().is_match(&event.summary)
            } else if exact_name {
                is_match = rule.name == event.summary
            } else {
                is_match = event.summary.contains(&rule.name)
            }

            if is_match {
                if let Some(notifications) = config.notifications.as_ref() {
                    notifications.notify(
                        format!("Found match for: {}({})", &event.summary, &event.id),
                        NotificationType::Normal,
                    );
                }

                // Add participants to the event
                if let Some(participants) = rule.participants.as_ref() {
                    if !participants.is_empty() {
                        let client = client.clone();
                        let event = event.clone();
                        let participants = participants.clone();
                        let notifications = config.notifications.clone();

                        // Add participants in the background
                        tokio::spawn(async move {
                            add_participants_to_event(client, event, notifications, participants)
                                .await;
                        });
                    }
                }

                // Stop checking other rules for event if this one matches
                break;
            }
        }
    }

    // Keep the process alive as long as the checker is running.
    let _ = event_checker.await;
}

/// Loads user's primary calendar and sends new events to channel
///
/// # Arguments
/// * `client`  - Google Calendar client
/// * `config`   - AutoCal config
/// * `time_min` - A time string in **RFC3339** format
/// * `sender`   - Channel for sending events
async fn check_for_new_events(
    client: google_calendar::Client,
    config: Config,
    mut time_min: String,
    sender: Sender<Event>,
) -> Result<(), String> {
    loop {
        if let Some(notifications) = config.notifications.as_ref() {
            notifications.notify("Checking for new events...", NotificationType::Normal);
        }

        // Get events
        if let Some(true) = client.is_expired().await {
            let token = client.refresh_access_token().await.expect("Error refreshing access token");
            Token::refresh(token);
        }

        let events = match list_all_events(&client, &time_min).await {
            Ok(events) => events,
            Err(e) if e.to_string().contains("UNAUTHENTICATED") => {
                let token = client.refresh_access_token().await.expect("Error refreshing access token");
                Token::refresh(token);
                list_all_events(&client, &time_min).await.expect("Error listing events")
            },
            Err(e) => {
                if let Some(notifications) = config.notifications.as_ref() {
                    notifications.notify(format!("Event poll failed: {e}"), NotificationType::Error);
                }
                tokio::time::sleep(Duration::from_secs(config.interval as u64)).await;
                continue;
            }
        };

       let mut newest_start = None;

        if events.status.is_success() {
            // Send the events to the channel
            for event in &events.body {
                // Get latest time for all-day or date-time
                if let Some(start) = event.start.as_ref() {
                    let candidate = start
                        .date_time
                        .as_ref()
                        .cloned()
                        .or_else(|| start.date.as_ref().map(|d| {
                            let next_day = d.succ_opt().unwrap();
                            Utc.from_utc_datetime(&next_day.and_hms_opt(0, 0, 0).unwrap())
                        }));

                    if let Some(ts) = candidate{
                        newest_start = Some(match newest_start {
                            Some(curr) if curr > ts => curr,
                            _ => ts,
                        });
                    }
                }

                // Send the event
                sender
                    .send(event.clone())
                    .await
                    .map_err(|e| e.to_string())?;
            }

            // Update time to the last event checked to avoid processing the same events again
            if let Some(ts) = newest_start {
                time_min = ts.to_rfc3339();
            }
        }

        // Wait for the next check
        tokio::time::sleep(Duration::from_secs(config.interval as u64)).await;
    }
}

async fn add_participants_to_event(
    client: google_calendar::Client,
    mut event: Event,
    notifications: Option<NotificationSettings>,
    participants: Vec<String>,
) {
    // Map of existing attendees
    let mut existing_attendees = HashMap::<String, bool>::new();
    for attendee in &event.attendees {
        existing_attendees.insert(attendee.email.clone(), true);
    }

    // Flag to track if we need to update the event
    let mut should_update = false;

    // Add new participants if they are not already in the event
    for participant in participants {
        if !existing_attendees.contains_key(&participant) {
            // Add the participant
            event.attendees.push(EventAttendee {
                additional_guests: 0,
                comment: Default::default(),
                display_name: Default::default(),
                email: participant.clone(),
                id: Default::default(),
                optional: false,
                organizer: false,
                resource: false,
                response_status: Default::default(),
                self_: false,
            });

            should_update = true;

            if let Some(notifications) = notifications.as_ref() {
                notifications.notify(format!("Adding new participant: {} to: {}({})", &participant, &event.summary, &event.id), NotificationType::Normal);
            }
        }
    }

    // If added new attendees, update the event in Google Calendar
    if should_update {
        if let Some(true) = client.is_expired().await {
            let token = client.refresh_access_token().await.expect("Error refreshing access token");
            Token::refresh(token);
        }

        // Send the update request
        let update = client.events().update(
            "primary",
            &event.id,
            Default::default(),
            10,
            true,
            SendUpdates::All,
            true,
            &event
        ).await.map_err(|e| e.to_string());

        // Send notifications
        if update.is_ok() {
           if let Some(notifications) = notifications.as_ref() {
               notifications.notify(format!("Updated: {}({})", &event.summary, &event.id), NotificationType::Normal);
           }
        }
    }
}