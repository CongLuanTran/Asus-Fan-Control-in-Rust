use serde::Deserialize;
use std::fs;
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
    t_enable: Option<f32>, // Temperature to turn on at full speed, default to 70
    t_auto: Option<f32>,   // Temperature to return to automatic control, default to 60
    interval: Option<Duration>, // How frequent does the program read sensor value, default to 5s
    delay: Option<Duration>, // The delay before the fan can be returned to auto, default to 30s
    b_rise: Option<f32>,   // Bias for new temperature read when it rises
    b_drop: Option<f32>,   // Bias for new temperature read when it drops
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
        let path = dirs::config_local_dir()
            .expect("Error: cannot find local config directory")
            .join("fanctl")
            .join("config.toml");
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(user_config) = toml::from_str::<FanControllerConfig>(&content) {
                if let Some(t_enable) = user_config.t_enable {
                    cfg.t_enable = Some(t_enable)
                }
                if let Some(t_auto) = user_config.t_auto {
                    cfg.t_auto = Some(t_auto)
                }
                if let Some(interval) = user_config.interval {
                    cfg.interval = Some(interval)
                }
                if let Some(delay) = user_config.delay {
                    cfg.delay = Some(delay)
                }
                if let Some(b_rise) = user_config.b_rise {
                    cfg.b_rise = Some(b_rise)
                }
                if let Some(b_drop) = user_config.b_drop {
                    cfg.b_drop = Some(b_drop)
                }
            }
        }
        cfg
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

        // Smooth the lastes value
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
