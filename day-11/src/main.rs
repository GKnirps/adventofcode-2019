use intcode::{parse, run_program, ReturnStatus, State};
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
    // intcode again!
    let program = parse(&content)?;

    let result = run_robot(program)?;

    println!("The robot painted {} panels", result.len());

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

type Vec2 = (i32, i32);

fn turn(signal: isize, (x, y): Vec2) -> Vec2 {
    if signal == 0 {
        (-y, x)
    } else {
        (y, -x)
    }
}

// this is kind of like Langton's ant
fn run_robot(program: Vec<isize>) -> Result<HashMap<Vec2, isize>, String> {
    let mut process = State::new(program);
    let mut robot_pos: Vec2 = (0, 0);
    let mut robot_orient: Vec2 = (0, 1);
    let mut painted_panels: HashMap<Vec2, isize> = HashMap::with_capacity(128);

    loop {
        let input = painted_panels.get(&robot_pos).cloned().unwrap_or(0);
        let (state, status, output) = run_program(process, &[input])?;
        process = state;
        if status == ReturnStatus::Halt {
            return Ok(painted_panels);
        }
        *painted_panels.entry(robot_pos).or_insert(0) = *output
            .get(0)
            .ok_or_else(|| "No paint command!".to_owned())?;
        let turn_signal = *output.get(1).ok_or_else(|| "No turn signal!".to_owned())?;
        robot_orient = turn(turn_signal, robot_orient);
        robot_pos = (robot_pos.0 + robot_orient.0, robot_pos.1 + robot_orient.1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
