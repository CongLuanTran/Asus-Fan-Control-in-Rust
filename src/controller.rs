use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use strum_macros::Display;

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanState {
    #[strum(serialize = "enabled")]
    Enabled,
    #[strum(serialize = "auto")]
    Auto,
}

#[derive(Debug, Deserialize)]
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

impl Default for FanControllerConfig {
    fn default() -> Self {
        Self {
            t_enable: Some(70.0),
            t_auto: Some(60.0),
            interval: Some(Duration::new(5, 0)),
            delay: Some(Duration::new(30, 0)),
            b_rise: Some(0.6),
            b_drop: Some(0.4),
        }
    }
}

impl FanControllerConfig {
    pub fn load_user_config() -> Self {
        let mut cfg = Self::default();
        let cfg_path = PathBuf::from("/etc/fanctl/config.toml");
        if fs::exists(&cfg_path).is_ok_and(|b| b) {
            match fs::read_to_string(&cfg_path) {
                Ok(content) => {
                    if let Ok(user_cfg) = toml::from_str::<FanControllerConfig>(&content) {
                        cfg.merge(user_cfg);
                    }
                }
                Err(e) => {
                    eprintln!("Error: encountered error reading config file: {e}");
                }
            }
        } else {
            eprintln!(
                "Error: cannot find config file at {}, using default value",
                &cfg_path.display()
            );
        }
        cfg
    }

    fn merge(&mut self, other: Self) {
        self.t_enable = other.t_enable.or(self.t_enable);
        self.t_auto = other.t_auto.or(self.t_auto);
        self.interval = other.interval.or(self.interval);
        self.delay = other.delay.or(self.delay);
        self.b_rise = other.b_rise.or(self.b_rise);
        self.b_drop = other.b_drop.or(self.b_drop);
    }
}

/*
Use hysteresis, response delay and disproportional smoothing to make the system react fast to heat
spikes, but slow to drops.
*/
#[derive(Debug)]
pub struct FanController {
    config: FanControllerConfig,
    pub fan_state: FanState, // Current state of the fan
    smoothed_temp: f32,      // The smoothed temperature
    pub latest_temp: f32,    // The latest read temperature
    pub next_read: Instant,  // Next sensor read time
}

impl FanController {
    pub fn new(config: FanControllerConfig) -> Self {
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
        }
        .unwrap();

        // Smooth the latest value
        self.smoothed_temp = bias * temp + (1.0 - bias) * self.smoothed_temp;
        self.latest_temp = temp; // This is mostly for display

        // Hysteresis switch fan state
        if self.smoothed_temp >= self.config.t_enable.unwrap() {
            self.fan_state = FanState::Enabled;
            // Use the longer delay when turning the fan on
            self.next_read += self.config.delay.unwrap();
        } else {
            // Else use the shorter interval
            self.next_read += self.config.interval.unwrap();
            if self.smoothed_temp <= self.config.t_auto.unwrap() {
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
        Self::new(FanControllerConfig::default())
    }
}
