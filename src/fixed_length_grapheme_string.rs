use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct FixedLengthGraphemeString {
    pub string: String,
    pub grapheme_length: u16,
    pub max_grapheme_length: u16,
}

impl FixedLengthGraphemeString {
    pub fn empty(max_grapheme_length: u16) -> FixedLengthGraphemeString {
        FixedLengthGraphemeString {
            string: String::new(),
            grapheme_length: 0,
            max_grapheme_length,
        }
    }

    pub fn new<S: Into<String>>(s: S, max_grapheme_length: u16) -> FixedLengthGraphemeString {
        let mut fixed_length_grapheme_string =
            FixedLengthGraphemeString::empty(max_grapheme_length);
        fixed_length_grapheme_string.push_grapheme_str(s);
        fixed_length_grapheme_string
    }

    pub fn push_grapheme_str<S: Into<String>>(&mut self, s: S) {
        for grapheme in s.into().graphemes(true) {
            if self.grapheme_length >= self.max_grapheme_length {
                return;
            }
            self.string.push_str(grapheme);
            self.grapheme_length += 1;
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.string.push_str(s);
    }
}

#[cfg(test)]
mod tests {
    use super::FixedLengthGraphemeString;

    #[test]
    fn length_works() {
        let input = FixedLengthGraphemeString::new("こんにちは世界", 20);
        assert_eq!(input.grapheme_length, 7);
    }

    #[test]
    fn max_length_works() {
        let mut input = FixedLengthGraphemeString::new("こんにちは世界", 5);
        assert_eq!(input.string, "こんにちは");
        input.push_grapheme_str("世界");
        assert_eq!(input.string, "こんにちは");
        input.max_grapheme_length = 7;
        input.push_grapheme_str("世界");
        assert_eq!(input.string, "こんにちは世界");
    }
}
