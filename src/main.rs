use std::collections::HashSet;
use std::fs;
use std::time::{Duration, Instant};

use rand::prelude::*;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal,
    Frame as TerminalFrame,
    layout::{Constraint, Flex, Layout, Rect},
    prelude::*,
    text::Line,
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};

const MAX_LIVES: u8 = 6;
const FILLED_HEART: char = '♥';
const HOLLOW_HEART: char = '♡';

struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    fn new(filename: &str) -> Self {
        let contents = fs::read_to_string(filename)
            .expect("Failed to read file"); // if panic!, .expect runs

        let words = contents
            .lines()
            .filter(|w| !w.is_empty())  // |w| is called a closure in rust
            .map(|w| w.to_string())     // it means for each element in contents, do this
            .collect();

        Self { words }
    }

    fn random(&self) -> &str {
        let mut rng = rand::rng();
        self.words
            .choose(&mut rng)
            .map(|s| s.as_str()) // map the chosen word as an immutable string
            .unwrap()
    }
}

struct Hangman { // lifetimes are a bitch
    word: String,
    guesses: HashSet<char>, // hashset is a hashmap that only stores keys
    lives: u8
}

impl Hangman {
    fn new(word: &str) -> Self {
        Self {
            word: word.to_string(),
            guesses: HashSet::new(),
            lives: 6, // can change this later based on difficulty or smth
        }
    }

    fn guess(&mut self, character: char) {
        if !self.guesses.insert(character) { // if the character has been guessed,
            return;                          // ignore the input
        }

        if !self.word.contains(character) { // if the character isnt in the word,
            self.lives -= 1;                // you lose a life
        }
    }

    fn board(&self) -> String {
        self.word
        .chars()
        .map(|c| {
            if self.guesses.contains(&c) {
                c
            } else {
                '_'
            }
        })
        .collect()
    }

    fn lives(&self) -> String {
        let lost = MAX_LIVES - self.lives;
        let mut s = String::from("Lives: ");
    
        for _ in 0..self.lives {
            s.push(FILLED_HEART);
        }
    
        for _ in 0..lost {
            s.push(HOLLOW_HEART);
        }
    
        s
    }

    fn is_won(&self) -> bool {
        self.board() == self.word
    }
    
    fn is_lost(&self) -> bool {
        self.lives == 0
    }
}

fn app(terminal: &mut DefaultTerminal, hangman: &mut Hangman) -> std::io::Result<()> {
    loop {
        if !handle_key_event(hangman)? {
            break Ok(());
        }

        terminal.draw(|frame| render(frame, hangman))?;

        if hangman.is_lost() {
            // play some you lost message
            // press e to exit or r to replay
            break Ok(());
        }

        if hangman.is_won() {
            // play some you won message
            // press e to exit or r to replay
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, app: &mut Hangman) {
    let [main_area, sub_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(9),
    ])
    .areas(frame.area());

    let guessed_height = (app.guesses.len() / 4 + 3) as u16;
    let [guessed_area] = Layout::vertical([Constraint::Length(guessed_height)]).areas(sub_area);

    let mut guesses: Vec<_> = app.guesses.iter().copied().collect();


    let guessed_text = guesses
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    frame.render_widget(" ", main_area);
    frame.render_widget(
        Paragraph::new(guessed_text)
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow))
            ),
        guessed_area,
    );
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let dict = Dictionary::new("word-list.txt");
    let mut hangman = Hangman::new(dict.random()); // get a random word
    ratatui::run(|terminal| app(terminal, &mut hangman))?;

    Ok(())
}

fn handle_key_event(hangman: &mut Hangman) -> std::io::Result<bool> {
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char(c) => {
                if c.is_ascii_alphabetic() {
                    hangman.guess(c.to_ascii_lowercase());
                }
            }

            KeyCode::Esc => return Ok(false),
            _ => {}
        }
    }
    Ok(true)
}