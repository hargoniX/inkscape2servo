use quicli::prelude::*;
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::Write;

fn process_line(line: &str, state: u8, outfile: Option<&mut File>) -> u8 {
    let mut new_state = state;
    let mut new_line = line;

    if line.contains("M3") || line.contains("M5") {
        return state;
    }

    if line.contains('Z') {
        let z_value_regex = Regex::new(r".*Z(?P<z_value>\-?\d+\.\d+).*")
            .expect("For some reason a perfectly valid regex did not work.");
        let z_value = z_value_regex
            .captures(line)
            .expect("A line with a Z in it did not contain a number after the Z");
        let z_value: f32 = z_value["z_value"].parse().unwrap();

        if z_value < 0.0 && state != 1 {
            new_line = "M3 (pen down)\nG4 P1 (wait a ms so the PWM changes can take action)";
            new_state = 1;
        } else if z_value > 0.0 && state != 0 {
            new_line = "M5 (pen up)\nG4 P1 (wait a ms so the PWM changes can take action)";
            new_state = 0;
        }
    }

    match outfile {
        None => println!("{}", new_line),
        Some(x) => writeln!(x, "{}", new_line).expect("Couldnt write to outfile"),
    }
    new_state
}

pub fn process(infile: String, outfile: Option<String>) {
    let content = read_file(infile).expect("Input file not found");
    let content_lines = content.lines();
    let mut state = 0;
    match outfile {
        None => {
            println!("M5 (pen up)\nG4 P1 (wait a ms so the PWM changes can take action)");
            for line in content_lines {
                state = process_line(line, state, None);
            }
        }
        Some(x) => {
            let mut outfile = OpenOptions::new()
                .write(true)
                .append(false)
                .create(true)
                .open(x)
                .expect("Couldnt open outfile");
            writeln!(
                &mut outfile,
                "M5 (pen up)\nG4 P1 (wait a ms so the PWM changes can take action)"
            )
            .expect("Couldnt write to outfile");

            for line in content_lines {
                state = process_line(line, state, Some(&mut outfile));
            }
        }
    }
}
