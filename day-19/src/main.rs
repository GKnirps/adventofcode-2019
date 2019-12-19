use intcode::{parse, run_program, ReturnStatus, State};
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

    let affected_in_50x50 = count_affected_points(&program)?;
    println!(
        "In the 50x50 area in front of the beam, {} points are affected",
        affected_in_50x50
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

fn check_pos(program: Vec<isize>, x: isize, y: isize) -> Result<bool, String> {
    let (_, status, output) = run_program(State::new(program), &[x, y])?;
    if status != ReturnStatus::Halt {
        return Err("Program did not halt, it is waiting for more input".to_owned());
    }
    output
        .first()
        .map(|i| *i != 0)
        .ok_or_else(|| "No output".to_owned())
}

fn count_affected_points(program: &[isize]) -> Result<u32, String> {
    let mut count: u32 = 0;
    for y in 0..50 {
        for x in 0..50 {
            if check_pos(program.to_vec(), x, y)? {
                count += 1;
            }
        }
    }
    Ok(count)
}
