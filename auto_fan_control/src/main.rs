use rust_gpiozero::*;
use std::thread::sleep;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::Command;

const TEMP_THRESHOLD_LOWER: u32 = 500;
const TEMP_THRESHOLD_UPPER: u32 = 600;
const DELAY: u64 = 5;

fn main() {
    let mut fan_is_on: bool = false;

    // Sig term variable and hook
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");

    // Create new switch control at GPIO14
    let mut switch = OutputDevice::new(14);
    switch.off();

    while running.load(Ordering::SeqCst) {
        // Check temperature
        let get_temp = Command::new("vcgencmd")
                    .arg("measure_temp")
                    .output()
                    .expect("failed to execute process");
        let result = String::from_utf8(get_temp.stdout).unwrap();
        let filtered_result: String = result.chars().filter(|c| c.is_digit(10)).collect();  // Filter out non number char
        println!("{}", filtered_result);
        let temp: u32 = filtered_result.parse::<u32>().unwrap();

        // Compare temperature
        // Add hysteresis
        if temp > TEMP_THRESHOLD_UPPER {
            // If above threshold, set pin
            if fan_is_on == false {
                println!("Turning ON fan");
                switch.on();
                fan_is_on = true;
            }
        }
        else if temp < TEMP_THRESHOLD_LOWER{
            // If below threshold, clear pin
            if fan_is_on == true {
                println!("Turning OFF fan");
                switch.off();
                fan_is_on = false;
            }
        }

        // Sleep for 2 seconds
        sleep(Duration::from_secs(DELAY));
    }

    // Turning off before ends the program
    switch.off();
    println!("Turning off fan and exiting...");
}
