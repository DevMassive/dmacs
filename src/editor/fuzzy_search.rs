use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use once_cell::sync::Lazy;

use crate::document::Document;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

#[derive(Default)]
pub struct FuzzySearch {
    pub query: String,
    pub matches: Vec<(String, usize)>,
    pub selected_index: usize,
    pub scroll_offset: usize,
}

impl FuzzySearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn handle_input(
        &mut self,
        key: pancurses::Input,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        document: &Document,
    ) -> bool {
        match key {
            pancurses::Input::Character('\x1b') => {
                self.reset();
                return false; // Exit on Esc
            }
            pancurses::Input::Character('\n') => {
                if let Some((_, line_number)) = self.matches.get(self.selected_index) {
                    *cursor_y = *line_number;
                    *cursor_x = 0;
                }
                self.reset();
                return false; // Exit fuzzy search
            }
            pancurses::Input::KeyBackspace
            | pancurses::Input::KeyDC
            | pancurses::Input::Character('\x7f')
            | pancurses::Input::Character('\x08') => {
                self.query.pop();
                self.update_matches(document);
            }
            pancurses::Input::Character(c) => {
                self.query.push(c);
                self.update_matches(document);
            }
            pancurses::Input::KeyUp => {
                if !self.matches.is_empty() {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.matches.len() - 1;
                    }
                }
            }
            pancurses::Input::KeyDown => {
                if !self.matches.is_empty() {
                    if self.selected_index < self.matches.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                }
            }
            _ => {}
        }
        true // Continue fuzzy search
    }

    pub fn update_matches(&mut self, document: &Document) {
        if self.query.is_empty() {
            self.matches = document
                .lines
                .iter()
                .enumerate()
                .map(|(i, line)| (line.clone(), i))
                .collect();
        } else {
            self.matches = document
                .lines
                .iter()
                .enumerate()
                .filter_map(|(i, line)| {
                    MATCHER
                        .fuzzy_match(line, &self.query)
                        .map(|_score| (line.clone(), i))
                })
                .collect();
        }
        self.selected_index = 0;
    }
}
