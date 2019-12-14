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
    let mut program = parse(&content)?;

    let (_, _, initial_output) = run_program(State::new(program.clone()), &[])?;
    let initial_paint = translate_paint_output(&initial_output);
    println!(
        "Initially, there are {} block tiles.",
        initial_paint.values().filter(|t| **t == 2).count()
    );

    program[0] = 2;
    let end_state = run_game(program)?;
    evaluate_result(&end_state);

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

fn run_game(program: Vec<isize>) -> Result<HashMap<(isize, isize), isize>, String> {
    let mut display: HashMap<(isize, isize), isize> = HashMap::with_capacity(1024);
    let mut process = State::new(program);
    let mut input: isize = 0;

    loop {
        let (state, status, output) = run_program(process, &[input])?;
        for (x, y, value) in output
            .chunks_exact(3)
            .map(|chunk| (chunk[0], chunk[1], chunk[2]))
        {
            display.insert((x, y), value);
        }
        if status == ReturnStatus::Halt {
            return Ok(display);
        }
        process = state;

        let ((paddle_x, _), (ball_x, _)) = find_paddle_and_ball(&display)
            .ok_or_else(|| "Unable to find paddle and ball on display!".to_owned())?;
        input = (ball_x - paddle_x).signum();
    }
}

fn find_paddle_and_ball(
    display: &HashMap<(isize, isize), isize>,
) -> Option<((isize, isize), (isize, isize))> {
    let paddle = display
        .iter()
        .filter(|((x, y), value)| !(*x == -1 && *y == 0) && **value == 3)
        .map(|(pos, _)| *pos)
        .next()?;
    let ball = display
        .iter()
        .filter(|((x, y), value)| !(*x == -1 && *y == 0) && **value == 4)
        .map(|(pos, _)| *pos)
        .next()?;

    Some((paddle, ball))
}

fn evaluate_result(display: &HashMap<(isize, isize), isize>) {
    let leftover_blocks = display
        .iter()
        .filter(|((x, y), value)| !(*x == -1 && *y == 0) && **value == 2)
        .count();
    if leftover_blocks > 0 {
        println!(
            "You lost the game! There are still {} blocks left!",
            leftover_blocks
        );
    }
    if let Some(points) = display.get(&(-1, 0)) {
        println!("Result: {} points", points);
    }
}
