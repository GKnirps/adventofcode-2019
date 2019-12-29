use intcode::{parse, run_program, ReturnStatus, State};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{stdin, BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let program = parse(&content)?;

    run_interactive(program)?;

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn run_interactive(program: Vec<isize>) -> Result<(), String> {
    let mut process = State::new(program);
    let mut input: Vec<isize> = vec![];

    loop {
        let (state, status, output) = run_program(process, &input)?;
        process = state;
        print_output(&output)?;
        if status == ReturnStatus::Halt {
            break;
        }
        input = read_input()?;
    }

    Ok(())
}

fn print_output(output: &[isize]) -> Result<(), String> {
    let ostring = output
        .iter()
        .map(|i| {
            u32::try_from(*i)
                .map_err(|e| e.to_string())
                .and_then(|u| char::try_from(u).map_err(|e| e.to_string()))
        })
        .collect::<Result<String, String>>()?;
    print!("{}", ostring);
    Ok(())
}

fn read_input() -> Result<Vec<isize>, String> {
    let mut buf = String::with_capacity(128);
    stdin().read_line(&mut buf).map_err(|e| e.to_string())?;
    let input = buf.chars().map(|c| c as isize).collect();
    Ok(input)
}
