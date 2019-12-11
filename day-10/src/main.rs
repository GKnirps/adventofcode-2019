use std::cmp;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.split('\n').filter(|s| !s.is_empty()).collect();

    let asteroids = parse_map(&lines);

    match max_visible_asteroids(&asteroids) {
        Some(n) => println!(
            "The best spot for a monitoring station can detect {} asteroids",
            n
        ),
        None => println!("There are no asteroids at all!"),
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

fn parse_map(lines: &[&str]) -> Vec<(i32, i32)> {
    lines
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .filter(|(_, c)| *c == '#')
                .map(move |(x, _)| (x as i32, y as i32))
        })
        .collect()
}

// this is a primitive approach but should work just fine for the dimensions we expect here
fn greatest_common_divisor(x: i32, y: i32) -> i32 {
    for d in (1..=cmp::min(x, y)).rev() {
        if x % d == 0 && y % d == 0 {
            return d;
        }
    }
    1
}

fn normalize(x: i32, y: i32) -> (i32, i32) {
    if x == 0 && y == 0 {
        (0, 0)
    } else if x == 0 {
        (0, y.signum())
    } else if y == 0 {
        (x.signum(), 0)
    } else {
        let gcd = greatest_common_divisor(x.abs(), y.abs());
        (x / gcd, y / gcd)
    }
}

fn count_visible_asteroids(x_base: i32, y_base: i32, asteroids: &[(i32, i32)]) -> usize {
    asteroids
        .iter()
        .filter(|a| **a != (x_base, y_base))
        .map(|(x, y)| normalize(x - x_base, y - y_base))
        .collect::<HashSet<(i32, i32)>>()
        .len()
}

fn max_visible_asteroids(asteroids: &[(i32, i32)]) -> Option<usize> {
    asteroids
        .iter()
        .map(|(x, y)| count_visible_asteroids(*x, *y, asteroids))
        .max()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_map() {
        // given
        let input = &[".#.#", "...#"];

        // when
        let map = parse_map(input);

        // then
        assert_eq!(map.len(), 3);
        assert!(map.contains(&(1, 0)));
        assert!(map.contains(&(3, 0)));
        assert!(map.contains(&(3, 1)));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(0, 0), (0, 0));
        assert_eq!(normalize(42, 0), (1, 0));
        assert_eq!(normalize(-42, 0), (-1, 0));
        assert_eq!(normalize(0, 42), (0, 1));
        assert_eq!(normalize(0, -42), (0, -1));
        assert_eq!(normalize(5, 10), (1, 2));
        assert_eq!(normalize(1, 2), (1, 2));
        assert_eq!(normalize(10, 5), (2, 1));
        assert_eq!(normalize(-10, 5), (-2, 1));
        assert_eq!(normalize(10, -5), (2, -1));
    }

    #[test]
    fn test_example_1() {
        // given
        let raw_map = &[".#..#", ".....", "#####", "....#", "...##"];
        let asteroids = parse_map(raw_map);

        // when
        let result = max_visible_asteroids(&asteroids).expect("Expected at least one asteroid");

        // then
        assert_eq!(result, 8);
    }

    #[test]
    fn test_example_2() {
        // given
        let raw_map = &[
            "......#.#.",
            "#..#.#....",
            "..#######.",
            ".#.#.###..",
            ".#..#.....",
            "..#....#.#",
            "#..#....#.",
            ".##.#..###",
            "##...#..#.",
            ".#....####",
        ];
        let asteroids = parse_map(raw_map);

        // when
        let result = max_visible_asteroids(&asteroids).expect("Expected at least one asteroid");

        // then
        assert_eq!(result, 33);
    }
}
