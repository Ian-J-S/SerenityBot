use crate::Error;
use poise::serenity_prelude::{ChannelId, Http};
use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Alert {
    category: String,
    headline: String,
}

impl Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Category: {}\n{}", self.category, self.headline)
    }
}

pub async fn alerts(http: Arc<Http>) -> Result<(), Error> {
    let mut interval = tokio::time::interval(Duration::from_mins(15));
    let channel_id = ChannelId::new(1460136483062284442);

    let mut alert_list = HashSet::new();

    loop {
        interval.tick().await;

        let client = Client::new();
        let response: Value = client
            .get("https://api.weather.gov/alerts/active?area=CA")
            .header(USER_AGENT, "rust-web-api-client")
            .send()
            .await?
            .json()
            .await?;

        if let Some(features) = response.get("features").and_then(Value::as_array) {
            for feature in features.iter() {
                let props = &feature["properties"];
                
                let category = props["category"].as_str()
                    .unwrap_or("No category").to_string();

                let headline = props["headline"].as_str()
                    .unwrap_or("No headline").to_string();

                if category == "No category" && headline == "No headline" {
                    continue;
                }

                let new_alert = Alert { category, headline };

                if alert_list.insert(new_alert.clone()) {
                    channel_id
                        .say(&*http, format!("**New alert**:\n{new_alert}"))
                        .await?;
                }
            }
        }
    }
}
