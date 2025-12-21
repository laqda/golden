use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use lazy_static::lazy_static;
use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};
use wasm_bindgen::prelude::*;

const NUMBER_OF_LETTERS_IN_INITIAL_GRID: usize = 8;
const NUMBER_OF_LETTERS_TRIPLETS: usize = 64;
const NUMBER_OF_LETTERS: usize = NUMBER_OF_LETTERS_IN_INITIAL_GRID + NUMBER_OF_LETTERS_TRIPLETS * 3;

#[derive(Debug, thiserror::Error, PartialEq)]
pub(crate) enum LexiconError {
    #[error("unknown letter '{char}'")]
    UnknownLetter { char: char },
    #[error("unknown letter index '{i}'")]
    UnknownLetterIndex { i: LetterIndex },
    #[error("invalid word length '{len}'")]
    InvalidWordLength { len: usize },
    #[error("letter index out of bounds '{index}'")]
    LetterIndexOutOfBounds { index: u8 },
    #[error(
        "unexpected number of letters in letters table: {number_of_letters}, expected {NUMBER_OF_LETTERS}"
    )]
    UnexpectedNumberOfLettersInLettersTable { number_of_letters: usize },
    #[error("missing score multiplier during score evaluation of the word of length {len}")]
    MissingScoreMultiplier { len: usize },
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LetterIndex(pub(crate) u8);

/// Byte representation of an empty cell.
pub(crate) const LETTER_INDEX_NONE: LetterIndex = LetterIndex(255);

impl From<u8> for LetterIndex {
    fn from(value: u8) -> Self {
        LetterIndex(value)
    }
}

impl Into<u8> for LetterIndex {
    fn into(self) -> u8 {
        self.0
    }
}

impl std::fmt::Debug for LetterIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for LetterIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Representation of a word in the game.
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Word {
    letters: Vec<LetterIndex>,
}

#[wasm_bindgen]
impl Word {
    pub fn letters(&self) -> Vec<u8> {
        self.letters.iter().map(|li| (*li).into()).collect()
    }

    pub fn length(&self) -> usize {
        self.letters.len()
    }
}

impl Word {
    pub const MIN_LENGTH: usize = 5;
    pub const MAX_LENGTH: usize = 8;

    pub(crate) fn new(letters: Vec<LetterIndex>) -> Result<Word, LexiconError> {
        if letters.len() < Self::MIN_LENGTH || letters.len() > Self::MAX_LENGTH {
            return Err(LexiconError::InvalidWordLength { len: letters.len() });
        }

        Ok(Word { letters })
    }

    pub(crate) fn score(
        &self,
        letter_table: &LettersTable,
        golden_word: &Word,
    ) -> Result<u16, LexiconError> {
        let letters_score: Result<u16, LexiconError> = self
            .letters
            .iter()
            .map(|index| Ok(letter_table.try_get_letter(*index)?.score as u16))
            .sum();

        let multiplier: u16 = match self.length() {
            5 => 1,
            6 => 2,
            7 => 3,
            8 => 4,
            _ => {
                return Err(LexiconError::MissingScoreMultiplier { len: self.length() });
            }
        };

        let letters_score = letters_score? * multiplier;

        if self == golden_word {
            return Ok(100 + letters_score);
        };

        Ok(letters_score)
    }
}

/// Index of letters used in the game.
#[wasm_bindgen]
#[derive(Clone)]
pub struct LettersTable {
    letters: Vec<LetterConfig>,
    letter_index_by_char: HashMap<char, LetterIndex>,
}

impl LettersTable {
    pub(crate) fn new(letters: Vec<LetterConfig>) -> Result<LettersTable, LexiconError> {
        // check that the letters table contains the expected number of letters
        let repartition_sum: usize = letters.iter().map(|lc| lc.repartition).sum();
        if repartition_sum != NUMBER_OF_LETTERS_IN_INITIAL_GRID + NUMBER_OF_LETTERS_TRIPLETS * 3 {
            return Err(LexiconError::UnexpectedNumberOfLettersInLettersTable {
                number_of_letters: repartition_sum,
            });
        }

        let mut letter_index_by_char = HashMap::new();

        for (index, letter_config) in letters.iter().enumerate() {
            letter_index_by_char.insert(letter_config.letter, LetterIndex::from(index as u8));
        }

        Ok(LettersTable {
            letters,
            letter_index_by_char,
        })
    }

    pub(crate) fn try_get_letter(&self, index: LetterIndex) -> Result<&LetterConfig, LexiconError> {
        self.letters
            .get(index.0 as usize)
            .ok_or(LexiconError::UnknownLetterIndex { i: index })
    }

    pub(crate) fn try_get_letter_index(&self, char: char) -> Result<LetterIndex, LexiconError> {
        self.letter_index_by_char
            .get(&char)
            .copied()
            .ok_or(LexiconError::UnknownLetter { char })
    }

    pub(crate) fn parse_word(&self, word_str: &str) -> Result<Word, LexiconError> {
        let mut letters = Vec::with_capacity(word_str.len());
        for c in word_str.chars() {
            let index = self.try_get_letter_index(c)?;
            letters.push(index);
        }

        Word::new(letters)
    }

    pub(crate) fn score_word(&self, word: &Word) -> Result<u32, LexiconError> {
        word.letters
            .iter()
            .map(|&index| {
                self.try_get_letter(index)
                    .map(|letter_config| letter_config.score)
            })
            .sum()
    }

    pub(crate) fn generate_letters_pool<R>(&self, rng: &mut R) -> LettersPool
    where
        R: Rng + ?Sized,
    {
        let mut all_letters: Vec<LetterIndex> = self
            .letters
            .iter()
            .enumerate()
            .flat_map(|(index, lc)| {
                std::iter::repeat(LetterIndex::try_from(index as u8).unwrap()).take(lc.repartition)
            })
            .collect();

        all_letters.shuffle(rng);

        let initial_grid_selection = all_letters
            .drain(0..NUMBER_OF_LETTERS_IN_INITIAL_GRID)
            .collect();

        let triplets = all_letters
            .chunks(3)
            .take(NUMBER_OF_LETTERS_TRIPLETS)
            .map(|chunk| (chunk[0], chunk[1], chunk[2]))
            .collect();

        LettersPool {
            initial_grid_selection,
            triplets,
        }
    }
}

#[wasm_bindgen]
impl LettersTable {
    pub fn try_get_letter_score(&self, index: u8) -> u32 {
        let i = LetterIndex::try_from(index).unwrap();
        let lc = self.try_get_letter(i).unwrap();
        lc.score
    }

    pub fn try_get_letter_char(&self, index: u8) -> char {
        let i = LetterIndex::try_from(index).unwrap();
        let lc = self.try_get_letter(i).unwrap();
        lc.letter
    }
}

/// Index of letters used in the game.
#[derive(Clone)]
pub struct LetterConfig {
    pub letter: char,
    pub repartition: usize,
    pub score: u32,
}

lazy_static! {
    pub static ref FRENCH_LETTERS_TABLE: LettersTable = {
        let letters = vec![
            LetterConfig {
                letter: 'A',
                repartition: 23,
                score: 1,
            },
            LetterConfig {
                letter: 'B',
                repartition: 4,
                score: 7,
            },
            LetterConfig {
                letter: 'C',
                repartition: 7,
                score: 5,
            },
            LetterConfig {
                letter: 'D',
                repartition: 5,
                score: 6,
            },
            LetterConfig {
                letter: 'E',
                repartition: 30,
                score: 1,
            },
            LetterConfig {
                letter: 'F',
                repartition: 3,
                score: 8,
            },
            LetterConfig {
                letter: 'G',
                repartition: 4,
                score: 7,
            },
            LetterConfig {
                letter: 'H',
                repartition: 3,
                score: 8,
            },
            LetterConfig {
                letter: 'I',
                repartition: 16,
                score: 2,
            },
            LetterConfig {
                letter: 'J',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'K',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'L',
                repartition: 9,
                score: 4,
            },
            LetterConfig {
                letter: 'M',
                repartition: 5,
                score: 6,
            },
            LetterConfig {
                letter: 'N',
                repartition: 10,
                score: 3,
            },
            LetterConfig {
                letter: 'O',
                repartition: 11,
                score: 3,
            },
            LetterConfig {
                letter: 'P',
                repartition: 5,
                score: 6,
            },
            LetterConfig {
                letter: 'Q',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'R',
                repartition: 15,
                score: 2,
            },
            LetterConfig {
                letter: 'S',
                repartition: 17,
                score: 2,
            },
            LetterConfig {
                letter: 'T',
                repartition: 13,
                score: 3,
            },
            LetterConfig {
                letter: 'U',
                repartition: 9,
                score: 4,
            },
            LetterConfig {
                letter: 'V',
                repartition: 3,
                score: 8,
            },
            LetterConfig {
                letter: 'W',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'X',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'Y',
                repartition: 1,
                score: 9,
            },
            LetterConfig {
                letter: 'Z',
                repartition: 2,
                score: 9,
            },
        ];

        LettersTable::new(letters).expect("Unable to generate letters table for french")
    };
}

#[derive(Debug, Clone)]
pub(crate) struct LettersPool {
    pub initial_grid_selection: Vec<LetterIndex>,
    pub triplets: Vec<(LetterIndex, LetterIndex, LetterIndex)>,
}

/// Dictionary for word validation using LetterIndex representation
pub struct Dictionary {
    words: HashSet<Word>,
    six_letter_words: Vec<Word>,
}

impl Dictionary {
    /// Create a new dictionary from a word list and letter table
    pub fn new(wordlist_content: &str, letters_table: &LettersTable) -> Self {
        let mut words: HashSet<Word> = HashSet::new();
        let mut six_letter_words: Vec<Word> = Vec::new();

        for line in wordlist_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // skip words that are not parsable
            // for instance, words with a length outside of the allowed range
            if let Some(word) = letters_table.parse_word(trimmed).ok() {
                if word.length() == 6 {
                    six_letter_words.push(word.clone());
                }

                words.insert(word);
            }
        }

        Dictionary {
            words,
            six_letter_words,
        }
    }

    /// Check if a Word exists in the dictionary
    pub fn contains(&self, word: &Word) -> bool {
        self.words.contains(&word)
    }

    pub fn get_random_six_letter_word<R>(&self, rng: &mut R) -> Word
    where
        R: Rng + ?Sized,
    {
        self.six_letter_words
            .choose(rng)
            .expect("No six letter words available")
            .clone()
    }
}

lazy_static! {
    pub static ref FRENCH_DICTIONARY: Dictionary = {
        const FRENCH_WORDLIST: &str = include_str!("../wordlists/french1.txt");
        Dictionary::new(FRENCH_WORDLIST, &FRENCH_LETTERS_TABLE)
    };
}
