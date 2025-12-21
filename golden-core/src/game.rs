use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use wasm_bindgen::prelude::*;

use crate::{
    clock::Clock,
    debug,
    grid::{self, Grid, GridSize, MoveResult, Position},
    lexicon::{
        FRENCH_DICTIONARY, FRENCH_LETTERS_TABLE, LETTER_INDEX_NONE, LetterIndex, LettersTable, Word,
    },
};

enum GridStatus {
    NotFull,
    Full,
}

/// The game object used by the UI through WebAssembly.
#[wasm_bindgen]
pub struct Game {
    /// Current state of the game.
    pub state: GameState,
    /// Maximum clock time in milliseconds.
    pub clock_max_ms: u32,
    /// Grid width.
    pub grid_width: GridSize,
    /// Grid height.
    pub grid_height: GridSize,
    clock: Clock,
    grid: Grid,
    rng: ChaCha8Rng,
    golden_word: Word,
    /// Score of the golden word.
    pub golden_word_score: u16,
    /// Current index in the triplets list.
    pub triplets_current_index: u8,
    triplets: Vec<(u8, u8, u8)>,
    /// Current score of the player.
    pub score: u16,
    found_words: Vec<FoundWord>,
    path_from: Option<Position>,
    path_to: Option<Position>,
}

#[wasm_bindgen]
impl Game {
    /// Creates a new game instance.
    ///
    /// # Arguments
    ///
    /// * `clock_ms` - Initial clock time in milliseconds.
    /// * `grid_width` - Width of the grid.
    /// * `grid_height` - Height of the grid.
    /// * `seed` - Seed for random generation.
    #[wasm_bindgen(constructor)]
    pub fn new(clock_ms: u32, grid_width: GridSize, grid_height: GridSize, seed: u32) -> Game {
        debug!("init game");
        debug!("  clock_ms: {}", clock_ms);
        debug!("  grid_width: {}", grid_width);
        debug!("  grid_height: {}", grid_height);
        debug!("  seed: {}", seed);

        let mut rng = rng_from_u32(seed);

        let clock = Clock::new(clock_ms);
        let pool = FRENCH_LETTERS_TABLE.generate_letters_pool(&mut rng);
        let golden_word = FRENCH_DICTIONARY.get_random_six_letter_word(&mut rng);
        let golden_word_score = golden_word
            .score(&FRENCH_LETTERS_TABLE, &golden_word)
            .expect("unable to calculate score of golden word")
            as u16;

        let grid = generate_initial_grid(
            grid_width,
            grid_height,
            pool.initial_grid_selection,
            &mut rng,
        );

        let triplets = pool
            .triplets
            .into_iter()
            .map(|t| (t.0.into(), t.1.into(), t.2.into()))
            .collect();

        Game {
            state: GameState::OnGoing,
            rng,
            clock_max_ms: clock_ms,
            grid_width,
            grid_height,
            clock,
            grid,
            golden_word,
            golden_word_score,
            triplets,
            triplets_current_index: 0,
            score: 0,
            found_words: vec![],
            path_from: None,
            path_to: None,
        }
    }

    /// Advances the game state by the given time delta in milliseconds.
    ///
    /// # Arguments
    ///
    /// * `delta_ms` - Time delta in milliseconds.
    /// * `clicks_on_cell` - List of cell positions (x, y) that were clicked during this tick.
    /// * `last_hovered_cell` - Optional position (x, y) of the last hovered cell.
    pub fn tick(
        &mut self,
        delta_ms: u32,
        clicks_on_cell: Vec<Position>,
        last_hovered_cell: Option<Position>,
    ) -> GameSnapshot {
        if self.state == GameState::Finished {
            return self.generate_game_snapshot();
        }

        self.update_clock(delta_ms);

        if self.clock.remaining_ms() == 0 {
            if let GridStatus::Full = self.place_new_triplets_in_grid() {
                return self.generate_game_snapshot();
            }
        }

        for pos in clicks_on_cell {
            match self.path_from {
                Some(from_pos) => {
                    // Unselect the from position if clicked again
                    if pos == from_pos {
                        self.path_from = None;
                        continue;
                    }

                    let moved = self.grid.move_cell(pos, from_pos);
                    if moved == MoveResult::Moved {
                        self.path_from = None;
                        self.path_to = None;
                        self.remove_found_words_in_grid();
                        if let GridStatus::Full = self.place_new_triplets_in_grid() {
                            return self.generate_game_snapshot();
                        }
                    }
                }
                None => {
                    // Start a new path if the clicked cell contains a letter
                    if let grid::Cell::Letter(_) = self.grid.cell(&pos) {
                        self.path_from = Some(pos);
                    }
                }
            }
        }

        if let Some(_path_from) = self.path_from {
            if let Some(hovered) = last_hovered_cell {
                self.path_to = Some(hovered);
            }
        } else {
            self.path_to = None;
        }

        self.generate_game_snapshot()
    }

    fn place_new_triplets_in_grid(&mut self) -> GridStatus {
        let Some(triplet) = self.pop_triplet() else {
            // TODO handle no more triplets case
            return GridStatus::NotFull;
        };

        if let GridStatus::Full = self.try_place_triplet(triplet) {
            // Grid is full, cannot place triplet
            self.finish_game();
            return GridStatus::Full;
        }

        self.clock.reset();

        GridStatus::NotFull
    }

    /// Attempts to place a triplet on the grid.
    /// Returns true if successful, false if the grid is full.
    fn try_place_triplet(&mut self, (l1, l2, l3): (u8, u8, u8)) -> GridStatus {
        let letters = [
            LetterIndex::from(l1),
            LetterIndex::from(l2),
            LetterIndex::from(l3),
        ];

        for &letter in &letters {
            if !self.try_place_letter(letter) {
                return GridStatus::Full;
            }

            self.remove_found_words_in_grid();
        }

        GridStatus::NotFull
    }

    /// Attempts to place a single letter on the grid.
    /// Returns true if successful, false if the grid is full.
    fn try_place_letter(&mut self, letter: LetterIndex) -> bool {
        self.place_letter_in_random_empty_cell(letter).is_some()
    }

    fn remove_found_words_in_grid(&mut self) {
        let matches =
            self.grid
                .retrieve_words(&FRENCH_DICTIONARY, &FRENCH_LETTERS_TABLE, &self.golden_word);

        // If the current path_from position is part of a found word, unselect it
        if let Some(pos) = self.path_from {
            if matches
                .iter()
                .find(|&m| m.positions.iter().find(|&p| *p == pos).is_some())
                .is_some()
            {
                self.path_from = None;
            }
        }

        let found_words: Vec<FoundWord> = matches
            .iter()
            .map(|m| {
                let score = m
                    .word
                    .score(&FRENCH_LETTERS_TABLE, &self.golden_word)
                    .expect("unable to calculate score of word in the grid")
                    as u16;

                let word: String = m
                    .word
                    .letters()
                    .iter()
                    .map(|&l| {
                        FRENCH_LETTERS_TABLE
                            .try_get_letter(LetterIndex::from(l))
                            .unwrap()
                            .letter
                    })
                    .collect();

                FoundWord { word, score }
            })
            .collect();

        let total_score_found_words: u16 = found_words.iter().map(|w| w.score).sum();

        self.found_words.extend(found_words);
        self.score += total_score_found_words;
    }

    /// Marks the game as finished.
    fn finish_game(&mut self) {
        self.state = GameState::Finished;
    }

    fn generate_game_snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            clock_remaining_ms: self.clock.remaining_ms(),
            grid: self.generate_grid_snapshot(),
            found_words: self.found_words.clone(),
        }
    }

    fn update_clock(&mut self, delta_ms: u32) {
        self.clock.substract(delta_ms);
    }

    fn generate_grid_snapshot(&self) -> Vec<Cell> {
        let (allowed_moving_positions, current_path) = match (self.path_from, self.path_to) {
            (Some(from), Some(to)) => {
                let allowed_moving_positions = self.grid.allowed_moving_positions(from);
                let path = self.grid.most_direct_path(&from, &to);
                (Some(allowed_moving_positions), path)
            }
            (Some(from), None) => {
                let allowed_moving_positions = self.grid.allowed_moving_positions(from);
                (Some(allowed_moving_positions), Some(vec![from]))
            }
            _ => (None, None),
        };

        self.grid
            .cells()
            .iter()
            .map(|(&pos, &c)| {
                let pathing_status = if let Some(ref path) = current_path {
                    if path.contains(&pos) {
                        CellPathingStatus::Path
                    } else {
                        match &allowed_moving_positions {
                            Some(allowed_positions) => {
                                if allowed_positions.contains(&pos) {
                                    CellPathingStatus::Walkable
                                } else {
                                    CellPathingStatus::Blocked
                                }
                            }
                            None => CellPathingStatus::None,
                        }
                    }
                } else {
                    CellPathingStatus::None
                };

                // exchange start and end positions in the current path
                let position = {
                    if let Some(current_path) = current_path.clone() {
                        let start = *current_path.first().unwrap();
                        let end = *current_path.last().unwrap();

                        if pos == start {
                            end
                        } else if pos == end {
                            start
                        } else {
                            pos
                        }
                    } else {
                        pos
                    }
                };

                Cell {
                    position,
                    pathing_status: pathing_status,
                    letter: match c {
                        grid::Cell::Letter(index) => index.into(),
                        grid::Cell::Empty => LETTER_INDEX_NONE.0,
                    },
                }
            })
            .collect()
    }

    fn place_letter_in_random_empty_cell(&mut self, letter_index: LetterIndex) -> Option<Position> {
        if let Some(pos) = self.grid.random_empty_cell_position(&mut self.rng) {
            self.grid.update_cell(pos, grid::Cell::Letter(letter_index));
            return Some(pos);
        }

        None
    }

    /// Gets the letter score for the given letter index in the letters table.
    pub fn get_letter_score(&self, index: u8) -> u32 {
        let index = LetterIndex::try_from(index).expect("Invalid letter index value");
        FRENCH_LETTERS_TABLE.try_get_letter(index).unwrap().score
    }

    /// Gets the letter for the given letter index in the letters table.
    pub fn get_letter(&self, index: u8) -> char {
        let index = LetterIndex::try_from(index).expect("Invalid letter index value");
        FRENCH_LETTERS_TABLE.try_get_letter(index).unwrap().letter
    }

    pub fn get_letters_table(&self) -> LettersTable {
        (*FRENCH_LETTERS_TABLE).clone()
    }

    pub fn golden_word(&self) -> Vec<u8> {
        self.golden_word.letters()
    }

    /// Gets all triplets as a flat vector of letter indices.
    pub fn triplets(&self) -> Vec<u8> {
        self.triplets
            .iter()
            .flat_map(|(a, b, c)| vec![*a, *b, *c])
            .collect()
    }

    fn pop_triplet(&mut self) -> Option<(u8, u8, u8)> {
        if self.triplets_current_index as usize >= self.triplets.len() {
            return None;
        }

        let triplet = self.triplets[self.triplets_current_index as usize];
        self.triplets_current_index += 1;
        Some(triplet)
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum GameState {
    OnGoing,
    Finished,
}

fn generate_initial_grid<R>(
    grid_width: GridSize,
    grid_height: GridSize,
    initial_letters: Vec<LetterIndex>,
    rng: &mut R,
) -> Grid
where
    R: Rng + ?Sized,
{
    let grid = Grid::empty(grid_width, grid_height);

    // place initial letters randomly in the grid
    let grid = initial_letters
        .into_iter()
        .fold(grid, |mut grid, letter_index| {
            let pos = grid
                .try_random_empty_cell_position(rng)
                .expect("Missing a mandatory empty cell during initial grid generation");

            grid.update_cell(pos, grid::Cell::Letter(letter_index));
            grid
        });

    grid
}

fn rng_from_u32(seed: u32) -> ChaCha8Rng {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[0..4].copy_from_slice(&seed.to_le_bytes());
    ChaCha8Rng::from_seed(seed_bytes)
}

/// Snapshot of the game state.
#[wasm_bindgen]
pub struct GameSnapshot {
    /// Remaining clock time in milliseconds.
    pub clock_remaining_ms: u32,
    grid: Vec<Cell>,
    found_words: Vec<FoundWord>,
}

#[wasm_bindgen]
impl GameSnapshot {
    pub fn grid(&self) -> Vec<Cell> {
        self.grid.clone()
    }

    pub fn found_words(&self) -> Vec<FoundWord> {
        self.found_words.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Cell {
    /// Pathing status of the cell.
    pub position: Position,
    /// Pathing status of the cell.
    pub pathing_status: CellPathingStatus,
    /// Letter of the cell.
    pub letter: u8,
}

/// Pathing status of a cell.
#[wasm_bindgen]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum CellPathingStatus {
    None,
    Path,
    Walkable,
    Blocked,
}

#[wasm_bindgen]
pub fn is_empty_cell(cell: u8) -> bool {
    let cell = LetterIndex::from(cell);
    cell == LETTER_INDEX_NONE
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct FoundWord {
    word: String,
    score: u16,
}

#[wasm_bindgen]
impl FoundWord {
    pub fn word(&self) -> String {
        self.word.clone()
    }

    pub fn score(&self) -> u16 {
        self.score
    }
}
