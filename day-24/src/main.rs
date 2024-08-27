use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let initial_grid = parse_input(&content);

    let first_repeated_pattern = run_until_repeat(initial_grid);
    println!(
        "The biodiversity of the first repeated pattern is {}",
        first_repeated_pattern.biodiversity()
    );

    let recursive_bugs = run_recursive(200, initial_grid);
    println!("With the recursive grid, there are {recursive_bugs} bugs after 200 minutes");

    Ok(())
}

fn parse_input(input: &str) -> Grid {
    // we just assume the input is formatted correctly
    Grid(
        input
            .chars()
            .rev()
            .filter_map(|c| match c {
                '#' => Some(1),
                '.' => Some(0),
                _ => None,
            })
            .take(25)
            .fold(0u32, |mut grid, tile| {
                grid <<= 1;
                grid | tile
            }),
    )
}

fn run_until_repeat(mut grid: Grid) -> Grid {
    let mut seen = HashSet::with_capacity(1024);

    while !seen.contains(&grid) {
        seen.insert(grid);
        grid = grid.next();
    }
    grid
}

// I'd like to do this with just a u32 as grid. I will probably regret this in part 2
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug, Default)]
struct Grid(u32);

const GRID_MASK: u32 = 0b1111111111111111111111111;

impl Grid {
    fn biodiversity(self) -> u32 {
        self.0 & GRID_MASK
    }

    fn next(self) -> Grid {
        let mut next_grid = 0;
        for y in 0i8..5 {
            for x in 0i8..5 {
                let neighbours_alive =
                    self.at(x - 1, y) + self.at(x + 1, y) + self.at(x, y - 1) + self.at(x, y + 1);
                let alive = self.at(x, y);
                if (alive == 0 && (neighbours_alive == 1 || neighbours_alive == 2))
                    || (alive == 1 && neighbours_alive == 1)
                {
                    next_grid |= 1 << (x + y * 5);
                }
            }
        }
        Grid(next_grid)
    }

    fn at(self, x: i8, y: i8) -> u32 {
        if x < 0 || y < 0 || x >= 5 || y >= 5 {
            0
        } else {
            (self.0 >> (y * 5 + x)) & 1
        }
    }

    fn count(mut self) -> u32 {
        let mut count = 0;
        while self.0 != 0 {
            count += self.0 & 1;
            self.0 >>= 1;
        }
        count
    }
}

fn run_recursive(n: usize, initial_pattern: Grid) -> u32 {
    let mut grid = vec![Grid::default(); n * 2 + 1];
    grid[n] = initial_pattern;

    for _ in 0..n {
        grid = recursive_grid_next(&grid);
    }
    grid.iter().map(|g| g.count()).sum()
}

// we do not do infinite recursion here. We assume the recursion is finite. The caller has to make
// sure that there are free grids in the innermost and outermost layer if they want to expand the
// recursion
fn recursive_grid_next(grid: &[Grid]) -> Vec<Grid> {
    let mut next_grid: Vec<Grid> = vec![Grid::default(); grid.len()];
    for grid_layer in 0..grid.len() {
        for y in 0i8..5 {
            for x in 0i8..5 {
                // ignore the center, that is the recursion tile
                if x != 2 || y != 2 {
                    let neighbours_alive = recursive_neighbours(grid, grid_layer, x, y);
                    let alive = grid[grid_layer].at(x, y);
                    if (alive == 0 && (neighbours_alive == 1 || neighbours_alive == 2))
                        || (alive == 1 && neighbours_alive == 1)
                    {
                        next_grid[grid_layer].0 |= 1 << (x + y * 5);
                    }
                }
            }
        }
    }
    next_grid
}

fn recursive_neighbours(grid: &[Grid], layer: usize, x: i8, y: i8) -> u32 {
    let mut alive: u32 = 0;
    if layer > 0 {
        if x == 0 {
            alive += grid[layer - 1].at(1, 2);
        }
        if x == 4 {
            alive += grid[layer - 1].at(3, 2);
        }
        if y == 0 {
            alive += grid[layer - 1].at(2, 1);
        }
        if y == 4 {
            alive += grid[layer - 1].at(2, 3);
        }
    }
    if layer < grid.len() - 1 {
        if x == 1 && y == 2 {
            alive += grid[layer + 1].at(0, 0)
                + grid[layer + 1].at(0, 1)
                + grid[layer + 1].at(0, 2)
                + grid[layer + 1].at(0, 3)
                + grid[layer + 1].at(0, 4);
        }
        if x == 3 && y == 2 {
            alive += grid[layer + 1].at(4, 0)
                + grid[layer + 1].at(4, 1)
                + grid[layer + 1].at(4, 2)
                + grid[layer + 1].at(4, 3)
                + grid[layer + 1].at(4, 4);
        }
        if x == 2 && y == 1 {
            alive += grid[layer + 1].at(0, 0)
                + grid[layer + 1].at(1, 0)
                + grid[layer + 1].at(2, 0)
                + grid[layer + 1].at(3, 0)
                + grid[layer + 1].at(4, 0);
        }
        if x == 2 && y == 3 {
            alive += grid[layer + 1].at(0, 4)
                + grid[layer + 1].at(1, 4)
                + grid[layer + 1].at(2, 4)
                + grid[layer + 1].at(3, 4)
                + grid[layer + 1].at(4, 4);
        }
    }
    // the dummy tile in the center is always 0, so we can add it without danger
    alive += grid[layer].at(x - 1, y)
        + grid[layer].at(x + 1, y)
        + grid[layer].at(x, y - 1)
        + grid[layer].at(x, y + 1);
    alive
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_input_parses_correctly() {
        // given
        let input = r".....
.....
.....
#....
.#...
";

        // when
        let grid = parse_input(input);

        // then
        assert_eq!(grid, Grid(2129920));
    }

    #[test]
    fn step_works_for_example() {
        // given
        let first = parse_input(
            r"....#
#..#.
#..##
..#..
#....
",
        );
        let second = parse_input(
            r"#..#.
####.
###.#
##.##
.##..
",
        );

        // when
        let next = first.next();

        // then
        assert_eq!(next, second);
    }

    #[test]
    fn run_until_repeat_works_for_example() {
        // given
        let initial = parse_input(
            r"....#
#..#.
#..##
..#..
#....
",
        );

        // when
        let biodiv = run_until_repeat(initial).biodiversity();

        // then
        assert_eq!(biodiv, 2129920);
    }

    #[test]
    fn run_recursive_works_for_example() {
        // given
        let initial = parse_input(
            r"....#
#..#.
#..##
..#..
#....
",
        );

        // when
        let count = run_recursive(10, initial);

        // then
        assert_eq!(count, 99);
    }
}
