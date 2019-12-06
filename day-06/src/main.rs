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

    let map = parse_map(&lines)?;
    let n_orbits = count_orbits(&map);
    println!(
        "There are {} direct and indirect orbits in the input",
        n_orbits
    );

    let path_length = path_length_to_santa(&map)?;
    println!("Number of orbital transfers required: {}", path_length);

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn parse_orbit(line: &str) -> Result<(&str, &str), String> {
    let mut splitted = line.splitn(2, ')');
    let orbited = splitted
        .next()
        .ok_or_else(|| "Expected an orbited object".to_owned())?;
    let orbiting = splitted
        .next()
        .ok_or_else(|| "Expected an orbiting object".to_owned())?;

    Ok((orbited, orbiting))
}

fn parse_map<'a>(lines: &[&'a str]) -> Result<HashMap<&'a str, Vec<&'a str>>, String> {
    let mut result = HashMap::with_capacity(lines.len());
    for pair in lines
        .iter()
        .map(|s| (*s).trim())
        .filter(|s| !s.is_empty())
        .map(|s| parse_orbit(s))
    {
        let (orbited, orbiting) = pair?;
        result
            .entry(orbited)
            .or_insert_with(|| Vec::with_capacity(10))
            .push(orbiting);
    }
    Ok(result)
}

fn count_orbits(map: &HashMap<&str, Vec<&str>>) -> usize {
    let mut stack: Vec<(&str, usize)> = Vec::with_capacity(map.len() * 2);
    let mut counter = 0;
    stack.push(("COM", 0));
    // We did not check that the map contains no circles. We assume it does not.
    // If it does, we will get into an infinite loop here. But until that happens
    // with provided inputs, I am too lazy to fix it.
    while let Some((name, depth)) = stack.pop() {
        if let Some(children) = map.get(name) {
            counter += children.len() * (depth + 1);
            for child in children {
                stack.push((child, depth + 1));
            }
        }
    }

    counter
}

fn find_path_from<'a>(map: &HashMap<&'a str, Vec<&'a str>>, target: &str) -> Option<Vec<&'a str>> {
    let mut visited: HashMap<&str, Option<&str>> = HashMap::with_capacity(map.len() * 2);
    let mut stack: Vec<&str> = Vec::with_capacity(map.len() * 2);
    stack.push("COM");
    visited.insert("COM", None);
    while let Some(obj) = stack.pop() {
        if let Some(children) = map.get(obj) {
            for child in children {
                if *child == target {
                    let mut path: Vec<&str> = Vec::with_capacity(visited.len());
                    path.push(obj);
                    let mut current = obj;
                    while let Some(parent) = visited.get(current).and_then(|p| p.as_ref()) {
                        path.push(parent);
                        current = parent;
                    }
                    return Some(path);
                } else if !visited.contains_key(child) {
                    // for a DAG, visited should never contain the child of the current node
                    stack.push(child);
                    visited.insert(child, Some(obj));
                }
            }
        }
    }
    None
}

fn path_length_to_santa(map: &HashMap<&str, Vec<&str>>) -> Result<usize, String> {
    let path_from_you =
        find_path_from(map, "YOU").ok_or_else(|| "You are not on the map".to_owned())?;
    let path_from_santa =
        find_path_from(map, "SAN").ok_or_else(|| "Santa is nowhere to be found".to_owned())?;

    for (you_index, obj) in path_from_you.iter().enumerate() {
        if let Some((san_index, _)) = path_from_santa
            .iter()
            .enumerate()
            .find(|(_, san_obj)| *san_obj == obj)
        {
            return Ok(you_index + san_index);
        }
    }
    Err("There seems to be no way to get to Santa".to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    static SAMPLE_INPUT: &[&str] = &[
        "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L",
    ];

    #[test]
    fn test_parse_map() {
        // given
        let input = SAMPLE_INPUT;

        // when
        let result = parse_map(input).expect("Expected successful parsing");

        // then
        assert_eq!(result.len(), 8);
        assert_eq!(result.get("COM"), Some(&vec!("B")));
        assert_eq!(result.get("B"), Some(&vec!("C", "G")));
        assert_eq!(result.get("C"), Some(&vec!("D")));
        assert!(result.get("L").is_none());
    }

    #[test]
    fn count_orbits_should_work_for_example() {
        // given
        let map = parse_map(SAMPLE_INPUT).expect("Expected successul parsing");

        // when
        let result = count_orbits(&map);

        // then
        assert_eq!(result, 42);
    }

    #[test]
    fn path_length_to_santa_should_work_for_example() {
        // given
        let raw_map = &[
            "COM)B", "B)C", "C)D", "D)E", "E)F", "B)G", "G)H", "D)I", "E)J", "J)K", "K)L", "K)YOU",
            "I)SAN",
        ];
        let map = parse_map(raw_map).expect("Expected successful parsing");

        // when
        let result = path_length_to_santa(&map).expect("Expected a path");

        // then
        assert_eq!(result, 4);
    }
}
