use itertools::Itertools;
use rand::{Rng, seq::IndexedRandom};
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};
use wasm_bindgen::prelude::*;

use crate::lexicon::{Dictionary, FRENCH_LETTERS_TABLE, LetterIndex, LettersTable, Word};
use priority_queue::PriorityQueue;

#[derive(Debug, thiserror::Error, PartialEq)]
pub(crate) enum GridError {
    #[error("no path found between ({0},{1}) and ({2},{3})", from.x, from.y, to.x, to.y)]
    NoPath { from: Position, to: Position },
    #[error("invalid position ({0},{1}), not in grid", pos.x, pos.y)]
    NotInGrid { pos: Position },
    #[error("invalid grid size {size}")]
    InvalidGridSize {
        size: usize,
        e: std::num::TryFromIntError,
    },
    #[error("invalid grid initialization due to unknown letter {letter}")]
    InvalidGridInitializationDueToUnknownLetter { letter: char },
    #[error("missing a mandatory empty cell")]
    MissingAMandatoryEmptyCell {},
}

/// Index of a row or column in the grid.
pub type GridIndex = u8;

/// Size of the grid (width or height).
pub type GridSize = u8;

/// A position in the grid.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: GridIndex,
    pub y: GridIndex,
}

#[wasm_bindgen]
impl Position {
    #[wasm_bindgen(constructor)]
    pub fn new(x: GridIndex, y: GridIndex) -> Position {
        Position { x, y }
    }
}

#[derive(Debug)]
pub(crate) struct MaybePosition {
    pub x: i8,
    pub y: i8,
}

impl MaybePosition {
    pub(crate) fn new(from: &Position, dir: &Direction) -> Self {
        match dir {
            Direction::N => MaybePosition {
                x: from.x as i8,
                y: from.y as i8 - 1,
            },
            Direction::E => MaybePosition {
                x: from.x as i8 + 1,
                y: from.y as i8,
            },
            Direction::S => MaybePosition {
                x: from.x as i8,
                y: from.y as i8 + 1,
            },
            Direction::O => MaybePosition {
                x: from.x as i8 - 1,
                y: from.y as i8,
            },
        }
    }
}

pub(crate) struct Grid {
    width: GridSize,
    height: GridSize,
    cells: HashMap<Position, Cell>,
}

impl Grid {
    /// Creates a new empty grid with the given width and height.
    pub(crate) fn empty(width: GridSize, height: GridSize) -> Self {
        let mut cells = HashMap::new();
        for i in 0..width {
            for j in 0..height {
                cells.insert(Position::new(i, j), Cell::Empty);
            }
        }

        Self {
            width,
            height,
            cells,
        }
    }

    /// Creates a new grid from the given vector of positions and letters.
    pub(crate) fn from_vec(
        width: usize,
        height: usize,
        vec: Vec<(Position, char)>,
    ) -> Result<Self, GridError> {
        let w = u8::try_from(width).map_err(|e| GridError::InvalidGridSize { size: width, e })?;
        let h = u8::try_from(height).map_err(|e| GridError::InvalidGridSize { size: height, e })?;

        let empty = Grid::empty(w, h);

        let filled = vec.into_iter().try_fold(empty, |mut grid, (pos, c)| {
            let index = FRENCH_LETTERS_TABLE.try_get_letter_index(c).map_err(|_e| {
                GridError::InvalidGridInitializationDueToUnknownLetter { letter: c }
            })?;

            grid.update_cell(pos, Cell::Letter(index));
            Ok::<Grid, _>(grid)
        })?;

        Ok(filled)
    }

    pub(crate) fn update_cell(&mut self, pos: Position, value: Cell) {
        self.cells.insert(pos, value);
    }

    pub(crate) fn cell(&self, pos: &Position) -> &Cell {
        self.cells
            .get(pos)
            .expect("unexpected out of grid position")
    }

    /// Check if a position is in the grid.
    pub(crate) fn is_in_grid(&self, pos: MaybePosition) -> Option<Position> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i8 || pos.y >= self.height as i8 {
            return None;
        }

        Some(Position::new(pos.x as u8, pos.y as u8))
    }

    pub(crate) fn cells(&self) -> &HashMap<Position, Cell> {
        &self.cells
    }

    fn empty_cells(&self) -> impl Iterator<Item = &Position> {
        self.cells.iter().filter_map(|(pos, cell)| match cell {
            Cell::Letter(_) => None,
            Cell::Empty => Some(pos),
        })
    }

    fn letters(&self) -> impl Iterator<Item = (&Position, &LetterIndex)> {
        self.cells.iter().filter_map(|(pos, cell)| match cell {
            Cell::Letter(c) => Some((pos, c)),
            Cell::Empty => None,
        })
    }

    pub(crate) fn random_empty_cell_position<R>(&self, rng: &mut R) -> Option<Position>
    where
        R: Rng + ?Sized,
    {
        let mut empty_cells_positions: Vec<&Position> = self.empty_cells().collect();
        // sort empty cells in an arbitrary but idempotent order
        // needed to unsure that using the same rng we get the same empty cell
        empty_cells_positions.sort_by(|&v1, &v2| match v1.x.cmp(&v2.x) {
            std::cmp::Ordering::Equal => v1.y.cmp(&v2.y),
            o => o,
        });

        empty_cells_positions.choose(rng).map(|&pos| *pos)
    }

    pub(crate) fn try_random_empty_cell_position<R>(
        &self,
        rng: &mut R,
    ) -> Result<Position, GridError>
    where
        R: Rng + ?Sized,
    {
        self.random_empty_cell_position(rng)
            .ok_or(GridError::MissingAMandatoryEmptyCell {})
    }
}

/// Find and extract words from the grid.
impl Grid {
    pub fn get_words(
        &self,
        dictionary: &Dictionary,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Vec<Match> {
        let mut all_matches = self.find_words(dictionary, letter_table, golden_word);

        all_matches.sort_by(
            |m1, m2| m2.score.cmp(&m1.score), // bigger score first
        );

        // filter matches that use letters from other matches with higher scores

        let mut matches = Vec::new();
        let mut already_used_letters = HashSet::<Position>::new();

        for m in all_matches {
            if m.positions
                .iter()
                .any(|pos| already_used_letters.contains(pos))
            {
                // letters can only be used by one word
                continue;
            }

            already_used_letters.extend(m.positions.iter());
            matches.push(m);
        }

        matches
    }

    fn find_words(
        &self,
        dictionary: &Dictionary,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Vec<Match> {
        self.letters()
            .fold(Vec::new(), |mut matches, (root_pos, root_letter)| {
                let matches_from_pos = self.find_words_from(
                    *root_pos,
                    *root_letter,
                    dictionary,
                    letter_table,
                    golden_word,
                );

                matches.extend(matches_from_pos);
                matches
            })
    }

    fn find_words_from(
        &self,
        pos: Position,
        letter: LetterIndex,
        dictionary: &Dictionary,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Vec<Match> {
        Direction::all()
            .into_iter()
            .fold(Vec::new(), |mut matches, dir| {
                let matches_in_direction = self.find_words_in_one_direction_from(
                    pos,
                    letter,
                    dir,
                    dictionary,
                    letter_table,
                    golden_word,
                );

                matches.extend(matches_in_direction.into_iter());
                matches
            })
    }

    fn find_words_in_one_direction_from(
        &self,
        pos: Position,
        letter: LetterIndex,
        dir: Direction,
        dictionary: &Dictionary,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Vec<Match> {
        let mut matches = vec![];

        let mut positions = vec![pos];
        let mut letters = vec![letter];

        let mut to_check = MaybePosition::new(&pos, &dir);

        loop {
            match self.is_in_grid(to_check) {
                None => break,
                Some(pos) => match self.cell(&pos) {
                    Cell::Empty => break,
                    Cell::Letter(c) => {
                        positions.push(pos);
                        letters.push(*c);

                        // next position in the given direction
                        to_check = MaybePosition::new(&pos, &dir);
                    }
                },
            }

            if positions.len() == Word::MAX_LENGTH {
                break;
            }
        }

        loop {
            // Not enough letters to form a word in that direction
            if positions.len() < Word::MIN_LENGTH {
                return matches;
            }

            let word = Word::new(letters.clone())
                .expect("word lenght should be okay as it is checked before");

            if dictionary.contains(&word) {
                let score = word
                    .score(letter_table, golden_word)
                    .expect("unable to generate score of a word in the grid");

                matches.push(Match {
                    word,
                    positions,
                    score,
                });

                // We only care about the longuest word because it will be the biggest score
                return matches;
            }

            positions.pop();
            letters.pop();
        }
    }

    pub fn retrieve_words(
        &mut self,
        dictionary: &Dictionary,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Vec<Match> {
        let matches = self.get_words(dictionary, letter_table, golden_word);
        for m in matches.iter() {
            for pos in m.positions.iter() {
                self.cells.insert(*pos, Cell::Empty);
            }
        }

        matches
    }
}

/// Pathing and movements in the grid.
impl Grid {
    pub fn allowed_moving_positions(&self, from: Position) -> HashSet<Position> {
        let mut allowed_positions = HashSet::<Position>::new();
        allowed_positions.insert(from);

        fn check_around(pos: Position, grid: &Grid, registry: &mut HashSet<Position>) {
            for direction in Direction::all() {
                let to_check = MaybePosition::new(&pos, &direction);
                if let Some(to_check) = grid.is_in_grid(to_check) {
                    if registry.contains(&to_check) {
                        continue;
                    }

                    registry.insert(to_check);

                    if let Cell::Empty = grid.cell(&to_check) {
                        check_around(to_check, grid, registry);
                    }
                } else {
                    continue;
                }
            }
        }

        check_around(from, &self, &mut allowed_positions);

        allowed_positions
    }

    pub fn most_direct_path(&self, from: &Position, to: &Position) -> Option<Vec<Position>> {
        if *from == *to {
            return Some(vec![*from]);
        }

        if !self.path_exists(from, to) {
            return None;
        }

        let mut shortest_paths: Vec<Vec<Position>> = vec![];

        // priority is given to shorthest paths
        let mut candidates = PriorityQueue::new();
        candidates.push(vec![*from], Reverse(1));

        while let Some((candidate, _)) = candidates.pop() {
            let current_shortest_path_length = shortest_paths.first().map(|p| p.len());
            let any_candidate_generated_will_be_too_long = match current_shortest_path_length {
                Some(size) => size < candidate.len() + 1,
                None => false,
            };

            if any_candidate_generated_will_be_too_long {
                break; // since candidates are ordered by length, if one is too long, all next ones will also be too long
            }

            // TODO check if expect is fine or not
            let head = candidate
                .last()
                .expect("all candidates must have at least one position, the starting one");

            if head != from && self.cell(&head) != &Cell::Empty {
                continue;
            }

            for direction in Direction::all() {
                let to_check = MaybePosition::new(head, &direction);
                match self.is_in_grid(to_check) {
                    None => continue,
                    Some(pos) => {
                        if pos == *to {
                            let mut valid_path = candidate.clone();
                            valid_path.push(pos);
                            shortest_paths.push(valid_path);
                            continue;
                        }

                        if candidate.contains(&pos) {
                            continue;
                        }

                        let new_candidate = {
                            let mut tmp = candidate.clone();
                            tmp.push(pos);
                            tmp
                        };

                        let priority = Reverse(new_candidate.len());
                        candidates.push(new_candidate, priority);
                    }
                };
            }
        }

        // select one of the shortest_paths that has the less angles
        shortest_paths
            .into_iter()
            .map(|path| {
                let number_of_angles = number_of_angles(&path);
                (path, number_of_angles)
            })
            .min_by(|(_, n1), (_, n2)| n1.cmp(n2))
            .map(|(path, _)| path)
    }

    fn path_exists(&self, from: &Position, to: &Position) -> bool {
        if *from == *to {
            return true;
        }

        let to = *to;

        let mut checked: HashSet<Position> = HashSet::new();
        checked.insert(*from);

        let mut need_to_check_around: Vec<Position> = vec![*from];

        while let Some(check_around) = need_to_check_around.pop() {
            for direction in Direction::all() {
                let to_check = MaybePosition::new(&check_around, &direction);
                let to_check = match self.is_in_grid(to_check) {
                    None => continue,
                    Some(pos) => pos,
                };

                if to_check == to {
                    return true;
                }

                if checked.contains(&to_check) {
                    continue;
                }

                checked.insert(to_check);

                match self.cell(&to_check) {
                    Cell::Letter(_) => continue,
                    Cell::Empty => need_to_check_around.push(to_check),
                }
            }
        }

        return false;
    }

    pub fn move_cell(&mut self, from: Position, to: Position) -> MoveResult {
        if !self.path_exists(&from, &to) {
            return MoveResult::NoPath;
        }

        let from_cell = *self.cell(&from);
        let to_cell = *self.cell(&to);

        self.cells.insert(to, from_cell);
        self.cells.insert(from, to_cell);

        MoveResult::Moved
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Orientation {
    Horizontal,
    Vertical,
}

fn number_of_angles(path: &Vec<Position>) -> usize {
    path.iter()
        .tuple_windows()
        .map(|(p1, p2)| {
            if p1.x == p2.x {
                Orientation::Horizontal
            } else if p1.y == p2.y {
                Orientation::Vertical
            } else {
                panic!("Invalid path")
            }
        })
        .tuple_windows()
        .fold(
            0,
            |count, (o1, o2)| {
                if o1 != o2 { count + 1 } else { count }
            },
        )
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MoveResult {
    Moved,
    NoPath,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    N,
    E,
    S,
    O,
}

impl Direction {
    fn all() -> Vec<Direction> {
        vec![Direction::N, Direction::E, Direction::S, Direction::O]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub word: Word,
    pub score: u16,
    pub positions: Vec<Position>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Cell {
    Letter(LetterIndex),
    Empty,
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

#[macro_export]
macro_rules! grid {
    () => {
        $crate::Grid::from_vec(vec![], 0)
    };
    ( [$( $x:expr ),* ]) => { {
        let vec = vec![$($x),*];
        let len  = vec.len();
        $crate::Grid::from_vec(vec, len)
    } };
    ( [$( $x0:expr ),*] $([$( $x:expr ),*])* ) => {
        {
            let mut vec = Vec::new();

            let width0 = $crate::count!($($x0)*);
            let mut _assert_width0 = [(); $crate::count!($($x0)*)];

            let mut row = 0usize;

            let mut col0 = 0usize;

            $(

                    if $x0 != ' ' {
                        vec.push((Position::new(col0 as u8, row as u8), $x0))
                    }

                    col0 = col0 + 1usize;
                    let _col0 = col0;

            )*

            row = row + 1usize;

            $(
                let _assert_width = [(); $crate::count!($($x)*)];
                _assert_width0 = _assert_width;

                let mut col = 0usize;

                $(
                    if $x != ' ' {
                        vec.push((Position::new(col as u8, row as u8), $x))
                    }

                    col = col + 1usize;
                    let _col = col;
                )*

                row = row + 1usize;
                let _row = row;
            )*


            $crate::grid::Grid::from_vec(width0, col0, vec)
        }
    };
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn grid_macro_works() -> Result<(), GridError> {
        let grid = grid!(
            ['Y', ' ', ' ', 'N']
            [' ', 'E', ' ', ' ']
            ['Y', ' ', 'S', 'O']
        )?;

        assert_eq!(
            grid.cell(&Position::new(0 as u8, 0 as u8)),
            &Cell::Letter(FRENCH_LETTERS_TABLE.try_get_letter_index('Y').unwrap())
        );

        assert_eq!(
            grid.cell(&Position::new(3 as u8, 0 as u8)),
            &Cell::Letter(FRENCH_LETTERS_TABLE.try_get_letter_index('N').unwrap())
        );

        assert_eq!(
            grid.cell(&Position::new(3 as u8, 2 as u8)),
            &Cell::Letter(FRENCH_LETTERS_TABLE.try_get_letter_index('O').unwrap())
        );

        assert_eq!(grid.cell(&Position::new(1 as u8, 0 as u8)), &Cell::Empty);

        assert_eq!(grid.cell(&Position::new(3 as u8, 1 as u8)), &Cell::Empty);

        Ok(())
    }
}
