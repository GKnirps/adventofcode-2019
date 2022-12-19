use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (grid, (start_x, start_y)) = Grid::from_str(&content)?;

    println!("Start exploring…");
    let shortest_paths = explore(&grid, start_x, start_y);

    let all_keys: Keys = available_keys(&grid);

    println!("Number of visited nodes: {}", shortest_paths.len());

    let shortest_path_length = shortest_paths
        .iter()
        .filter(|(node, _)| node.keys == all_keys)
        .map(|(_, dist)| dist)
        .min();

    if let Some(d) = shortest_path_length {
        println!("Shortest path to all keys: {}", d);
    } else {
        println!("Did not find all keys?!?");
    }

    let (quad_grid, qs1, qs2, qs3, qs4) = grid.to_quad_grid((start_x, start_y))?;
    let quad_shortest_paths = quad_explore(&quad_grid, qs1, qs2, qs3, qs4);
    let quad_sp_length = quad_shortest_paths
        .iter()
        .filter(|(node, _)| node.keys == all_keys)
        .map(|(_, dist)| dist)
        .min();
    if let Some(d) = quad_sp_length {
        println!("Shortest path to all keys with four separate bots: {d}");
    } else {
        println!("Did not find all keys… maybe placing the additional walls removed some of them?");
    }

    Ok(())
}

fn available_keys(grid: &Grid) -> Keys {
    let mut all_keys: Keys = [false; MAX_KEYS];
    for key in grid.tiles.iter().filter_map(|tile| match tile {
        Tile::Key(c) => Some(*c),
        _ => None,
    }) {
        all_keys[key] = true;
    }
    all_keys
}

const MAX_KEYS: usize = 26;
type Keys = [bool; MAX_KEYS];

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
        let mut len_iter = lines.lines().map(|s| s.len());
        let size_x = len_iter
            .next()
            .ok_or_else(|| "No lines in the input".to_owned())?;
        if len_iter.any(|len| len != size_x) {
            return Err("Not all lines have the same length!".to_owned());
        }

        let size_y = lines.lines().count();
        let tiles = lines
            .bytes()
            .filter(|c| *c != b'\n')
            .map(Tile::from_byte)
            .collect::<Option<Vec<Tile>>>()
            .ok_or_else(|| "Found unknown input char".to_owned())?;

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

        let start_x = start % size_x;
        let start_y = start / size_x;

        Ok((grid, (start_x, start_y)))
    }

    fn to_quad_grid(mut self, (x, y): Vec2) -> Result<(Grid, Vec2, Vec2, Vec2, Vec2), String> {
        if x == 0 || y == 0 || x + 1 >= self.size_x || y + 1 >= self.size_y {
            return Err("Split position is on the border, unable to split map".to_owned());
        }
        self.set(x - 1, y, Tile::Wall);
        self.set(x, y - 1, Tile::Wall);
        self.set(x + 1, y, Tile::Wall);
        self.set(x, y + 1, Tile::Wall);

        Ok((
            self,
            (x - 1, y - 1),
            (x - 1, y + 1),
            (x + 1, y - 1),
            (x + 1, y + 1),
        ))
    }

    fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if let Some(t) = self.tiles.get_mut(y * self.size_x + x) {
            *t = tile;
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Tile> {
        self.tiles.get(y * self.size_x + x).copied()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
enum Tile {
    Wall,
    Floor,
    Key(usize),
    Door(usize),
}

impl Tile {
    fn from_byte(c: u8) -> Option<Tile> {
        if c == b'#' {
            Some(Tile::Wall)
        } else if c == b'.' || c == b'@' {
            Some(Tile::Floor)
        } else if c.is_ascii_lowercase() {
            Some(Tile::Key((c - b'a') as usize))
        } else if c.is_ascii_uppercase() {
            Some(Tile::Door((c - b'A') as usize))
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Node {
    x: usize,
    y: usize,
    keys: Keys,
}

#[derive(Clone, Eq, Debug)]
struct QueueEntry<T> {
    node: T,
    dist: usize,
}

impl<T: Eq> Ord for QueueEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist.cmp(&other.dist).reverse()
    }
}

impl<T: Eq> PartialOrd for QueueEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Eq> PartialEq for QueueEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

// This solution takes a lot of time and memory, probably because the number of nodes grows
// exponentially with the number of keys (come to think of it, is this related to the travelling salesman problem?)
fn explore(grid: &Grid, start_x: usize, start_y: usize) -> HashMap<Node, usize> {
    // The actual required capacity depends on how many keys there are, but whatever
    let mut visited: HashMap<Node, usize> = HashMap::with_capacity(grid.size_x * grid.size_y);
    let mut queue: BinaryHeap<QueueEntry<Node>> =
        BinaryHeap::with_capacity(grid.size_x * grid.size_y);
    queue.push(QueueEntry {
        node: Node {
            x: start_x,
            y: start_y,
            keys: [false; MAX_KEYS],
        },
        dist: 0,
    });

    while let Some(QueueEntry { node, dist }) = queue.pop() {
        if visited.contains_key(&node) {
            continue;
        }
        visited.insert(node.clone(), dist);
        // TODO: would it help to cache the results of this search?
        for key_dist in reachable_keys_with_keyset(grid, node.keys, (node.x, node.y)) {
            let mut keys = node.keys;
            keys[key_dist.key] = true;
            queue.push(QueueEntry {
                node: Node {
                    x: key_dist.pos.0,
                    y: key_dist.pos.1,
                    keys,
                },
                dist: dist + key_dist.dist,
            });
        }
    }

    visited
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct QuadNode {
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    p4: Vec2,
    keys: Keys,
}

fn quad_explore(
    grid: &Grid,
    sp1: Vec2,
    sp2: Vec2,
    sp3: Vec2,
    sp4: Vec2,
) -> HashMap<QuadNode, usize> {
    let mut visited: HashMap<QuadNode, usize> = HashMap::with_capacity(grid.size_x * grid.size_y);
    let mut queue: BinaryHeap<QueueEntry<QuadNode>> =
        BinaryHeap::with_capacity(grid.size_x * grid.size_y);
    queue.push(QueueEntry {
        node: QuadNode {
            p1: sp1,
            p2: sp2,
            p3: sp3,
            p4: sp4,
            keys: [false; MAX_KEYS],
        },
        dist: 0,
    });

    while let Some(QueueEntry { node, dist }) = queue.pop() {
        if visited.contains_key(&node) {
            continue;
        }
        visited.insert(node.clone(), dist);
        // TODO: would it help to cache the results of this search?
        for key_dist in reachable_keys_with_keyset(grid, node.keys, node.p1) {
            let mut keys = node.keys;
            keys[key_dist.key] = true;
            queue.push(QueueEntry {
                node: QuadNode {
                    p1: key_dist.pos,
                    keys,
                    ..node
                },
                dist: dist + key_dist.dist,
            });
        }
        for key_dist in reachable_keys_with_keyset(grid, node.keys, node.p2) {
            let mut keys = node.keys;
            keys[key_dist.key] = true;
            queue.push(QueueEntry {
                node: QuadNode {
                    p2: key_dist.pos,
                    keys,
                    ..node
                },
                dist: dist + key_dist.dist,
            });
        }
        for key_dist in reachable_keys_with_keyset(grid, node.keys, node.p3) {
            let mut keys = node.keys;
            keys[key_dist.key] = true;
            queue.push(QueueEntry {
                node: QuadNode {
                    p3: key_dist.pos,
                    keys,
                    ..node
                },
                dist: dist + key_dist.dist,
            });
        }
        for key_dist in reachable_keys_with_keyset(grid, node.keys, node.p4) {
            let mut keys = node.keys;
            keys[key_dist.key] = true;
            queue.push(QueueEntry {
                node: QuadNode {
                    p4: key_dist.pos,
                    keys,
                    ..node
                },
                dist: dist + key_dist.dist,
            });
        }
    }

    visited
}

#[derive(Copy, Clone, Debug)]
struct KeyDist {
    key: usize,
    pos: Vec2,
    dist: usize,
}

// find all keys that are reachable with the current key set (i.e. without collecting more
// keys). This excludes keys behind doors, but also keys that can only be reached by passing over a
// square that contains a key (as that would extend the key set)
fn reachable_keys_with_keyset(grid: &Grid, keys: Keys, start_pos: Vec2) -> Vec<KeyDist> {
    let mut queue: VecDeque<(Vec2, usize)> = VecDeque::with_capacity(grid.tiles.len());
    let mut seen: HashSet<Vec2> = HashSet::with_capacity(grid.tiles.len());
    let mut found_keys: Vec<KeyDist> = Vec::with_capacity(MAX_KEYS);

    queue.push_back((start_pos, 0));

    while let Some((pos, dist)) = queue.pop_front() {
        if seen.contains(&pos) {
            continue;
        }
        seen.insert(pos);
        match grid.get(pos.0, pos.1) {
            Some(Tile::Key(key)) => {
                if keys[key] {
                    for n in neighbours(pos, grid).into_iter().flatten() {
                        queue.push_back((n, dist + 1));
                    }
                } else {
                    found_keys.push(KeyDist { key, pos, dist });
                }
            }
            Some(Tile::Door(key)) => {
                if keys[key] {
                    for n in neighbours(pos, grid).into_iter().flatten() {
                        queue.push_back((n, dist + 1));
                    }
                }
            }
            Some(Tile::Floor) => {
                for n in neighbours(pos, grid).into_iter().flatten() {
                    queue.push_back((n, dist + 1));
                }
            }
            _ => (),
        }
    }

    found_keys
}

fn neighbours((x, y): Vec2, grid: &Grid) -> [Option<Vec2>; 4] {
    let mut result = [None; 4];
    if x > 0 {
        result[0] = grid
            .get(x - 1, y)
            .filter(|t| t != &Tile::Wall)
            .map(|_| (x - 1, y));
    }
    if y > 0 {
        result[1] = grid
            .get(x, y - 1)
            .filter(|t| t != &Tile::Wall)
            .map(|_| (x, y - 1));
    }
    result[2] = grid
        .get(x + 1, y)
        .filter(|t| t != &Tile::Wall)
        .map(|_| (x + 1, y));
    result[3] = grid
        .get(x, y + 1)
        .filter(|t| t != &Tile::Wall)
        .map(|_| (x, y + 1));

    result
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"#################
#i.G..c...e..H.p#
########.########
#j.A..b...f..D.o#
########@########
#k.E..a...g..B.n#
########.########
#l.F..d...h..C.m#
#################
"#;

    #[test]
    fn explore_works_for_simplest_example() {
        // given
        let (grid, (sx, sy)) = Grid::from_str(
            r#"#########
#b.A.@.a#
#########
"#,
        )
        .expect("Expected successful parsing");
        let all_keys = available_keys(&grid);

        // when
        let visited = explore(&grid, sx, sy);

        // then
        let shortest_path_length = visited
            .iter()
            .filter(|(node, _)| node.keys == all_keys)
            .map(|(_, dist)| dist)
            .min()
            .copied();

        assert_eq!(shortest_path_length, Some(8));
    }

    #[test]
    fn explore_works_for_example() {
        // given
        let (grid, (sx, sy)) = Grid::from_str(EXAMPLE).expect("Expected successful parsing");
        let all_keys = available_keys(&grid);

        // when
        let visited = explore(&grid, sx, sy);

        // then
        let shortest_path_length = visited
            .iter()
            .filter(|(node, _)| node.keys == all_keys)
            .map(|(_, dist)| dist)
            .min()
            .copied();

        assert_eq!(shortest_path_length, Some(136));
    }
}
