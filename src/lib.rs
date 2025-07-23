use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers, EventStream},
    terminal::{self, ClearType},
    QueueableCommand,
};
use futures_util::stream::StreamExt;
use std::fs::File;
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
        if at_y == self.lines.len() {
            self.lines.push(String::new());
        }
        let line = self.lines.get_mut(at_y).unwrap();
        if at_x > line.len() {
            line.push(c);
        } else {
            line.insert(at_x, c);
        }
    }

    pub fn delete(&mut self, at_x: usize, at_y: usize) {
        if at_y >= self.lines.len() {
            return;
        }
        let line = self.lines.get_mut(at_y).unwrap();
        if at_x >= line.len() {
            return;
        }
        line.remove(at_x);
    }

    pub fn insert_newline(&mut self, at_x: usize, at_y: usize) {
        if at_y > self.lines.len() {
            return;
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
            return;
        }
        let current_line = self.lines.get_mut(at_y).unwrap();
        let new_line = current_line.split_off(at_x);
        self.lines.insert(at_y + 1, new_line);
    }
}

// Editor state
pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    cursor_x: u16,
    cursor_y: u16,
    desired_cursor_x: u16,
    status_message: String,
    tty: File,
}

impl Editor {
    pub fn new(filename: Option<String>, tty: File) -> Self {
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
            desired_cursor_x: 0,
            status_message: "".to_string(),
            tty,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let result = futures::executor::block_on(self.run_event_loop());
        terminal::disable_raw_mode()?;
        result
    }

    async fn run_event_loop(&mut self) -> io::Result<()> {
        let mut event_stream = EventStream::new();
        loop {
            self.refresh_screen(&mut stdout())?;
            if self.process_keypress(&mut event_stream).await? {
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

    async fn process_keypress(&mut self, event_stream: &mut EventStream) -> io::Result<bool> {
        if let Some(Ok(Event::Key(key_event))) = event_stream.next().await {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                    self.document.save()?;
                    self.should_quit = true;
                }
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    self.document.save()?;
                    self.status_message = "File saved successfully.".to_string();
                }
                (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                    self.cursor_x = 0;
                    self.desired_cursor_x = 0;
                }
                (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                    let y = self.cursor_y as usize;
                    self.cursor_x = self.document.lines[y].len() as u16;
                    self.desired_cursor_x = self.cursor_x;
                }
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    let y = self.cursor_y as usize;
                    let x = self.cursor_x as usize;
                    let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
                    if x < line_len {
                        self.document.delete(x, y);
                    } else if y < self.document.lines.len() - 1 {
                        let next_line = self.document.lines.remove(y + 1);
                        self.document.lines[y].push_str(&next_line);
                    }
                }
                (KeyCode::Char(c), _) => {
                    self.document.insert(self.cursor_x as usize, self.cursor_y as usize, c);
                    self.cursor_x += 1;
                    self.desired_cursor_x = self.cursor_x;
                    self.status_message = "".to_string();
                }
                (KeyCode::Backspace, _) => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                        self.document.delete(self.cursor_x as usize, self.cursor_y as usize);
                        self.desired_cursor_x = self.cursor_x;
                    } else if self.cursor_y > 0 {
                        let prev_line_len = self.document.lines[self.cursor_y as usize - 1].len();
                        let current_line = self.document.lines.remove(self.cursor_y as usize);
                        self.document.lines[self.cursor_y as usize - 1].push_str(&current_line);
                        self.cursor_y -= 1;
                        self.cursor_x = prev_line_len as u16;
                        self.desired_cursor_x = self.cursor_x;
                    }
                }
                (KeyCode::Enter, _) => {
                    self.document.insert_newline(self.cursor_x as usize, self.cursor_y as usize);
                    self.cursor_y += 1;
                    self.cursor_x = 0;
                    self.desired_cursor_x = 0;
                }
                (KeyCode::Up, _) => {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                }
                (KeyCode::Down, _) => {
                    if self.cursor_y < self.document.lines.len() as u16 - 1 {
                        self.cursor_y += 1;
                    }
                }
                (KeyCode::Left, _) => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                        self.desired_cursor_x = self.cursor_x;
                    }
                }
                (KeyCode::Right, _) => {
                    let line_len = self.document.lines[self.cursor_y as usize].len() as u16;
                    if self.cursor_x < line_len {
                        self.cursor_x += 1;
                        self.desired_cursor_x = self.cursor_x;
                    }
                }
                _ => {}
            }
            // Clamp cursor_x to the end of the line after every keypress
            let y = self.cursor_y as usize;
            if y < self.document.lines.len() {
                let line_len = self.document.lines[y].len() as u16;
                self.cursor_x = self.desired_cursor_x.min(line_len);
            }
        }
        Ok(self.should_quit)
    }
}