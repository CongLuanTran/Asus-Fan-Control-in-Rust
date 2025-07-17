use clap::{Parser, Subcommand};
use std::io::Write;
use std::thread;
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    path::PathBuf,
    time::{Duration, Instant},
};
use strum_macros::Display;
use sysinfo::{Component, Components};
use walkdir::WalkDir;

fn find_cpu(components: &mut Components) -> &mut Component {
    // We find the component that has label 'Package id 0'
    // It is one of the indicator of CPU temperature
    // You can explore more by running `sensors` in your terminal
    components
        .list_mut()
        .iter_mut()
        .find(|component| component.label().contains("Package"))
        .expect("Error: Cannot find CPU sensors")
}

fn find_pwn1() -> PathBuf {
    // Try to find the file that can control fan mode in the directory of 'asus-nb-wmi'
    // The name of the file is 'pwm1_enable'
    // Writing a '0' to this file will turn the fan on full speed. A '2' will return it to auto mode
    WalkDir::new("/sys/devices/platform/asus-nb-wmi/hwmon")
        .into_iter()
        .filter_map(Result::ok)
        .find(|file| file.path().file_name() == Some(OsStr::new("pwm1_enable")))
        .expect("Error: Cannot find pwm1_enable file")
        .path()
        .to_path_buf()
}

fn write_pwn1(path: &PathBuf, value: u16) {
    match OpenOptions::new().write(true).open(path) {
        Ok(mut file) => {
            if let Err(e) = write!(file, "{}", value) {
                eprintln!("Failed to write to {:?}: {}", path, e);
            };
        }
        Err(e) => {
            eprintln!("Error opening {:?}: {}", path, e);
        }
    }
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
enum FanState {
    #[strum(serialize = "enabled")]
    Enabled,
    #[strum(serialize = "auto")]
    Auto,
}

/*
Use hysteresis, response delay and disproportional smoothing to make the system react fast to heat
spikes, but slow to drops.
*/
struct FanController {
    t_enable: f32,       // Temperature to turn on at full speed, default to 70
    t_auto: f32,         // Temperature to return to automatic control, default to 60
    interval: Duration,  // How frequent does the program read sensor value, default to 5s
    delay: Duration,     // The delay before the fan can be returned to auto, default to 30s
    b_rise: f32,         // Bias for new temperature read when it rises
    b_drop: f32,         // Bias for new temperature read when it drops
    fan_state: FanState, // Current state of the fan
    smoothed_temp: f32,  // The smoothed temperature
    latest_temp: f32,    // The latest read temperature
    next_read: Instant,  // Next sensor read time
}

impl FanController {
    fn new(
        t_enable: f32,
        t_auto: f32,
        interval: u64,
        delay: u64,
        b_rise: f32,
        b_drop: f32,
    ) -> Self {
        Self {
            t_enable,
            t_auto,
            interval: Duration::from_secs(interval),
            delay: Duration::from_secs(delay),
            b_rise,
            b_drop,
            fan_state: FanState::Auto,
            smoothed_temp: 0.0,
            latest_temp: 0.0,
            next_read: Instant::now(),
        }
    }

    fn update(&mut self, temp: f32) {
        // Switch bias depending on whether the temperature is rising or falling
        let bias = if temp > self.smoothed_temp {
            self.b_rise
        } else {
            self.b_drop
        };

        // Smooth the lastes value
        self.smoothed_temp = bias * temp + (1.0 - bias) * self.smoothed_temp;
        self.latest_temp = temp; // This is mostly for display

        // Hysteresis switch fan state
        if self.smoothed_temp >= self.t_enable {
            self.fan_state = FanState::Enabled;
            // Use the longer delay when turning the fan on
            self.next_read += self.delay;
        } else {
            // Else use the shorter interval
            self.next_read += self.interval;
            if self.smoothed_temp <= self.t_auto {
                self.fan_state = FanState::Auto;
            }
        }
    }
}

impl Default for FanController {
    fn default() -> Self {
        Self::new(70.0, 60.0, 5, 30, 0.8, 0.2)
    }
}

fn start_daemon() {
    // First we find all components of our system
    let mut components = Components::new_with_refreshed_list();
    // Get the component that represent cpu temperature
    let cpu = find_cpu(&mut components);

    // Find the pwn1_enable file and get its path
    let pwn1_enable = find_pwn1();

    // Initialize controller
    let mut controller = FanController::default();

    // Read the temp for the first time
    let temp = cpu.temperature().unwrap();
    controller.update(temp);
    loop {
        // Read latest info
        cpu.refresh();
        let temp = cpu.temperature().unwrap();

        // Update controller state
        controller.update(temp);

        // We turn the fan on full speed (0) if the temperature reach the threshold, else return it to auto ()
        let value = match controller.fan_state {
            FanState::Enabled => 0,
            FanState::Auto => 2,
        };

        // Write the value into the file
        write_pwn1(&pwn1_enable, value);

        // // Write to stdout, which systemd will capture
        // print!(
        //     "Fan Status: {}, Latest Temperature: {}°C",
        //     controller.fan_state, controller.latest_temp
        // );
        // Wait until the next interval
        thread::sleep(controller.next_read - Instant::now());
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Status,
    Daemon,
}

fn show_status() {
    let out = std::process::Command::new("systemctl")
        .args(["status", "fanctl.service"])
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&out.stdout));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Daemon => start_daemon(),
        Command::Status => show_status(),
    }
}
