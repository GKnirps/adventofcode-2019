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
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
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
}
