use crate::{Error, config::Config};
use chrono::Local;
use poise::serenity_prelude::{ChannelId, Http};
use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Alert {
    category: String,
    headline: String,
    description: String,
}

impl Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Category: {}\n{}\n{}", self.category, self.headline, self.description)
    }
}

/// Runs in the background of the bot if configured.
/// Pulls weather alerts from the nws.gov API and sends
/// them to the preconfigured channel.
pub async fn alerts(http: Arc<Http>, cfg: Config) -> Result<(), Error> {
    let mut interval = tokio::time::interval(cfg.alerts.check_interval);
    let channel_id = ChannelId::new(cfg.alerts.alerts_channel);

    // List of already seen alerts
    let mut alert_list = HashSet::new();

    loop {
        interval.tick().await;

        if let Some(quiet_hours) = &cfg.quiet_hours
            && quiet_hours.is_quiet(Local::now().time()) {
            continue;
        }

        let areas = cfg.alerts.areas.join(",");

        let client = Client::new();
        let response: Value = client
            .get(format!("https://api.weather.gov/alerts/active?zone={areas}"))
            .header(USER_AGENT, "rust-web-api-client")
            .send()
            .await?
            .json()
            .await?;

        if let Some(features) = response.get("features").and_then(Value::as_array) {
            for feature in features.iter() {
                let props = &feature["properties"];
                
                let category = props["category"].as_str()
                    .unwrap_or("").to_string();

                // Skip categories not in configuration
                if !cfg.alerts.alert_types.contains(&category) {
                    continue;
                }

                let headline = props["headline"].as_str()
                    .unwrap_or("").to_string();

                let description = props["description"].as_str()
                    .unwrap_or("").to_string();

                if category.is_empty() && headline.is_empty() && description.is_empty() {
                    continue;
                }

                let new_alert = Alert { category, headline, description };

                if alert_list.insert(new_alert.clone()) {
                    channel_id
                        .say(&*http, format!("**New alert**:\n{new_alert}"))
                        .await?;
                }
            }
        }
    }
}
