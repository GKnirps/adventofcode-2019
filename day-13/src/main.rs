use intcode::{parse, run_program, State};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let program = parse(&content)?;

    let (_, _, initial_output) = run_program(State::new(program), &[])?;
    let initial_paint = translate_paint_output(&initial_output);
    println!(
        "Initially, there are {} block tiles.",
        initial_paint.values().filter(|t| **t == 2).count()
    );

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn translate_paint_output(output: &[isize]) -> HashMap<(isize, isize), isize> {
    output
        .chunks_exact(3)
        .map(|chunk| ((chunk[0], chunk[1]), chunk[2]))
        .collect()
}
