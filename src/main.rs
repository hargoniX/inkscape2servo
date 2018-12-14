use quicli::prelude::*;
use regex::Regex;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

/*
note to me
M5: pen up
M3: pen down
*/

fn process_line(line: &str, state: u8) -> u8 {
    let mut new_state = state;
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
            println!("M3 (pen down)\nG4 P1 (wait a ms so the PWM changes can take action)");
            new_state = 1;
        } else if z_value > 0.0 && state != 0 {
            println!("M5 (pen up)\nG4 P1 (wait a ms so the PWM changes can take action)");
            new_state = 0;
        }
    }
    println!("{}", line);
    new_state
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("head")?;

    let content = read_file(&args.file)?;
    let content_lines = content.lines();
    let mut state = 0; // 0 = up, 1 = down

    info!("Outputting header");
    println!("M5 (pen up)\nG4 P1 (wait a ms so the PWM changes can take action)");

    for line in content_lines {
        state = process_line(line, state);
    }

    Ok(())
}
