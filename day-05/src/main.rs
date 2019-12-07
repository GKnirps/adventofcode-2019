use intcode::{parse, run_program};
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let initial_state = parse(&content)?;

    println!("Start process");
    match run_program(initial_state, BufReader::new(io::stdin()), io::stdout()) {
        Ok(_) => println!("Process halted"),
        Err(e) => println!("Process failed: {}", e),
    }

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;
    Ok(result)
}
