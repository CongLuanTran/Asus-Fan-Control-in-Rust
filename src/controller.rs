use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fmt, fs};

use serde::Deserialize;
use strum::Display;
use tracing::{error, info};

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanState {
    #[strum(serialize = "enabled")]
    Enabled,
    #[strum(serialize = "auto")]
    Auto,
}

#[derive(Debug, Deserialize, Default)]
pub struct FanControllerConfig {
    #[serde(alias = "threshold_enable")]
    t_enable: Option<f32>, // Temperature to turn on at full speed, default to 70
    #[serde(alias = "threshold_auto")]
    t_auto: Option<f32>, // Temperature to return to automatic control, default to 60
    interval: Option<Duration>, // How frequent does the program read sensor value, default to 5s
    delay: Option<Duration>,    // The delay before the fan can be returned to auto, default to 30s
    #[serde(alias = "bias_rise")]
    b_rise: Option<f32>, // Bias for new temperature read when it rises
    #[serde(alias = "bias_drop")]
    b_drop: Option<f32>, // Bias for new temperature read when it drops
}

#[derive(Debug)]
pub struct RuntimeConfig {
    t_enable: f32,
    t_auto: f32,
    interval: Duration,
    delay: Duration,
    b_rise: f32,
    b_drop: f32,
}

impl fmt::Display for RuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "t_enable: {}, t_auto: {}, interval {:?}, delay: {:?}, b_rise: {}, b_drop: {}",
            self.t_enable, self.t_auto, self.interval, self.delay, self.b_rise, self.b_drop
        )
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            t_enable: 70.0,
            t_auto: 60.0,
            interval: Duration::new(5, 0),
            delay: Duration::new(30, 0),
            b_rise: 0.6,
            b_drop: 0.4,
        }
    }
}

impl From<FanControllerConfig> for RuntimeConfig {
    fn from(cfg: FanControllerConfig) -> Self {
        let defaults = RuntimeConfig::default();

        Self {
            t_enable: cfg.t_enable.unwrap_or(defaults.t_enable),
            t_auto: cfg.t_auto.unwrap_or(defaults.t_auto),
            interval: cfg.interval.unwrap_or(defaults.interval),
            delay: cfg.delay.unwrap_or(defaults.delay),
            b_rise: cfg.b_rise.unwrap_or(defaults.b_rise),
            b_drop: cfg.b_drop.unwrap_or(defaults.b_drop),
        }
    }
}

impl FanControllerConfig {
    pub fn load_user_config() -> Self {
        let cfg_path = PathBuf::from("/etc/fanctl/config.toml");

        fs::read_to_string(&cfg_path)
            .map_err(|e| {
                error!("Error: failed reading {}: {e}", cfg_path.display());
            })
            .ok()
            .and_then(|content| {
                toml::from_str::<FanControllerConfig>(&content)
                    .map_err(|e| {
                        error!("Error: invalid config {}: {e}", cfg_path.display());
                    })
                    .ok()
            })
            .unwrap_or_default()
    }
}

/*
Use hysteresis, response delay and disproportional smoothing to make the system react fast to heat
spikes, but slow to drops.
*/
#[derive(Debug)]
pub struct FanController {
    config: RuntimeConfig,   // Runtime configuration with defaults applied
    pub fan_state: FanState, // Current state of the fan
    smoothed_temp: f32,      // The smoothed temperature
    pub latest_temp: f32,    // The latest read temperature
    pub next_read: Instant,  // Next sensor read time
}

impl FanController {
    pub fn new(config: RuntimeConfig) -> Self {
        info!("Using config: {}", config);

        Self {
            config,
            fan_state: FanState::Auto,
            smoothed_temp: 0.0,
            latest_temp: 0.0,
            next_read: Instant::now(),
        }
    }

    pub fn update(&mut self, temp: f32) {
        // Switch bias depending on whether the temperature is rising or falling
        let bias = if temp > self.smoothed_temp {
            self.config.b_rise
        } else {
            self.config.b_drop
        };

        // Smooth the latest value
        self.smoothed_temp = bias * temp + (1.0 - bias) * self.smoothed_temp;
        self.latest_temp = temp; // This is mostly for display

        // Hysteresis switch fan state
        if self.smoothed_temp >= self.config.t_enable {
            self.fan_state = FanState::Enabled;
            // Use the longer delay when turning the fan on
            self.next_read += self.config.delay;
        } else {
            // Else use the shorter interval
            self.next_read += self.config.interval;
            if self.smoothed_temp <= self.config.t_auto {
                self.fan_state = FanState::Auto;
            }
        }
    }

    pub fn status(&self) -> String {
        format!("Temp: {}\nState: {}", self.latest_temp, self.fan_state)
    }
}

impl Default for FanController {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}
