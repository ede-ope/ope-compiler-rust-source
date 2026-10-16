use std::fs;
use std::process::Command;

// regex::CaptureNames;

pub fn run_file(filename: &str) {
    println!("Running file: {}", filename);
    let source = fs::read_to_string(filename).expect("Failed to read source file");

    for line in source.lines() {
        let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
        if parts.is_empty() || parts[0].is_empty() {
            continue;
        }

        match parts[0] {
            "PRINT" => {
                if parts.len() > 1 {
                    println!("{}", parts[1]);
                }
            }
            "RUN" => {
                if parts.len() > 1 {
                    let cmd_parts: Vec<&str> = parts[1].split_whitespace().collect();
                    if !cmd_parts.is_empty() {
                        let mut cmd = Command::new(cmd_parts[0]);
                        if cmd_parts.len() > 1 {
                            cmd.args(&cmd_parts[1..]);
                        }
                        match cmd.status() {
                            Ok(status) => println!("Command exited with: {}", status),
                            Err(err) => eprintln!("Failed to run command: {}", err),
                        }
                    }
                }
            }
            _ => { /* Already validated in compile */ }
        }
    }
}
 