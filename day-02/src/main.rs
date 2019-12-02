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

    match puzzle1(initial_state.clone()) {
        Ok(result) => println!("Puzzle 1 result: {}", result),
        Err(msg) => println!("Puzzle 1 error: {}", msg),
    }

    match puzzle2(initial_state) {
        Ok(result) => println!("Puzzle 2 result: {}", result),
        Err(msg) => println!("Puzzle 2 error: {}", msg),
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

fn parse(input: &str) -> Result<Vec<usize>, String> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>().map_err(|e| e.to_string()))
        .collect()
}

fn run_program(mem: Vec<usize>) -> Result<usize, String> {
    let mut mem = mem;
    if mem.is_empty() {
        return Err("no program".to_owned());
    }
    let mut pc: usize = 0;
    while mem[pc] != 99 {
        match mem[pc] {
            1 => {
                let (v1, v2, dest) = get_operands(pc, &mem)?;
                mem[dest] = v1 + v2;
            }
            2 => {
                let (v1, v2, dest) = get_operands(pc, &mem)?;
                mem[dest] = v1 * v2;
            }
            _ => {
                return Err(format!("Unknown opcode {}", mem[pc]));
            }
        };
        pc += 4;
        if pc >= mem.len() {
            return Err("Program did not halt".to_owned());
        }
    }
    Ok(mem[0])
}

fn get_operands(pc: usize, mem: &[usize]) -> Result<(usize, usize, usize), String> {
    if pc + 3 >= mem.len() {
        return Err(format!(
            "Not enough operands for pc {} and mem.len() {}",
            pc,
            mem.len()
        ));
    }
    let v1 = mem.get(mem[pc + 1]).ok_or_else(|| {
        format!(
            "Index 1 ({}) is out of bounds (mem.len(): {})",
            mem[pc + 1],
            mem.len()
        )
    })?;
    let v2 = mem.get(mem[pc + 2]).ok_or_else(|| {
        format!(
            "Index 2 ({}) is out of bounds (mem.len(): {})",
            mem[pc + 2],
            mem.len()
        )
    })?;
    let dest = mem[pc + 3];
    if dest >= mem.len() {
        return Err(format!(
            "Destination {} is out of bounds (mem.len(): {})",
            dest,
            mem.len()
        ));
    }
    Ok((*v1, *v2, dest))
}

fn run_program_with_input(mem: Vec<usize>, noun: usize, verb: usize) -> Result<usize, String> {
    let mut mem = mem;
    if mem.len() < 3 {
        return Err("program too short to modify".to_owned());
    }

    mem[1] = noun;
    mem[2] = verb;

    run_program(mem)
}

fn puzzle1(mem: Vec<usize>) -> Result<usize, String> {
    run_program_with_input(mem, 12, 2)
}

fn puzzle2(mem: Vec<usize>) -> Result<usize, String> {
    // screw this, let's brute force this
    for noun in 0..100 {
        for verb in 0..100 {
            if let Ok(19_690_720) = run_program_with_input(mem.clone(), noun, verb) {
                return Ok(100 * noun + verb);
            }
        }
    }
    Err("No result in search space".to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        assert_eq!(
            run_program(vec!(1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50)),
            Ok(3500)
        );
        assert_eq!(run_program(vec!(1, 0, 0, 0, 99)), Ok(2));
    }
}
