use intcode::{parse, run_program, ReturnStatus, State};
use std::convert::TryFrom;
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

    let walkscript = "NOT A J\n\
                      NOT B T\n\
                      OR T J\n\
                      NOT C T\n\
                      OR T J\n\
                      AND D J\n\
                      WALK\n";
    let hull_damage = run_springbot(program.clone(), walkscript)?;
    println!("Hull damage: {}", hull_damage);

    let runscript = "NOT A J\n\
                     NOT B T\n\
                     OR T J\n\
                     NOT C T\n\
                     OR T J\n\
                     AND D J\n\
                     NOT E T\n\
                     NOT T T\n\
                     OR H T\n\
                     AND T J\n\
                     RUN\n";
    let more_hull_damage = run_springbot(program, runscript)?;
    println!(
        "Hull damage detected with increased sensor range: {}",
        more_hull_damage
    );

    Ok(())
}

fn run_springbot(program: Vec<isize>, springscript: &str) -> Result<isize, String> {
    let input: Vec<isize> = springscript
        .chars()
        .filter_map(|c| u32::try_from(c).ok().and_then(|u| isize::try_from(u).ok()))
        .collect();

    let (_, status, output) = run_program(State::new(program), &input)?;
    if status != ReturnStatus::Halt {
        return Err("Program did not halt correctly".to_owned());
    }

    let ostring: String = output
        .iter()
        .filter_map(|i| u32::try_from(*i).ok().and_then(|u| char::try_from(u).ok()))
        .collect();
    println!("{}", ostring);

    output.last().copied().ok_or_else(|| "No output".to_owned())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}
