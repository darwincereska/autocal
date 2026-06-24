# AutoCal

AutoCal watches your Google Calendar and applies participant rules when matching events appear. It polls on an interval, finds new events, and can add attendees automatically.

## What it does

- Polls a primary Google Calendar on a fixed interval
- Matches events by substring, exact name, or regex
- Adds configured participants to matching events
- Logs activity when logging is enabled

## Requirements

- Rust toolchain
- A Google Calendar API client ID, client secret, and redirect URI
- A `config.toml` file in the project root

## Install

```bash
cargo install autocal
```

## Configure

AutoCal reads `./config.toml` at startup.

```toml
# Poll interval in seconds
interval = 3600

[client]
client_id = "CLIENT_ID"
client_secret = "CLIENT_SECRET"
redirect_uri = "https://example.com/callback"

# [notifications]
# logging = true

#[notifications.gotify]
#url = "https://gotify.example.com"
#token = "GOTIFY_TOKEN"

[[events]]
name = "Example"
# label = "Optional label"
# use_regex = false
# exact_name = false
participants = ["example@email.com"]
```

### Config fields

- `interval`: poll interval in seconds
- `client.client_id`: Google Calendar OAuth client ID
- `client.client_secret`: Google Calendar OAuth client secret
- `client.redirect_uri`: OAuth redirect URI
- `notifications.logging`: writes status and error messages to the log when `true`
- `notifications.gotify`: present in the config schema, but not implemented yet in the current code
- `events`: list of event rules

### Event matching

For each event rule:

- `use_regex = true` uses `name` as a regex pattern
- `exact_name = true` requires an exact string match
- otherwise AutoCal matches if the calendar event summary contains `name`

When a rule matches, AutoCal adds every email in `participants` that is not already on the event.

## OAuth and token flow

On first start, AutoCal loads `config.toml`, then attempts to load a saved token from `token.txt`.

- If a token exists, it is reused and refreshed automatically when needed
- If no token exists, AutoCal starts the Google OAuth flow, writes the token to `token.txt`, and then begins polling

## Run

```bash
cargo run
```

Make sure `config.toml` exists in the working directory before starting the process.

## Example

If you want to match events whose summary contains `Standup` and add two participants:

```toml
[[events]]
name = "Standup"
participants = ["alice@example.com", "bob@example.com"]
```

