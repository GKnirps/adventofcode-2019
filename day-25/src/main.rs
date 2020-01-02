use intcode::{parse, run_program, ReturnStatus, State};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{stdin, BufReader, BufWriter, Read, Write};
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Command {
    NativeDoor(Door),
    NativeTake(String),
    NativeDrop(String),
    NativeInv,
    Trace(String),
    Map,
}

impl Command {
    fn is_native(&self) -> bool {
        match self {
            Command::NativeDoor(_) => true,
            Command::NativeTake(_) => true,
            Command::NativeDrop(_) => true,
            Command::NativeInv => true,
            _ => false,
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Command::NativeDoor(Door::North) => write!(f, "north"),
            Command::NativeDoor(Door::East) => write!(f, "east"),
            Command::NativeDoor(Door::South) => write!(f, "south"),
            Command::NativeDoor(Door::West) => write!(f, "west"),
            Command::NativeTake(name) => write!(f, "take {}", name),
            Command::NativeDrop(name) => write!(f, "drop {}", name),
            Command::NativeInv => write!(f, "inv"),
            Command::Trace(fname) => write!(f, "$trace {}", fname),
            Command::Map => write!(f, "$map"),
        }
    }
}

fn run_interactive(program: Vec<isize>) -> Result<(), String> {
    let mut process = State::new(program);
    let mut input: String = String::new();

    let mut rooms: HashMap<(i32, i32), Room> = HashMap::with_capacity(32);
    let mut input_trace: Vec<Command> = Vec::with_capacity(256);
    let mut pos: (i32, i32) = (0, 0);

    loop {
        let formal_input = parse_input(&input);
        let (state, status, output_raw) = run_program(process, &str_to_intcode_input(&input))?;
        process = state;
        let output = parse_output(&output_raw)?;

        match formal_input {
            Some(Command::NativeDoor(door)) => {
                // TODO: only insert new rooms
                // FIXME: map initial room as well (map is bugged otherwise)
                if let Ok(room) = parse_room(&output) {
                    // TODO: check if we actually moved (we may not have moved for some reason)
                    let (pos_x, pos_y) = pos;
                    let (d_x, d_y) = door.coordinates();
                    pos = (pos_x + d_x, pos_y + d_y);
                    rooms.insert(pos, room);
                } else {
                    print_error("Unable to parse room, room map may be incomplete from now on.");
                }
            }
            Some(Command::Trace(ref filename)) => {
                if let Err(e) = write_trace(&input_trace, filename) {
                    print_error(&format!("Error writing trace: {}", e));
                }
            }
            Some(Command::Map) => {
                print_map(&rooms, pos);
            }
            _ => {}
        }
        if let Some(traced) = formal_input {
            input_trace.push(traced);
        }

        print!("{}", output);
        if status == ReturnStatus::Halt {
            break;
        }
        input = read_input()?;
    }

    Ok(())
}

fn write_trace(input_trace: &[Command], filename: &str) -> std::io::Result<()> {
    let ofile = File::create(Path::new(filename))?;
    let mut buffered = BufWriter::new(ofile);
    for command in input_trace.iter().filter(|c| c.is_native()) {
        writeln!(&mut buffered, "{}", command)?;
    }
    buffered.flush()
}

fn print_error(error: &str) {
    // TODO: print on stderr
    println!("\x1b[1;31m{}\x1b[0m", error);
}

fn parse_output(output: &[isize]) -> Result<String, String> {
    output
        .iter()
        .map(|i| {
            u32::try_from(*i)
                .map_err(|e| e.to_string())
                .and_then(|u| char::try_from(u).map_err(|e| e.to_string()))
        })
        .collect()
}

fn read_input() -> Result<String, String> {
    let mut buf = String::with_capacity(128);
    stdin().read_line(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

fn parse_input(input: &str) -> Option<Command> {
    if input.starts_with("$trace ") {
        Some(Command::Trace(input["$trace ".len()..].trim().to_owned()))
    } else if input.trim() == "$map" {
        Some(Command::Map)
    } else if input.trim() == "inv" {
        Some(Command::NativeInv)
    } else if input.starts_with("take ") {
        Some(Command::NativeTake(
            input["take ".len()..].trim().to_owned(),
        ))
    } else if input.starts_with("drop ") {
        Some(Command::NativeDrop(
            input["drop ".len()..].trim().to_owned(),
        ))
    } else {
        parse_door_command(input).map(Command::NativeDoor)
    }
}

fn parse_door_command(input: &str) -> Option<Door> {
    match input.trim() {
        "north" => Some(Door::North),
        "east" => Some(Door::East),
        "south" => Some(Door::South),
        "west" => Some(Door::West),
        _ => None,
    }
}

fn str_to_intcode_input(input: &str) -> Vec<isize> {
    input.chars().map(|c| c as isize).collect()
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Door {
    North,
    East,
    South,
    West,
}

impl Door {
    fn coordinates(self) -> (i32, i32) {
        match self {
            Door::North => (0, -1),
            Door::East => (1, 0),
            Door::South => (0, 1),
            Door::West => (-1, 0),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Room {
    name: String,
    description: String,
    doors: Vec<Door>,
    items: Vec<String>,
}

fn print_map(rooms: &HashMap<(i32, i32), Room>, pos: (i32, i32)) {
    let min_x = rooms.keys().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = rooms.keys().map(|(_, y)| *y).min().unwrap_or(0);
    let max_x = rooms.keys().map(|(x, _)| *x).max().unwrap_or(0);
    let max_y = rooms.keys().map(|(_, y)| *y).max().unwrap_or(0);

    for y in min_y..=max_y {
        print_door_line(y, min_x, max_x, rooms, Door::North);
        println!();
        print_room_line(y, min_x, max_x, rooms, pos);
        println!();
        print_door_line(y, min_x, max_x, rooms, Door::South);
        println!();
    }
}

fn print_room_line(
    y: i32,
    min_x: i32,
    max_x: i32,
    rooms: &HashMap<(i32, i32), Room>,
    (pos_x, pos_y): (i32, i32),
) {
    for x in min_x..=max_x {
        if let Some(room) = rooms.get(&(x, y)) {
            print!(
                "{}",
                if room.doors.contains(&Door::West) {
                    "-"
                } else {
                    " "
                }
            );
            // TODO: color rooms with items (different color for dangerous items?)
            if pos_x == x && pos_y == y {
                print!("\x1b[32m0\x1b[0m");
            } else {
                print!("0");
            }
            print!(
                "{}",
                if room.doors.contains(&Door::East) {
                    "-"
                } else {
                    " "
                }
            );
        } else {
            print!("   ");
        }
    }
}

fn print_door_line(
    y: i32,
    min_x: i32,
    max_x: i32,
    rooms: &HashMap<(i32, i32), Room>,
    direction: Door,
) {
    for x in min_x..=max_x {
        if let Some(room) = rooms.get(&(x, y)) {
            print!(
                " {} ",
                if room.doors.contains(&direction) {
                    "|"
                } else {
                    " "
                }
            );
        } else {
            print!("   ");
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum ParseMode {
    Unknown,
    Doors,
    Items,
}

// I managed to avoid regexes so far, and I do not intend to change that.
fn parse_room(output: &str) -> Result<Room, String> {
    let mut lines = output.lines().skip_while(|l| l.trim().is_empty());
    let name = lines
        .next()
        .and_then(parse_room_name)
        .ok_or_else(|| "No name for room".to_owned())?;
    let description = lines.next().unwrap_or("");

    let mut parse_mode = ParseMode::Unknown;
    let mut doors: Vec<Door> = Vec::with_capacity(4);
    let mut items: Vec<String> = Vec::with_capacity(1);
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "Doors here lead:" {
            parse_mode = ParseMode::Doors;
            continue;
        } else if trimmed == "Items here:" {
            parse_mode = ParseMode::Items;
            continue;
        }
        match parse_mode {
            ParseMode::Doors => {
                if let Some(door) = parse_door(line) {
                    doors.push(door);
                } else {
                    parse_mode = ParseMode::Unknown;
                }
            }
            ParseMode::Items => {
                if let Some(item) = parse_item(line) {
                    items.push(item);
                } else {
                    parse_mode = ParseMode::Unknown;
                }
            }
            ParseMode::Unknown => {}
        };
    }

    Ok(Room {
        name: name.to_owned(),
        description: description.to_owned(),
        doors,
        items,
    })
}

fn parse_room_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.len() > 4 && trimmed.starts_with("==") && trimmed.ends_with("==") {
        Some(&trimmed[2..(trimmed.len() - 2)])
    } else {
        None
    }
}

fn parse_door(line: &str) -> Option<Door> {
    match line.trim() {
        "- north" => Some(Door::North),
        "- east" => Some(Door::East),
        "- south" => Some(Door::South),
        "- west" => Some(Door::West),
        _ => None,
    }
}

fn parse_item(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("- ") {
        Some(trimmed[2..].to_owned())
    } else {
        None
    }
}
