use std::cmp::{self, Ordering};
use std::collections::{HashMap, HashSet};
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
        Some((n, x, y)) => {
            println!(
                "The best spot for a monitoring station is ({}, {}) can detect {} asteroids",
                x, y, n
            );
            if let Some((x200, y200)) = find_200th_asteroid(x, y, &asteroids) {
                println!(
                    "The 200th asteroid vaporized by the laser is at ({}, {}), result is {}",
                    x200,
                    y200,
                    x200 * 100 + y200
                );
            }
        }
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

fn max_visible_asteroids(asteroids: &[(i32, i32)]) -> Option<(usize, i32, i32)> {
    asteroids
        .iter()
        .map(|(x, y)| (count_visible_asteroids(*x, *y, asteroids), *x, *y))
        .max_by_key(|(count, _, _)| *count)
}

fn asteroids_by_direction(
    x_base: i32,
    y_base: i32,
    asteroids: &[(i32, i32)],
) -> HashMap<(i32, i32), Vec<(i32, i32)>> {
    let mut ast_by_dir = asteroids.iter().filter(|a| **a != (x_base, y_base)).fold(
        HashMap::with_capacity(asteroids.len()),
        |mut map, (x, y)| {
            let dir = normalize(x - x_base, y - y_base);
            map.entry(dir)
                .or_insert_with(|| Vec::with_capacity(asteroids.len()))
                .push((*x, *y));
            map
        },
    );

    for list in ast_by_dir.values_mut() {
        list.sort_by_key(|(x, y)| (x - x_base).abs() + (y - y_base).abs());
    }
    ast_by_dir
}

fn find_200th_asteroid(x_base: i32, y_base: i32, asteroids: &[(i32, i32)]) -> Option<(i32, i32)> {
    let by_dir = asteroids_by_direction(x_base, y_base, asteroids);
    let mut entries: Vec<((i32, i32), Vec<(i32, i32)>)> = by_dir.into_iter().collect();
    entries.sort_by(|(dir1, _), (dir2, _)| compare_vec_angle(*dir1, *dir2));
    let targets: Vec<Vec<(i32, i32)>> = entries.into_iter().map(|(_, a)| a).collect();

    let max_length = targets.iter().map(|v| v.len()).max()?;
    let mut count: usize = 0;

    for index in 0..max_length {
        for target in &targets {
            if let Some(asteroid) = target.get(index) {
                if count < 199 {
                    count += 1;
                } else {
                    return Some(*asteroid);
                }
            }
        }
    }

    None
}

fn scal_prod((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> f64 {
    (x1 * x2 + y1 * y2) as f64
}

fn compare_vec_angle(v1: (i32, i32), v2: (i32, i32)) -> Ordering {
    let (x1, _) = v1;
    let (x2, _) = v2;
    let len1 = scal_prod(v1, v1).sqrt();
    let len2 = scal_prod(v2, v2).sqrt();
    if len1 == 0.0 || len2 == 0.0 {
        // not really an angle to calculate, make zero length vectors less than everything else as fallback
        return len1
            .partial_cmp(&len2)
            .expect("We have a weird floating point value here");
    }

    let base: (i32, i32) = (0, -1);

    let a1 = (scal_prod(base, v1) / len1).acos();
    let a2 = (scal_prod(base, v2) / len2).acos();

    let full_angle1 = if x1 < 0 {
        std::f64::consts::PI * 2.0 - a1
    } else {
        a1
    };
    let full_angle2 = if x2 < 0 {
        std::f64::consts::PI * 2.0 - a2
    } else {
        a2
    };

    // this is the reason I wanted to avoid floating point numbers at first.
    // I could handle that without a panic in theory, but I have no nerve for that now.
    full_angle1
        .partial_cmp(&full_angle2)
        .expect("We have a weird floating point value here")
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
        assert_eq!(result, (8, 3, 4));
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
        assert_eq!(result, (33, 5, 8));
    }

    #[test]
    fn test_compare_vec_angle() {
        // given
        let v_pairs = &[
            ((0, 0), (0, 0), Ordering::Equal),
            ((0, -1), (-1, -1), Ordering::Less),
            ((0, -1), (-1, 0), Ordering::Less),
            ((0, -1), (-5, 0), Ordering::Less),
            ((0, -1), (-1, 1), Ordering::Less),
            ((0, -1), (0, -1), Ordering::Equal),
            ((0, -1), (0, 0), Ordering::Greater), // by my definition, actually, this is not defined
            ((0, -1), (0, 1), Ordering::Less),
            ((0, -1), (1, -1), Ordering::Less),
            ((0, -1), (1, 0), Ordering::Less),
            ((0, -1), (1, 1), Ordering::Less),
            ((1, 1), (2000, 2000), Ordering::Equal),
            ((1, 1), (1, -2), Ordering::Greater),
            ((1, 1), (1, 2), Ordering::Less),
            ((1, 1), (2, 1), Ordering::Greater),
            ((1, 1), (-1, 1), Ordering::Less),
            ((-1, -12), (-1, -11), Ordering::Greater),
            ((-10, -1), (-8, 1), Ordering::Greater),
        ];

        for (v1, v2, ord) in v_pairs {
            println!("Comparing {:?} and {:?}", v1, v2);
            assert_eq!(compare_vec_angle(*v1, *v2), *ord);
            assert_eq!(compare_vec_angle(*v2, *v1), ord.reverse());
        }
    }

    #[test]
    fn test_example_laser() {
        // given
        let raw_map = &[
            ".#..##.###...#######",
            "##.############..##.",
            ".#.######.########.#",
            ".###.#######.####.#.",
            "#####.##.#.##.###.##",
            "..#####..#.#########",
            "####################",
            "#.####....###.#.#.##",
            "##.#################",
            "#####.##.###..####..",
            "..######..##.#######",
            "####.##.####...##..#",
            ".#####..#.######.###",
            "##...#.##########...",
            "#.##########.#######",
            ".####.#.###.###.#.##",
            "....##.##.###..#####",
            ".#.#.###########.###",
            "#.#.#.#####.####.###",
            "###.##.####.##.#..##",
        ];
        let asteroids = parse_map(raw_map);

        // when
        let result = find_200th_asteroid(11, 13, &asteroids);

        // then
        assert_eq!(result, Some((8, 2)));
    }
}
