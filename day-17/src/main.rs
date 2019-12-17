use intcode::{parse, run_program, State};
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

    let initial_img = read_cam_image(program)?;
    let scaffold_intersections = find_scaff_intersections(&initial_img);
    let alignment_checksum = get_alignment_sum(&scaffold_intersections);
    println!(
        "The sum of alignment parameters of scaffold intersections is {}",
        alignment_checksum
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

#[cfg(test)]
mod test {
    use super::*;
}
