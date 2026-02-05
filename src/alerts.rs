use crate::{Error, config::Config};
use chrono::{DateTime, Local, NaiveDateTime};
use poise::serenity_prelude::{ChannelId, Http};
use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    sync::Arc, time::Duration,
};
use tokio::sync::watch::Receiver;
use tracing::{info, error};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Alert {
    category: String,
    headline: String,
    description: String,
    end_time: NaiveDateTime,
}

impl Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Category: {}\n{}\n{}", self.category, self.headline, self.description)
    }
}

/// Removes NWS alerts that are past their specified end time.
fn clear_ended_alerts(alert_list: &mut HashSet<Alert>) {
    alert_list.retain(|a| {
        let now = Local::now().naive_local();
        a.end_time > now
    });
}

/// Runs in the background of the bot if configured.
/// Pulls weather alerts from the nws.gov API and sends
/// them to the preconfigured channel.
pub async fn alerts(http: Arc<Http>, cfg: Config, mut rx: Receiver<Config>)
-> Result<(), Error> {
    // List of already seen alerts
    let mut alert_list = HashSet::new();

    let client = Client::new();

    let mut interval = tokio::time::interval(cfg.alerts.check_interval);
    let mut channel_id = ChannelId::new(cfg.alerts.alerts_channel);
    let mut areas = cfg.alerts.areas.join(",");
    let mut alert_types = cfg.alerts.alert_types.clone();
    let mut quiet_hours = cfg.quiet_hours.clone();

    // Check for ended alerts every 1 hour
    let mut cleanup_interval = tokio::time::interval(Duration::from_hours(1));
    
    info!("Listening for NWS alerts");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Some(quiet_hours) = &quiet_hours
                    && quiet_hours.is_quiet(Local::now().time()) {
                    continue;
                }

                // Perform the request and handle any errors without terminating the task
                let resp_res = client
                    .get(format!("https://api.weather.gov/alerts/active?zone={areas}"))
                    .header(USER_AGENT, "rust-web-api-client")
                    .send()
                    .await;

                let response_value: Value = match resp_res {
                    Ok(resp) => match resp.json().await {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Failed to parse NWS response JSON: {:#}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        error!("Failed to fetch NWS alerts: {:#}", e);
                        continue;
                    }
                };

                if let Some(features) = response_value.get("features").and_then(Value::as_array) {
                    for feature in features.iter() {
                        let props = &feature["properties"];

                        let category = props["category"].as_str()
                            .unwrap_or("").to_string();
                        // Skip categories not in configuration
                        if !alert_types.contains(&category) {
                            continue;
                        }

                        let headline = props["headline"].as_str()
                            .unwrap_or("").to_string();
                        let description = props["description"].as_str()
                            .unwrap_or("").to_string();

                        if category.is_empty() && headline.is_empty() && description.is_empty() {
                            continue;
                        }

                        // Parse the end time safely
                        let end_time = props["ends"].as_str()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.naive_local());

                        let end_time = match end_time {
                            Some(t) => t,
                            None => {
                                error!("Alert missing or invalid 'ends' time, skipping: {:?}", props);
                                continue;
                            }
                        };

                        let new_alert = Alert { category, headline, description, end_time };
                        if alert_list.insert(new_alert.clone())
                            && let Err(e) = channel_id
                                .say(&*http, format!("**New alert**:\n{new_alert}"))
                                .await {
                            error!("Failed to send alert to channel: {e}");
                        }
                    }
                }
            }
            _ = cleanup_interval.tick() => {
                let old_len = alert_list.len();
                clear_ended_alerts(&mut alert_list);
                info!("Cleared {} stale alerts", old_len - alert_list.len());
            }
            // Reload config when signalled
            _ = rx.changed() => {
                info!("Config updated, applying new settings");
                let cfg = rx.borrow_and_update();
                #[cfg(debug_assertions)]
                println!("New config: {:?}", *cfg);
                
                // Update config values
                interval = tokio::time::interval(cfg.alerts.check_interval);
                channel_id = ChannelId::new(cfg.alerts.alerts_channel);
                areas = cfg.alerts.areas.join(",");
                alert_types = cfg.alerts.alert_types.clone();
                quiet_hours = cfg.quiet_hours.clone();
            }
        }
    }
}
