use std::{fs, thread::sleep, time::{Duration, Instant}};

use sysinfo::Components;
use walkdir::WalkDir;

fn main() {
    let mut components = Components::new_with_refreshed_list();
    let cpu = components
        .list_mut()
        .into_iter()
        .find(|component| component.label().contains("Package"))
        .expect("Error: Cannot find CPU sensors");

    let pwm = WalkDir::new("/sys/devices/platform/asus-nb-wmi/hwmon")
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|file| file.path().to_string_lossy().contains("pwm1_enable"))
        .expect("Error: Cannot find pwm1_enable file");

    let interval = Duration::from_secs(1);
    let mut next_time = Instant::now() + interval;
    let mut record = [0.0; 10];
    let mut sum = 0.0;
    let mut index = 0;
    loop {
        cpu.refresh();
        sum -= record[index];
        record[index] = cpu.temperature();
        sum += record[index];
        index = (index + 1) % 10;
        let value = if sum < 500.0 {2} else {0};
        fs::write(pwm.path(), value.to_string()).unwrap();
        sleep(next_time - Instant::now());
        next_time += interval;
    }
}
