use std::collections::{BTreeSet, HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::rc::Rc;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let (grid, (start_x, start_y)) = Grid::from_str(&content)?;

    println!("Start exploring…");
    let shortest_paths = explore(&grid, start_x, start_y);

    let all_keys: BTreeSet<char> = grid
        .tiles
        .iter()
        .filter_map(|tile| match tile {
            Tile::Key(c) => Some(*c),
            _ => None,
        })
        .collect();

    println!("Number of visited nodes: {}", shortest_paths.len());
    println!("all keys: {:?}", all_keys);

    let shortest_path_length = shortest_paths
        .iter()
        .filter(|(node, _)| *node.keys == all_keys)
        .map(|(_, dist)| dist)
        .min();

    if let Some(d) = shortest_path_length {
        println!("Shortest path to all keys: {}", d);
    } else {
        println!("Did not find all keys?!?");
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

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Grid {
    tiles: Vec<Tile>,
    size_x: usize,
    size_y: usize,
}

impl Grid {
    fn from_str(lines: &str) -> Result<(Grid, Vec2), String> {
        let mut len_iter = lines.split('\n').filter(|s| !s.is_empty()).map(|s| s.len());
        let size_x = len_iter
            .next()
            .ok_or_else(|| "No lines in the input".to_owned())?;
        if len_iter.any(|len| len != size_x) {
            return Err("Not all lines have the same length!".to_owned());
        }

        let size_y = lines.split('\n').filter(|s| !s.is_empty()).count();
        let tiles = lines
            .chars()
            .filter(|c| *c != '\n')
            .map(Tile::from_char)
            .collect::<Option<Vec<Tile>>>()
            .ok_or_else(|| "Found unknown input char")?;

        let grid = Grid {
            tiles,
            size_x,
            size_y,
        };

        let start = lines
            .chars()
            .filter(|c| *c != '\n')
            .position(|c| c == '@')
            .ok_or_else(|| "Did not find player position".to_owned())?;

        let start_x = start % size_y;
        let start_y = start / size_y;

        Ok((grid, (start_x, start_y)))
    }

    fn get(&self, x: usize, y: usize) -> Option<Tile> {
        self.tiles.get(y * self.size_x + x).copied()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Tile {
    Wall,
    Floor,
    Key(char),
    Door(char),
}

impl Tile {
    fn from_char(c: char) -> Option<Tile> {
        if c == '#' {
            Some(Tile::Wall)
        } else if c == '.' || c == '@' {
            Some(Tile::Floor)
        } else if c.is_ascii_alphabetic() {
            if c.is_lowercase() {
                Some(Tile::Key(c))
            } else {
                Some(Tile::Door(c.to_ascii_lowercase()))
            }
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Node {
    x: usize,
    y: usize,
    // We use a BTreeSet for the keys because HashMap is not hashable.
    // We do not use a Vec because we need set properties
    keys: Rc<BTreeSet<char>>,
}

impl Node {
    fn with_pos(&self, x: usize, y: usize) -> Node {
        Node {
            x,
            y,
            keys: self.keys.clone(),
        }
    }
    fn with_key(&self, key: char, x: usize, y: usize) -> Node {
        let mut keys: BTreeSet<char> = (*self.keys).clone();
        keys.insert(key);
        Node {
            x,
            y,
            keys: Rc::new(keys),
        }
    }
}

// This solution takes a lot of time and memory, probably because the number of nodes grows
// exponentially with the number of keys (come to think of it, is this related to the travelling salesman problem?)
fn explore(grid: &Grid, start_x: usize, start_y: usize) -> HashMap<Node, usize> {
    // The actual required capacity depends on how many keys there are, but whatever
    let mut visited: HashMap<Node, usize> = HashMap::with_capacity(grid.size_x * grid.size_y);
    let mut queue: VecDeque<(Node, usize)> = VecDeque::with_capacity(grid.size_x * grid.size_y);
    queue.push_back((
        Node {
            x: start_x,
            y: start_y,
            keys: Rc::new(BTreeSet::new()),
        },
        0,
    ));

    while let Some((node, dist)) = queue.pop_front() {
        if visited.contains_key(&node) {
            continue;
        }
        for neighbour in neighbours(&node, grid) {
            queue.push_back((neighbour, dist + 1));
        }
        visited.insert(node, dist);
    }

    visited
}

fn neighbours(node: &Node, grid: &Grid) -> Vec<Node> {
    let mut result: Vec<Node> = Vec::with_capacity(4);
    let Node { x, y, .. } = node;
    if *x > 0 {
        if let Some(neighbour) = check_neighbour(node, grid, x - 1, *y) {
            result.push(neighbour);
        }
    }
    if *y > 0 {
        if let Some(neighbour) = check_neighbour(node, grid, *x, y - 1) {
            result.push(neighbour);
        }
    }
    if let Some(neighbour) = check_neighbour(node, grid, x + 1, *y) {
        result.push(neighbour);
    }
    if let Some(neighbour) = check_neighbour(node, grid, *x, y + 1) {
        result.push(neighbour);
    }

    result
}

fn check_neighbour(node: &Node, grid: &Grid, other_x: usize, other_y: usize) -> Option<Node> {
    match grid.get(other_x, other_y) {
        Some(Tile::Floor) => Some(node.with_pos(other_x, other_y)),
        Some(Tile::Key(key)) => Some(node.with_key(key, other_x, other_y)),
        Some(Tile::Door(key)) => {
            if node.keys.contains(&key) {
                Some(node.with_pos(other_x, other_y))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
