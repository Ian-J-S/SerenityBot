use std::collections::HashSet;
use std::fs;

use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub alerts_channel: Option<String>,
    pub alert_types: HashSet<String>,
    pub quiet_hours: QuietHours,
}

#[derive(Debug, Deserialize)]
pub struct QuietHours {
    start: NaiveTime,
    end: NaiveTime,
}

impl QuietHours {
    fn is_quiet(&self, now: NaiveTime) -> bool {
        if self.start <= self.end {
            // Same-day
            now >= self.start && now < self.end
        } else {
            // Overnight (ex 22:00–07:00)
            now >= self.start || now < self.end
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.toml")?;

    let config: Config = toml::from_str(&config_str)?;

    #[cfg(debug_assertions)] {
        println!("Alerts channel: {:?}", config.alerts_channel);
        println!("Alert types: {:?}", config.alert_types);
        println!("Quiet hours: {:?}", config.quiet_hours);
    }

    Ok(config)
}
