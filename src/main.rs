use std::{
    fs,
    thread::sleep,
    time::{Duration, Instant},
};
// Sysinfo is a crate for querying system info. It is used to read CPU temperature
// The newest version of sysinfo (0.33) doesn't work for me, so this code use version 0.32.1
use sysinfo::Components;
// WalkDir is used to find all files in the directory path
use walkdir::WalkDir;

fn main() {
    // First we find all components of our system
    let mut components = Components::new_with_refreshed_list();
    // Then we find the component that has label 'Package id 0'
    // It is one of the indicator of CPU temperature
    // You can explore more by running `sensors` in your terminal
    let cpu = components
        .list_mut()
        .into_iter()
        .find(|component| component.label().contains("Package"))
        .expect("Error: Cannot find CPU sensors");

    // Try to find the file that can control fan mode in the directory of 'asus-nb-wmi'
    // The name of the file is 'pwm1_enable'
    // Writing a '0' to this file will turn the fan on full speed. A '2' will return it to auto mode
    let pwm = WalkDir::new("/sys/devices/platform/asus-nb-wmi/hwmon")
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|file| file.path().to_string_lossy().contains("pwm1_enable"))
        .expect("Error: Cannot find pwm1_enable file");

    // Set the loop interval, here I will read the temperature every second
    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;
    // Just some variable to hold the value
    let mut record = [0.0; 10];
    let mut sum = 0.0;
    let mut index = 0;
    loop {
        // We first use refresh to read the latest info
        cpu.refresh();
        // Then we remove the oldest value
        sum -= record[index];
        // Then add the new value
        record[index] = cpu.temperature();
        sum += record[index];
        // And we move the index forward
        index = (index + 1) % 10;
        // If the sum is higher than 500 (i.e. the average is higher than 50)
        // We turn the fan on full speed, else return it to auto
        let value = if sum < 500.0 { 2 } else { 0 };
        // Write the value into the file
        fs::write(pwm.path(), value.to_string()).unwrap();
        // Wait until the next interval
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
