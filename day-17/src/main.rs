use intcode::{parse, run_program, ReturnStatus, State};
use std::collections::HashMap;
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

    let initial_img = read_cam_image(program.clone())?;
    let scaffold_intersections = find_scaff_intersections(&initial_img);
    let alignment_checksum = get_alignment_sum(&scaffold_intersections);
    println!(
        "The sum of alignment parameters of scaffold intersections is {}",
        alignment_checksum
    );

    println!("Initial map:");
    print_map(&initial_img);

    println!("Running robot!");
    let dust = program_and_run_robot(program)?;
    println!("The robot collected {} specks of dust.", dust);

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn print_map(map: &HashMap<Vec2, char>) {
    // now it bites me in the ass that I decided to use a map for the image.
    // at least I know it starts at (0,0)
    let x_size: usize = map
        .keys()
        .filter_map(|(x, _)| usize::try_from(*x).ok())
        .max()
        .map(|x| x + 1)
        .unwrap_or(0);
    let y_size: usize = map
        .keys()
        .filter_map(|(_, y)| usize::try_from(*y).ok())
        .max()
        .map(|x| x + 2)
        .unwrap_or(0);

    for y in 0..y_size {
        for x in 0..x_size {
            print!("{}", map.get(&(x, y)).cloned().unwrap_or('.'));
        }
        println!();
    }
}

type Vec2 = (usize, usize);

// I'm using a map again even though it would probably better to use a vec.
// However, I'm to lazy to handle special cases where the line has different lengths
fn read_cam_image(program: Vec<isize>) -> Result<HashMap<Vec2, char>, String> {
    let (_, _, output) = run_program(State::new(program), &[])?;
    let mut row: usize = 0;
    let mut col: usize = 0;
    let mut map: HashMap<Vec2, char> = HashMap::with_capacity(output.len());

    for cell in output {
        let cell_char: char = u32::try_from(cell)
            .map_err(|e| e.to_string())
            .and_then(|u| char::try_from(u).map_err(|e| e.to_string()))?;
        if cell_char == '\n' {
            row += 1;
            col = 0;
        } else {
            map.insert((col, row), cell_char);
            col += 1;
        }
    }
    Ok(map)
}

fn find_scaff_intersections(map: &HashMap<Vec2, char>) -> Vec<Vec2> {
    map.iter()
        .filter(|((px, py), c)| {
            **c != '.'
                && *px > 0
                && *py > 0
                && !is_open(map.get(&(*px, py - 1)))
                && !is_open(map.get(&(*px, py + 1)))
                && !is_open(map.get(&(px - 1, *py)))
                && !is_open(map.get(&(px + 1, *py)))
        })
        .map(|(pos, _)| *pos)
        .collect()
}

fn is_open(cell: Option<&char>) -> bool {
    match cell {
        Some('.') => true,
        None => true,
        _ => false,
    }
}

fn get_alignment_sum(points: &[Vec2]) -> usize {
    points.iter().map(|(px, py)| px * py).sum()
}

fn program_and_run_robot(mut program: Vec<isize>) -> Result<isize, String> {
    if program.is_empty() {
        return Err("No program for cleaning robot".to_owned());
    }
    program[0] = 2;

    // I still have no idea for an algorithm to figure out that movement program…
    let move_logic: Vec<isize> =
        "A,B,A,B,A,C,B,C,A,C\nL,6,R,12,L,6\nR,12,L,10,L,4,L,6\nL,10,L,10,L,4,L,6\nn\n"
            .chars()
            .filter_map(|c| u32::try_from(c).ok().and_then(|u| isize::try_from(u).ok()))
            .collect();

    let (_, status, output) = run_program(State::new(program), &move_logic)?;
    if status != ReturnStatus::Halt {
        return Err("Robot did not exit with return status HALT".to_owned());
    }

    output
        .last()
        .cloned()
        .ok_or_else(|| "No output after running robot".to_owned())
}
