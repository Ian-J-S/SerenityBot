//! Defines the configuration of the bot.
//!
//! Structs here represent the keys and values of the TOML
//! file used to configure the bot. If no TOML is found,
//! or it is not able to be parsed, the bot will run without
//! sending alerts.
//!
//! The config file must be named 'config.toml' and must
//! be placed in the same directory as the bot.

use std::{
    collections::HashSet,
    fs,
    time::Duration,
};

use chrono::NaiveTime;
use serde::Deserialize;
use serde_with::{DurationSeconds, serde_as};

/// Main bot configuration struct.
///
/// `config.toml` keys follow the variable names
/// in the struct.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub alerts: AlertConfig,
    pub quiet_hours: QuietHours,
}

/// TOML key `[alerts]`
/// Used to configure that alerts are sent to, the types of NWS alerts displayed, 
/// and how often to check for new alerts.
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct AlertConfig {
    pub alerts_channel: u64,
    pub alert_types: HashSet<String>,

    #[serde_as(as = "DurationSeconds<u64>")]
    pub check_interval: Duration,
}

/// TOML key `[quiet_hours]`
/// Schedule times that the bot does not send any alerts.
#[derive(Debug, Deserialize)]
pub struct QuietHours {
    start: NaiveTime,
    end: NaiveTime,
}

impl QuietHours {
    /// Check if the current time is within quiet hours.
    pub fn is_quiet(&self, now: NaiveTime) -> bool {
        if self.start <= self.end {
            // Same-day
            now >= self.start && now < self.end
        } else {
            // Overnight (ex 22:00–07:00)
            now >= self.start || now < self.end
        }
    }
}

/// Load the configuration file from the relative path 'config.toml'
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("config.toml")?;

    let config: Config = toml::from_str(&config_str)?;

    #[cfg(debug_assertions)] {
        println!("Alerts: {:?}", config.alerts);
        println!("Quiet hours: {:?}", config.quiet_hours);
    }

    Ok(config)
}
