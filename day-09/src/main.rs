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
    let prog = parse(content.trim())?;

    println!("Output for test mode: {}", run_test_mode(prog)?);

    Ok(())
}

fn run_test_mode(prog: Vec<isize>) -> Result<isize, String> {
    let (_, _, output) = run_program(State::new(prog), &[1])?;
    output
        .first()
        .cloned()
        .ok_or_else(|| "No output for test mode".to_owned())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}
