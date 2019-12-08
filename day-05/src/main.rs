use intcode::{parse, run_program, State};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let initial_state = parse(&content)?;

    println!("Start process with input 1");
    let (_, _, output1) = run_program(State::new(initial_state.clone()), &[1])?;
    match output1.last() {
        Some(o) => println!("Last output for input 1: {}", o),
        None => println!("No output for input 1"),
    }

    println!("Start process with input 5");
    let (_, _, output5) = run_program(State::new(initial_state), &[5])?;
    match output5.last() {
        Some(o) => println!("Last output for input 5: {}", o),
        None => println!("No output for input 5"),
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
