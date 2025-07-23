use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::io::{self, stdout, Write};

// Document being edited
pub struct Document {
    pub lines: Vec<String>,
    pub filename: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> io::Result<Self> {
        let content = std::fs::read_to_string(filename)?;
        let lines = content.lines().map(|s| s.to_string()).collect();
        Ok(Self {
            lines,
            filename: Some(filename.to_string()),
        })
    }

    pub fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
            filename: None,
        }
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = std::fs::File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
    }

    pub fn insert(&mut self, at_x: usize, at_y: usize, c: char) {
        if at_y > self.lines.len() {
            return;
        }
        let line = self.lines.get_mut(at_y).unwrap();
        line.insert(at_x, c);
    }
}

// Editor state
pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    cursor_x: u16,
    cursor_y: u16,
    status_message: String,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let document = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    doc
                } else {
                    Document {
                        lines: vec!["".to_string()],
                        filename: Some(fname),
                    }
                }
            }
            None => Document::default(),
        };

        Self {
            should_quit: false,
            document,
            cursor_x: 0,
            cursor_y: 0,
            status_message: "".to_string(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let result = self.run_event_loop();
        terminal::disable_raw_mode()?;
        result
    }

    fn run_event_loop(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        loop {
            self.refresh_screen(&mut stdout)?;
            if self.process_keypress()? {
                break;
            }
        }
        Ok(())
    }

    fn refresh_screen<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.queue(cursor::Hide)?;
        w.queue(terminal::Clear(ClearType::All))?;
        w.queue(cursor::MoveTo(0, 0))?;

        if !self.should_quit {
            // Draw text
            for (index, line) in self.document.lines.iter().enumerate() {
                w.queue(cursor::MoveTo(0, index as u16))?;
                w.write_all(line.as_bytes())?;
            }

            // Draw status bar
            let status_bar = format!("{} - {} lines | {}", 
                self.document.filename.as_deref().unwrap_or("[No Name]"),
                self.document.lines.len(),
                self.status_message
            );
            let (_cols, rows) = terminal::size()?;
            w.queue(cursor::MoveTo(0, rows - 1))?;
            w.write_all(status_bar.as_bytes())?;

            // Move cursor to its position
            w.queue(cursor::MoveTo(self.cursor_x, self.cursor_y))?;
        }

        w.queue(cursor::Show)?;
        w.flush()
    }

    fn process_keypress(&mut self) -> io::Result<bool> {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('x') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.document.save()?;
                    self.should_quit = true;
                }
                KeyCode::Char('s') if key_event.modifiers == KeyModifiers::CONTROL => {
                    self.document.save()?;
                    self.status_message = "File saved successfully.".to_string();
                }
                KeyCode::Char(c) => {
                    self.document.insert(self.cursor_x as usize, self.cursor_y as usize, c);
                    self.cursor_x += 1;
                    self.status_message = "".to_string();
                }
                KeyCode::Up => {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.cursor_y < self.document.lines.len() as u16 - 1 {
                        self.cursor_y += 1;
                    }
                }
                KeyCode::Left => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                }
                KeyCode::Right => {
                    let line_len = self.document.lines[self.cursor_y as usize].len() as u16;
                    if self.cursor_x < line_len {
                        self.cursor_x += 1;
                    }
                }
                _ => {}
            }
        }
        Ok(self.should_quit)
    }
}
