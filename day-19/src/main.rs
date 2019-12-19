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

    let (x, y) = find_first_fitting_square(&program)?;
    println!(
        "First fitting square starts at {}×{}, result value is {}",
        x,
        y,
        x * 10000 + y
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

fn find_first_fitting_square(program: &[isize]) -> Result<(isize, isize), String> {
    let mut start_y = 0;
    let mut start_x = 0;

    loop {
        let (lower_x, lower_y) = find_first_affected(program, start_x, start_y)?;
        let mut x_diff = 0;
        while check_pos(program.to_vec(), lower_x + x_diff + 99, lower_y)? {
            if check_pos(program.to_vec(), lower_x + x_diff, lower_y + 99)? {
                return Ok((lower_x + x_diff, lower_y));
            }
            x_diff += 1;
        }
        start_y = lower_y + 1;
        start_x = lower_x;
    }
}

fn find_first_affected(
    program: &[isize],
    offset_x: isize,
    offset_y: isize,
) -> Result<(isize, isize), String> {
    // This is a heuristic, we assume if there is no affected point in a 100x100 square from the offset,
    // there is no point in looking any further. If our beam is not extremely unusual and if we are
    // careful with the inputs, this should work though.
    for y in 0..100 {
        for x in 0..100 {
            if check_pos(program.to_vec(), x + offset_x, y + offset_y)? {
                return Ok((x + offset_x, y + offset_y));
            }
        }
    }
    Err("Unable to find any affected point within the search radius".to_owned())
}
