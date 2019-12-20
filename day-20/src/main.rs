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
    let lines: Vec<&str> = content.split('\n').collect();

    let grid = Grid::from_lines(&lines)?;

    if let Some(shortest_path) = path_length_to_exit(&grid) {
        println!("Shortest path to exit: {}", shortest_path);
    } else {
        println!("No path to exit");
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

type Vec2 = (usize, usize);

#[derive(Clone, PartialEq, Eq, Debug)]
struct Grid {
    tiles: Vec<char>,
    size: Vec2,
    warps: HashMap<Vec2, Vec2>,
    start: Vec2,
    end: Vec2,
}

impl Grid {
    fn from_lines(lines: &[&str]) -> Result<Grid, String> {
        let size_x = line_length(lines)?;
        let size_y = lines.len();

        if size_x < 6 || size_y < 6 {
            return Err("Input too small".to_owned());
        }

        let tiles: Vec<char> = lines.iter().flat_map(|line| line.chars()).collect();

        let mut warp_endpoints: HashMap<(char, char), Vec2> = HashMap::with_capacity(128);
        let mut warps: HashMap<Vec2, Vec2> = HashMap::with_capacity(128);

        for (i, c1) in tiles.iter().enumerate() {
            if let Some((id, pos)) = check_for_warp(*c1, i, size_x, &tiles) {
                if let Some(other_pos) = warp_endpoints.remove(&id) {
                    warps.insert(pos, other_pos);
                    warps.insert(other_pos, pos);
                } else {
                    warp_endpoints.insert(id, pos);
                }
            }
        }

        if warp_endpoints.len() != 2 {
            return Err(format!("Unmatched warp endpoints: {:?}", warp_endpoints));
        }
        let start = *warp_endpoints
            .get(&('A', 'A'))
            .ok_or_else(|| "No start point found".to_owned())?;
        let end = *warp_endpoints
            .get(&('Z', 'Z'))
            .ok_or_else(|| "No end point found".to_owned())?;

        Ok(Grid {
            tiles,
            size: (size_x, size_y),
            warps,
            start,
            end,
        })
    }

    fn get(&self, x: usize, y: usize) -> Option<char> {
        self.tiles.get(self.size.0 * y + x).copied()
    }
}

fn line_length(lines: &[&str]) -> Result<usize, String> {
    let length = lines
        .first()
        .map(|l| l.len())
        .ok_or_else(|| "No lines in input".to_owned())?;
    if !lines.iter().any(|l| l.len() != length) {
        return Err("Not all lines have the same length!".to_owned());
    }
    Ok(length)
}

fn check_for_warp(
    c1: char,
    i: usize,
    size_x: usize,
    tiles: &[char],
) -> Option<((char, char), Vec2)> {
    if c1.is_ascii_alphabetic() {
        let x = i % size_x;
        let y = i / size_x;
        if let Some(c2) = tiles
            .get(i + 1)
            .filter(|c2| c2.is_ascii_alphabetic())
            .copied()
        {
            if x > 0 && Some(&'.') == tiles.get(i - 1) {
                return Some(((c1, c2), (x - 1, y)));
            } else if Some(&'.') == tiles.get(i + 2) {
                return Some(((c1, c2), (x + 2, y)));
            }
        } else if let Some(c2) = tiles
            .get((y + 1) * size_x + x)
            .filter(|c2| c2.is_ascii_alphabetic())
            .copied()
        {
            if y > 0 && Some(&'.') == tiles.get((y - 1) * size_x + x) {
                return Some(((c1, c2), (x, y - 1)));
            } else if Some(&'.') == tiles.get((y + 2) * size_x + x) {
                return Some(((c1, c2), (x, y + 2)));
            }
        }
    }

    None
}

fn path_length_to_exit(grid: &Grid) -> Option<usize> {
    let mut visited: HashSet<Vec2> = HashSet::with_capacity(grid.tiles.len());
    let mut queue: VecDeque<(Vec2, usize)> = VecDeque::with_capacity(grid.tiles.len());
    queue.push_back((grid.start, 0));

    while let Some((pos, dist)) = queue.pop_front() {
        if pos == grid.end {
            return Some(dist);
        }
        if visited.contains(&pos) {
            continue;
        }
        visited.insert(pos);
        for neighbour in neighbours(grid, pos) {
            queue.push_back((neighbour, dist + 1));
        }
        if let Some(warp_pos) = grid.warps.get(&pos) {
            queue.push_back((*warp_pos, dist + 1));
        }
    }
    None
}

fn neighbours(grid: &Grid, (x, y): Vec2) -> Vec<Vec2> {
    let mut result: Vec<Vec2> = Vec::with_capacity(4);
    if x > 0 && Some('.') == grid.get(x - 1, y) {
        result.push((x - 1, y));
    }
    if y > 0 && Some('.') == grid.get(x, y - 1) {
        result.push((x, y - 1));
    }
    if Some('.') == grid.get(x + 1, y) {
        result.push((x + 1, y));
    }
    if Some('.') == grid.get(x, y + 1) {
        result.push((x, y + 1));
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;
}
