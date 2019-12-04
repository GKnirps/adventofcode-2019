use std::cmp;
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
    let lines: Vec<&str> = content.split('\n').collect();

    let (wire1, wire2) = parse_wires(&lines)?;

    if let Some(distance) =
        closest_crossing_manhattan(&wire1, &wire2).map(|(x, y)| central_distance(x, y))
    {
        println!("Distance to closest crossing: {}", distance);
    } else {
        println!("No crossings found!");
    }

    if let Some(distance) = closest_crossing_wire_length(&wire1, &wire2) {
        println!("Distance to closest crossing in wire length: {}", distance);
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Turn {
    Hor(i32),
    Ver(i32),
}

fn parse_turn(turn: &str) -> Result<Turn, String> {
    if turn.len() < 2 {
        return Err(format!("Invalid turn: {}", turn));
    }
    if !turn.is_char_boundary(1) {
        return Err(format!("Invalid turn: {}", turn));
    }
    let (dir, abs_raw) = turn.split_at(1);
    let abs: i32 = abs_raw
        .parse()
        .map_err(|e| format!("Invalid turn: {}, error: {}", turn, e))?;

    match dir {
        "U" => Ok(Turn::Ver(abs)),
        "R" => Ok(Turn::Hor(abs)),
        "D" => Ok(Turn::Ver(-abs)),
        "L" => Ok(Turn::Hor(-abs)),
        _ => Err(format!("Invalid turn: {}", turn)),
    }
}

fn parse_wire(line: &str) -> Result<Vec<Turn>, String> {
    line.split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(parse_turn)
        .collect()
}

fn parse_wires(lines: &[&str]) -> Result<(Vec<Turn>, Vec<Turn>), String> {
    let mut wires = lines.iter().map(|l| parse_wire(*l));

    let wire1 = wires
        .next()
        .transpose()?
        .ok_or_else(|| "No first wire.".to_owned())?;
    let wire2 = wires
        .next()
        .transpose()?
        .ok_or_else(|| "No second wire.".to_owned())?;

    Ok((wire1, wire2))
}

fn wire_positions(wire: &[Turn]) -> HashMap<(i32, i32), i32> {
    let mut wire_pos: HashMap<(i32, i32), i32> = HashMap::with_capacity(500);
    let mut current_pos = (0, 0);
    let mut current_steps = 0;
    for turn in wire {
        match turn {
            Turn::Ver(length) => {
                let (curr_x, curr_y) = current_pos;
                for y in cmp::min(curr_y, curr_y + length)..=cmp::max(curr_y, curr_y + length) {
                    let point = (curr_x, y);
                    let steps = current_steps + (y - curr_y).abs();
                    wire_pos
                        .entry(point)
                        .and_modify(|v| *v = cmp::min(*v, steps))
                        .or_insert(steps);
                }
                current_pos = (curr_x, curr_y + length);
                current_steps += length.abs();
            }
            Turn::Hor(length) => {
                let (curr_x, curr_y) = current_pos;
                for x in cmp::min(curr_x, curr_x + length)..=cmp::max(curr_x, curr_x + length) {
                    let point = (x, curr_y);
                    let steps = current_steps + (x - curr_x).abs();
                    wire_pos
                        .entry(point)
                        .and_modify(|v| *v = cmp::min(*v, steps))
                        .or_insert(steps);
                }
                current_pos = (curr_x + length, curr_y);
                current_steps += length.abs();
            }
        };
    }
    wire_pos
}

fn wire_crossings(wire1: &[Turn], wire2: &[Turn]) -> Vec<(i32, i32, i32)> {
    let wire_pos_1 = wire_positions(wire1);
    let wire_pos_2 = wire_positions(wire2);

    wire_pos_1
        .iter()
        .filter(|((x, y), _)| *x != 0 || *y != 0)
        .filter_map(|(pos, steps1)| {
            wire_pos_2
                .get(pos)
                .map(|steps2| (pos.0, pos.1, steps1 + steps2))
        })
        .collect()
}

fn central_distance(x: i32, y: i32) -> i32 {
    x.abs() + y.abs()
}

fn closest_crossing_manhattan(wire1: &[Turn], wire2: &[Turn]) -> Option<(i32, i32)> {
    let crossings = wire_crossings(wire1, wire2);
    crossings
        .iter()
        .map(|(x, y, _)| (x, y))
        .min_by_key(|(x, y)| central_distance(**x, **y))
        .map(|(x, y)| (*x, *y))
}

fn closest_crossing_wire_length(wire1: &[Turn], wire2: &[Turn]) -> Option<i32> {
    let crossings = wire_crossings(wire1, wire2);
    crossings.iter().map(|(_, _, d)| d).min().cloned()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_turn_should_parse_valid_turns() {
        assert_eq!(parse_turn("U35"), Ok(Turn::Ver(35)));
        assert_eq!(parse_turn("R2"), Ok(Turn::Hor(2)));
        assert_eq!(parse_turn("D2"), Ok(Turn::Ver(-2)));
        assert_eq!(parse_turn("L123"), Ok(Turn::Hor(-123)));
    }

    #[test]
    fn parse_turn_should_fail_on_invalid_turns() {
        assert!(parse_turn("U").is_err());
        assert!(parse_turn("Ü35").is_err());
        assert!(parse_turn("Ublubb").is_err());
        assert!(parse_turn("W20").is_err());
    }

    #[test]
    fn wire_crossings_should_work_for_example() {
        // given
        let (wire1, wire2) =
            parse_wires(&["R8,U5,L5,D3", "U7,R6,D4,L4"]).expect("Expected valid wires");

        // when
        let crossings = wire_crossings(&wire1, &wire2);

        // then
        assert_eq!(crossings.len(), 2);
        assert!(crossings.contains(&(3, 3)));
    }

    #[test]
    fn closest_crossing_should_work_for_first_example() {
        // given
        let (wire1, wire2) =
            parse_wires(&["R8,U5,L5,D3", "U7,R6,D4,L4"]).expect("Expected valid wires");

        // when
        let crossing = closest_crossing(&wire1, &wire2);

        // then
        assert_eq!(crossing, Some((3, 3)));
    }

    #[test]
    fn closest_crossing_should_work_for_second_example() {
        // given
        let (wire1, wire2) = parse_wires(&[
            "R75,D30,R83,U83,L12,D49,R71,U7,L72",
            "U62,R66,U55,R34,D71,R55,D58,R83",
        ])
        .expect("Expected valid wires");

        // when
        let crossing = closest_crossing(&wire1, &wire2).map(|(x, y)| central_distance(x, y));

        // then
        assert_eq!(crossing, Some(159));
    }

    #[test]
    fn closest_crossing_should_work_for_third_example() {
        // given
        let (wire1, wire2) = parse_wires(&[
            "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
            "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7",
        ])
        .expect("Expected valid wires");

        // when
        let crossing = closest_crossing(&wire1, &wire2).map(|(x, y)| central_distance(x, y));

        // then
        assert_eq!(crossing, Some(135));
    }
}
