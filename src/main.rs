use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

const BRIGHTNESS_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";
const MAX_BRIGHTNESS_PATH: &str = "/sys/class/backlight/intel_backlight/max_brightness";

fn get_max_brightness() -> f32 {
    let max_brightness_str = fs::read_to_string(MAX_BRIGHTNESS_PATH).expect("file error");
    return max_brightness_str.trim().parse::<f32>().unwrap();
}

fn get_brightness() -> f32 {
    let brightness_str = fs::read_to_string(BRIGHTNESS_PATH).expect("file error");
    return brightness_str.trim().parse::<f32>().unwrap();
}

fn get_monitor_string() -> String {
    let xrandr_out = Command::new("xrandr")
        .output()
        .expect("failed to spawn xrandr")
        .stdout;
    //let xrandr_out_s = String::from_utf8_lossy(&xrandr_out);
    //println!("xrandr output {}", xrandr_out_s);

    let grep_ps = Command::new("grep")
        .arg("-w")
        .arg("connected")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn grep");
    grep_ps
        .stdin
        .unwrap()
        .write_all(&xrandr_out)
        .expect("unable to write to grep stdin");

    let mut grep_out_vec = Vec::new();
    grep_ps
        .stdout
        .unwrap()
        .read_to_end(&mut grep_out_vec)
        .unwrap();

    let awk_ps = Command::new("awk")
        .arg("{print $1}")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("unable to spawn awk");
    awk_ps
        .stdin
        .unwrap()
        .write_all(&grep_out_vec)
        .expect("unable to writeto awk stdin");
    let mut awk_out = String::new();
    awk_ps.stdout.unwrap().read_to_string(&mut awk_out).unwrap();
    println!("your screen {}", awk_out);
    return awk_out;
}

#[derive(Debug)]
pub enum InputError {
    OutOfBounds,
    NotAnInteger,
    TooManyArgs,
}

fn get_delta(t: &str) -> Result<i32, InputError> {
    let result: Result<f32, _> = t.parse::<f32>();
    if let Ok(i) = result {
        if i > 100.0 || i <= 0.0 {
            return Err(InputError::OutOfBounds);
        } else {
            let rat = i / 100.0;
            let bright_d = rat * get_max_brightness();
            println!("new ratio {} brightness delta {}", rat, bright_d);
            return Ok(bright_d.round() as i32);
        }
    } else {
        return Err(InputError::NotAnInteger);
    }
}

fn add_brightness(t: i32) {
    println!("add fn enter {} ", t);
}

fn sub_brightness(t: i32) {
    println!("subtract {}", t);
}

fn set_brightness(t: i32) {
    println!("set {}", t);
    get_monitor_string();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];
    let current_ratio = get_brightness() / get_max_brightness();
    println!(
        "current brightness {} max brightness {} ratio {}",
        get_brightness(),
        get_max_brightness(),
        current_ratio
    );

    if query == "max" {
        println!("setting brightness to max");
    } else {
        let first_char = &query[0..1];
        let the_rest = &query[1..query.len()];
        // if first_char == "p" || first_char == "m" {
        let bright_d = if first_char == "p" || first_char == "m" {
            get_delta(the_rest)
        } else {
            get_delta(query)
        };
        match bright_d {
            Ok(d) => match first_char {
                "p" => add_brightness(d),
                "m" => sub_brightness(d),
                _ => set_brightness(d),
            },

            Err(e) => println!("Error {:?}", e),
        }
    }
}
