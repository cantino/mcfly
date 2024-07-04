use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum InputCommand {
    Insert(char),
    Backspace,
    Delete,
    Move(Move),
}

#[derive(Debug)]
pub enum Move {
    BOL,
    EOL,
    BackwardWord,
    ForwardWord,
    Backward,
    Forward,
    Exact(usize),
}

#[derive(Debug)]
/// `CommandInput` data structure
pub struct CommandInput {
    /// The command itself
    pub command: String,
    /// The current cursor position
    pub cursor: usize,
    /// A cache of the length of command in graphemes (not bytes or chars!)
    pub len: usize,
}

impl fmt::Display for CommandInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.command.fmt(f)
    }
}

impl CommandInput {
    pub fn from<S: Into<String>>(s: S) -> CommandInput {
        let mut input = CommandInput {
            command: s.into(),
            cursor: 0,
            len: 0,
        };
        input.recompute_caches();
        input.cursor = input.len;
        input
    }

    pub fn clear(&mut self) {
        self.command.clear();
        self.recompute_caches();
    }

    pub fn set(&mut self, str: &str) {
        self.command = str.to_string();
        self.recompute_caches();
    }

    pub fn move_cursor(&mut self, direction: Move) {
        let mut tmp: isize = self.cursor as isize;

        match direction {
            Move::Backward => tmp -= 1,
            Move::Exact(i) => tmp = i as isize,
            Move::Forward => tmp += 1,
            Move::BOL => tmp = 0,
            Move::EOL => tmp = self.len as isize,
            Move::ForwardWord => {
                tmp = self.next_word_boundary() as isize;
            }
            Move::BackwardWord => {
                tmp = self.previous_word_boundary() as isize;
            }
        }

        tmp = tmp.clamp(0, self.len as isize);
        self.cursor = tmp as usize;
    }

    pub fn delete(&mut self, cmd: Move) {
        let mut new_command = String::with_capacity(self.command.len());
        let command_copy = self.command.clone();
        let vec = command_copy.grapheme_indices(true);

        match cmd {
            Move::Backward => {
                if self.cursor == 0 {
                    return;
                }
                self.move_cursor(Move::Backward);

                for (count, (_, item)) in vec.enumerate() {
                    if count != self.cursor {
                        new_command.push_str(item);
                    }
                }

                self.command = new_command;
                self.recompute_caches();
            }
            Move::Forward => {
                if self.cursor == self.len {
                    return;
                }

                for (count, (_, item)) in vec.enumerate() {
                    if count != self.cursor {
                        new_command.push_str(item);
                    }
                }

                self.command = new_command;
                self.recompute_caches();
            }
            Move::EOL => {
                if self.cursor == self.len {
                    return;
                }

                for (count, (_, item)) in vec.enumerate() {
                    if count < self.cursor {
                        new_command.push_str(item);
                    }
                }

                self.command = new_command;
                self.recompute_caches();
                self.move_cursor(Move::EOL);
            }
            Move::BOL => {
                if self.cursor == 0 {
                    return;
                }

                for (count, (_, item)) in vec.enumerate() {
                    if count >= self.cursor {
                        new_command.push_str(item);
                    }
                }

                self.command = new_command;
                self.recompute_caches();
                self.move_cursor(Move::BOL);
            }
            Move::ForwardWord => {
                if self.cursor == self.len {
                    return;
                }

                let next_word_boundary = self.next_word_boundary();

                for (count, (_, item)) in vec.enumerate() {
                    if count < self.cursor || count >= next_word_boundary {
                        new_command.push_str(item);
                    }
                }

                self.command = new_command;
                self.recompute_caches();
            }
            Move::BackwardWord => {
                if self.cursor == 0 {
                    return;
                }

                let previous_word_boundary = self.previous_word_boundary();

                let mut removed_characters: usize = 0;

                for (count, (_, item)) in vec.enumerate() {
                    if count < previous_word_boundary || count >= self.cursor {
                        new_command.push_str(item);
                    } else {
                        removed_characters += 1;
                    }
                }

                self.command = new_command;
                self.recompute_caches();
                let new_cursor_pos = self.cursor - removed_characters;
                self.move_cursor(Move::Exact(new_cursor_pos));
            }
            _ => unreachable!(),
        }
    }

    pub fn insert(&mut self, c: char) {
        let mut new_command = String::with_capacity(self.command.len());
        {
            let vec = self.command.graphemes(true);
            let mut pushed = false;
            for (count, item) in vec.enumerate() {
                if count == self.cursor {
                    pushed = true;
                    new_command.push(c);
                }
                new_command.push_str(item);
            }
            if !pushed {
                new_command.push(c);
            }
        }
        self.command = new_command;
        self.recompute_caches();
        self.move_cursor(Move::Forward);
    }

    fn recompute_caches(&mut self) {
        self.len = self.command.graphemes(true).count();
    }

    /// Return the index of the grapheme cluster that represents the end of the previous word before
    /// the cursor.
    fn previous_word_boundary(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }

        let mut word_boundaries = self
            .command
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        word_boundaries.push(self.command.len().to_owned());

        let mut word_index: usize = 0;
        let mut found_word: bool = false;
        let command_copy = self.command.clone();
        let vec = command_copy
            .grapheme_indices(true)
            .enumerate()
            .collect::<Vec<(usize, (usize, &str))>>();

        for &(count, (offset, _)) in vec.iter().rev() {
            if count <= self.cursor {
                if !found_word && (vec[if count >= 1 { count - 1 } else { 0 }].1).1 == " " {
                    continue; // Ignore leading spaces
                } else if found_word {
                    if offset == word_boundaries[word_index] {
                        // We've found the previous word boundary.
                        return count;
                    }
                } else {
                    found_word = true;
                    while word_boundaries[word_index] < offset {
                        word_index += 1;
                    }

                    #[allow(clippy::implicit_saturating_sub)]
                    if word_index > 0 {
                        word_index -= 1;
                    }
                }
            }
        }

        0
    }

    /// Return the index of the grapheme cluster that represents the start of the next word after
    /// the cursor.
    fn next_word_boundary(&self) -> usize {
        let command_copy = self.command.clone();

        let grapheme_indices = command_copy.grapheme_indices(true);

        let mut word_boundaries = self
            .command
            .split_word_bound_indices()
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();

        word_boundaries.push(self.command.len().to_owned());

        let mut next_word_index: usize = 0;
        let mut found_word: bool = false;

        for (count, (offset, item)) in grapheme_indices.enumerate() {
            if count >= self.cursor {
                if !found_word && item == " " {
                    continue; // Ignore leading spaces
                } else if found_word {
                    if offset == word_boundaries[next_word_index] {
                        // We've found the next word boundary.
                        return count;
                    }
                } else {
                    found_word = true;

                    while word_boundaries[next_word_index] <= offset {
                        next_word_index += 1;
                    }
                }
            }
        }

        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::CommandInput;

    #[test]
    fn display_works() {
        let input = CommandInput::from("foo bar baz");
        assert_eq!(format!("{input}"), "foo bar baz");
    }

    #[test]
    fn next_word_boundary_works() {
        let mut input = CommandInput::from("foo bar baz");
        input.cursor = 0;
        assert_eq!(input.next_word_boundary(), 3);

        input.cursor = 3;
        assert_eq!(input.next_word_boundary(), 7);

        input.cursor = 4;
        assert_eq!(input.next_word_boundary(), 7);

        input.cursor = 5;
        assert_eq!(input.next_word_boundary(), 7);

        input.cursor = 6;
        assert_eq!(input.next_word_boundary(), 7);

        input.cursor = 7;
        assert_eq!(input.next_word_boundary(), 11);

        input.cursor = 11;
        assert_eq!(input.next_word_boundary(), 11);

        input.cursor = 12;
        assert_eq!(input.next_word_boundary(), 11);
    }

    #[test]
    fn previous_word_boundary_works() {
        let mut input = CommandInput::from("foo bar baz");
        input.cursor = 0;
        assert_eq!(input.previous_word_boundary(), 0);

        input.cursor = 1;
        assert_eq!(input.previous_word_boundary(), 0);

        input.cursor = 3;
        assert_eq!(input.previous_word_boundary(), 0);

        input.cursor = 4;
        assert_eq!(input.previous_word_boundary(), 0);

        input.cursor = 5;
        assert_eq!(input.previous_word_boundary(), 4);

        input.cursor = 7;
        assert_eq!(input.previous_word_boundary(), 4);

        input.cursor = 8;
        assert_eq!(input.previous_word_boundary(), 4);

        input.cursor = 11;
        assert_eq!(input.previous_word_boundary(), 8);

        input.cursor = 12;
        assert_eq!(input.previous_word_boundary(), 8);
    }
}
