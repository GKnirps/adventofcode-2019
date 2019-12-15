use intcode::{parse, run_program, ReturnStatus, State};
use std::cmp;
use std::collections::{HashMap, HashSet, VecDeque};
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

    let full_map = explore(program)?;
    print_map(&full_map);

    if let Some(path_length) = shortest_path_to_ox(&full_map) {
        println!("Shortest path to the oxygen tank is {} tiles.", path_length);
    } else {
        println!("Cannot find path to oxygen tank.");
    }
    let full_oxygen = longest_path_from_ox(&full_map)?;
    println!("After {} minutes, the section is safe again.", full_oxygen);

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Tile {
    Floor,
    Wall,
    OxSys,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    fn invert(self) -> Dir {
        match self {
            Dir::North => Dir::South,
            Dir::East => Dir::West,
            Dir::South => Dir::North,
            Dir::West => Dir::East,
        }
    }

    fn command(self) -> isize {
        match self {
            Dir::North => 1,
            Dir::South => 2,
            Dir::West => 3,
            Dir::East => 4,
        }
    }

    fn mv(self, (px, py): Vec2) -> Vec2 {
        match self {
            Dir::North => (px, py - 1),
            Dir::East => (px + 1, py),
            Dir::South => (px, py + 1),
            Dir::West => (px - 1, py),
        }
    }
}

type Vec2 = (i32, i32);

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn print_map(map: &HashMap<Vec2, Tile>) {
    let min_x: i32 = map.keys().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y: i32 = map.keys().map(|(_, y)| *y).min().unwrap_or(0);
    let max_x: i32 = map.keys().map(|(x, _)| *x).max().unwrap_or(0);
    let max_y: i32 = map.keys().map(|(_, y)| *y).max().unwrap_or(0);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            print!(
                "{}",
                map.get(&(x, y))
                    .map(|tile| match tile {
                        Tile::Wall => '#',
                        Tile::Floor => '.',
                        Tile::OxSys => 'O',
                    })
                    .unwrap_or(' ')
            );
        }
        println!();
    }
}

fn explore(program: Vec<isize>) -> Result<HashMap<Vec2, Tile>, String> {
    let mut pos: Vec2 = (0, 0);
    let mut map: HashMap<Vec2, Tile> = HashMap::with_capacity(2048);
    map.insert(pos, Tile::Floor);
    let mut trace: Vec<Dir> = Vec::with_capacity(1024);

    let mut process = State::new(program);

    loop {
        if let Some(dir) = get_unexplored_direction(&map, pos) {
            let (state, status, output) = run_program(process, &[dir.command()])?;
            process = state;
            if status == ReturnStatus::Halt {
                return Err("Robot stopped unexpectedly.".to_owned());
            }
            match output.first() {
                Some(0) => {
                    map.insert(dir.mv(pos), Tile::Wall);
                }
                Some(code) => {
                    pos = dir.mv(pos);
                    map.insert(pos, if *code == 1 { Tile::Floor } else { Tile::OxSys });
                    trace.push(dir);
                }
                None => return Err("No output after robot moved!".to_owned()),
            };
        } else if let Some(dir) = trace.pop() {
            let back_dir = dir.invert();
            let (state, status, _) = run_program(process, &[back_dir.command()])?;
            process = state;
            if status == ReturnStatus::Halt {
                return Err("Robot stopped unexpectedly.".to_owned());
            }
            pos = back_dir.mv(pos);
        } else {
            return Ok(map);
        }
    }
}

fn shortest_path_to_ox(map: &HashMap<Vec2, Tile>) -> Option<u32> {
    let mut visited: HashSet<Vec2> = HashSet::with_capacity(map.len());
    let mut queue: VecDeque<(Vec2, u32)> = VecDeque::with_capacity(map.len());
    queue.push_back(((0, 0), 0));

    while let Some((pos, dist)) = queue.pop_front() {
        if visited.contains(&pos) {
            continue;
        }
        match map.get(&pos) {
            Some(Tile::Wall) => {}
            None => {}
            Some(Tile::OxSys) => return Some(dist),
            Some(Tile::Floor) => {
                queue.push_back(((pos.0 - 1, pos.1), dist + 1));
                queue.push_back(((pos.0, pos.1 - 1), dist + 1));
                queue.push_back(((pos.0 + 1, pos.1), dist + 1));
                queue.push_back(((pos.0, pos.1 + 1), dist + 1));
            }
        };
        visited.insert(pos);
    }
    None
}

fn longest_path_from_ox(map: &HashMap<Vec2, Tile>) -> Result<u32, String> {
    let ox_pos: Vec2 = map
        .iter()
        .find(|(_, tile)| **tile == Tile::OxSys)
        .map(|(pos, _)| *pos)
        .ok_or_else(|| "Unable to find oxygen system".to_owned())?;

    let mut longest_path: u32 = 0;
    let mut visited: HashSet<Vec2> = HashSet::with_capacity(map.len());
    let mut queue: VecDeque<(Vec2, u32)> = VecDeque::with_capacity(map.len());
    queue.push_back((ox_pos, 0));

    while let Some((pos, dist)) = queue.pop_front() {
        if visited.contains(&pos) {
            continue;
        }
        longest_path = cmp::max(longest_path, dist);
        match map.get(&pos) {
            Some(Tile::Wall) => {}
            None => {}
            Some(_) => {
                queue.push_back(((pos.0 - 1, pos.1), dist + 1));
                queue.push_back(((pos.0, pos.1 - 1), dist + 1));
                queue.push_back(((pos.0 + 1, pos.1), dist + 1));
                queue.push_back(((pos.0, pos.1 + 1), dist + 1));
            }
        };
        visited.insert(pos);
    }
    Ok(longest_path)
}

fn get_unexplored_direction(map: &HashMap<Vec2, Tile>, pos: Vec2) -> Option<Dir> {
    [Dir::North, Dir::East, Dir::West, Dir::South]
        .iter()
        .find(|dir| !map.contains_key(&dir.mv(pos)))
        .cloned()
}
