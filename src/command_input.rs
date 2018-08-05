use unicode_segmentation::UnicodeSegmentation;
use core::mem;
use std::fmt;

#[derive(Debug)]
pub enum InputCommand {
    CapitalizeWord,
    DowncaseWord,
    Insert(char),
    Backspace,
    Delete,
    Move(Move),
    Overwrite(char),
    TransposeChars,
    TransposeWords,
    Undo,
    UpcaseWord,
}

#[derive(Debug)]
pub enum Move {
    BOL,
    EOL,
    BackwardWord,
    ForwardWord(WordLocation),
    Backward,
    Forward,
}

#[derive(Debug)]
pub enum WordLocation {
    Start,
    BeforeEnd,
    AfterEnd,
}

#[derive(Debug)]
pub struct CommandInput {
    pub command: String,
    pub cursor: usize,
    pub len: usize,
    pub word_boundaries: Vec<usize>
}

impl fmt::Display for CommandInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.command.fmt(f)
    }
}

impl CommandInput {
    pub fn from<S: Into<String>>(s: S) -> CommandInput {
        let mut input = CommandInput { command: s.into(), cursor: 0, len: 0, word_boundaries: Vec::new() };
        input.recompute_caches();
        input.cursor = input.len;
        input
    }

    pub fn clear(&mut self) {
        self.command.clear();
        self.recompute_caches();
    }

    pub fn recompute_caches(&mut self) {
        self.len = self.command.graphemes(true).count();
        self.word_boundaries = self.
            command.
            split_word_bound_indices().
            map(|(i, _)| i).
            collect::<Vec<usize>>();
    }

    pub fn move_cursor(&mut self, direction: Move) {
        let mut tmp: isize = self.cursor as isize;

        match direction {
            Move::Backward => { tmp -= 1 },
            Move::Forward => { tmp += 1 },
            Move::BOL => { tmp = 0 },
            Move::EOL => { tmp = self.len as isize },
//            Move::BackwardWord => {
//                split_word_bounds
//            },
//            Move::ForwardWord(WordLocation),
            _ => {}
        }

        if tmp < 0 {
            tmp = 0;
        } else if tmp > self.len as isize {
            tmp = self.len as isize;
        }
        self.cursor = tmp as usize;
    }
    
    pub fn delete(&mut self, cmd: Move) {
        let mut new_command = String::with_capacity(self.command.len());
        let command_copy = self.command.to_owned();
        let vec = command_copy.graphemes(true);

        match cmd {
            Move::Backward => {
                if self.cursor == 0 {
                    return
                }
                self.move_cursor(Move::Backward);

                for (count, item) in vec.enumerate() {
                    if count != self.cursor {
                        new_command.push_str(item);
                    }
                }

                mem::replace(&mut self.command, new_command);
                self.recompute_caches();
            },
            Move::Forward => {
                if self.cursor == self.len {
                    return
                }

                for (count, item) in vec.enumerate() {
                    if count != self.cursor {
                        new_command.push_str(item);
                    }
                }

                mem::replace(&mut self.command, new_command);
                self.recompute_caches();
            },
            Move::EOL => {
                if self.cursor == self.len {
                    return
                }

                for (count, item) in vec.enumerate() {
                    if count < self.cursor {
                        new_command.push_str(item);
                    }
                }

                mem::replace(&mut self.command, new_command);
                self.recompute_caches();
                self.move_cursor(Move::EOL);
            },
            Move::BOL => {
                if self.cursor == 0 {
                    return
                }

                for (count, item) in vec.enumerate() {
                    if count >= self.cursor {
                        new_command.push_str(item);
                    }
                }

                mem::replace(&mut self.command, new_command);
                self.recompute_caches();
                self.move_cursor(Move::BOL);
            },
            _ => unreachable!()
        }
    }

    pub fn insert(&mut self, c: char) {
        let mut new_command = String::with_capacity(self.command.len());
        {
            let vec = self.command.graphemes(true);
            let mut count = 0;
            let mut pushed = false;
            for item in vec {
                if count == self.cursor {
                    pushed = true;
                    new_command.push(c);
                }
                new_command.push_str(item);
                count += 1;
            }
            if !pushed {
                new_command.push(c);
            }
        }
        mem::replace(&mut self.command, new_command);
        self.recompute_caches();
        self.move_cursor(Move::Forward);
    }
}
