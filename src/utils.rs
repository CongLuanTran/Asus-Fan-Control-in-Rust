use std::io::Write;
use std::{ffi::OsStr, fs::OpenOptions, path::PathBuf};
use sysinfo::Component;
use sysinfo::Components;
use walkdir::WalkDir;

pub fn find_cpu(components: &mut Components) -> &mut Component {
    // We find the component that has label 'Package id 0'
    // It is one of the indicator of CPU temperature
    // You can explore more by running `sensors` in your terminal
    components
        .list_mut()
        .iter_mut()
        .find(|component| component.label().contains("Package"))
        .expect("Error: Cannot find CPU sensors")
}

pub fn find_pwn1() -> PathBuf {
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

pub fn write_pwn1(path: &PathBuf, value: u16) {
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
