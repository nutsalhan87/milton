use std::{
    env,
    error::Error,
    fs::{self, read_dir, remove_file, File},
    io::Write,
    process::Command,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq)]
struct Golden {
    source: String,
    input: String,
    compiled: String,
    stdout: String,
    stderr: String,
}

const SOURCES: &str = "tests/golden_sources.nl";
const BINARY: &str = "tests/golden_binary";

#[test]
fn golden() -> Result<(), Box<dyn Error>> {
    let mut result: Result<(), Box<dyn Error>> = Ok(());
    let update = env::var("UPDATE").is_ok();
    for entry in read_dir("tests/golden")? {
        let path = entry?.path();
        let file_str = fs::read_to_string(&path)?;
        let g: Golden = serde_yaml::from_str(&file_str)?;

        let mut temp_sources = File::create(SOURCES)?;
        temp_sources.write_all(g.source.as_bytes())?;
        let compiler_output = Command::new("target/debug/nlisp")
            .args([SOURCES, BINARY])
            .output()?;
        let compiled = {
            let s = String::from_utf8(compiler_output.stderr)?;
            let lines: Vec<String> = s.lines().map(|v| v.to_string()).collect();
            let last_lines: Vec<String> = lines.into_iter().rev().take(100).rev().collect();
            last_lines.join("\n")
        };

        let machine_output = Command::new("target/debug/milton")
            .arg(BINARY)
            .args(g.input.split_whitespace())
            .output()?;
        let machine_stdout = String::from_utf8(machine_output.stdout)?;
        let machine_stderr = {
            let s = String::from_utf8(machine_output.stderr)?;
            let lines: Vec<String> = s.lines().map(|v| v.to_string()).collect();
            let last_lines: Vec<String> = lines.into_iter().rev().take(100).rev().collect();
            last_lines.join("\n")
        };

        let new_g = Golden {
            source: g.source.clone(),
            input: g.input.clone(),
            compiled,
            stdout: machine_stdout,
            stderr: machine_stderr,
        };
        if update {
            println!(
                "{} have written",
                path.file_name().unwrap().to_str().unwrap()
            );
            fs::write(&path, serde_yaml::to_string(&new_g)?)?;
        } else if g != new_g {
            result = Err(format!(
                "Incongruity in test {}",
                path.file_name().unwrap().to_str().unwrap()
            )
            .into());
            break;
        }
    }

    remove_file(BINARY)?;
    remove_file(SOURCES)?;

    result
}
