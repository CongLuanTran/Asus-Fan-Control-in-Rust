use std::io::Write;
use std::{
    ffi::OsStr,
    fs::OpenOptions,
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};
// Sysinfo is a crate for querying system info. It is used to read CPU temperature
// The newest version of sysinfo (0.33) doesn't work for me, so this code use version 0.32.1
use sysinfo::{Component, Components};
// WalkDir is used to find all files in the directory path
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
    let mut file = OpenOptions::new().write(true).open(path).unwrap();
    write!(file, "{}", value).unwrap();
}

fn main() {
    // First we find all components of our system
    let mut components = Components::new_with_refreshed_list();
    // Get the component that represent cpu temperature
    let cpu = find_cpu(&mut components);

    // Find the pwn1_enable file and get its path
    let pwn1_enable = find_pwn1();

    // Set the loop interval, here I will read the temperature every 5 second
    let interval = Duration::from_secs(5);
    let mut next_time = Instant::now() + interval;
    // Just some variable to hold the value
    let mut temp = cpu.temperature().unwrap() as u16;
    loop {
        // We first use refresh to read the latest info
        cpu.refresh();
        // Smoothing the temperature by taking a mean of its current and previous reading
        temp = (temp + cpu.temperature().unwrap() as u16) / 2;
        // We turn the fan on full speed, else return it to auto
        let value = if temp < 50 { 2 } else { 0 };
        // Write the value into the file
        write_pwn1(&pwn1_enable, value);
        // Wait until the next interval
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
