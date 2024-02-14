use regex::Regex;

#[derive(Debug)]
pub(crate) enum Token {
    Blank,
    HorizontalRule,
    UnorderedList,
    Paragraph,
    Header(u8),
    Text(Text),
}

// TODO: refactor to work with separate tokens for opening and closing tags
// this will be very useful when dealing with nested tags
#[derive(Debug)]
pub(crate) enum Text {
    Regular(String),
    Bold(String),
    Italic(String),
    BoldItalic(String),
    Strikethrough(String),
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum State {
    Start,
    Text,
    End,
}

struct TextParser {
    regular_pattern: Regex,
    bold_pattern: Regex,
    italic_pattern: Regex,
    bold_italic_pattern: Regex,
    strikethrough_pattern: Regex,
}

impl TextParser {
    fn new() -> Self {
        Self {
            regular_pattern: Regex::new(r"^[^\*_~]+").unwrap(),
            bold_pattern: Regex::new(r"(^__[^_]*__)|(^\*\*[^\*]*\*\*)").unwrap(),
            italic_pattern: Regex::new(r"(^_[^_]*_)|(^\*[^\*]*\*)").unwrap(),
            bold_italic_pattern: Regex::new(
                r"^_(\*\*[^_\*]*\*\*)_|^\*(__[^_\*]*__)\*|^__(\*[^_\*]*\*)__|^\*\*(_[^_\*]*_)\*\*",
            )
            .unwrap(),
            strikethrough_pattern: Regex::new(r"^~~[^~]*~~").unwrap(),
        }
    }

    fn parse(&self, text: String) -> (usize, Text) {
        if let Some(caps) = self.regular_pattern.captures(&text) {
            let mtch = caps.get(0).unwrap();
            let end = mtch.end();
            return (mtch.end(), Text::Regular(text[..end].to_owned()));
        }

        if let Some(caps) = self.bold_italic_pattern.captures(&text) {
            let mtch = caps.get(0).unwrap();
            let end = mtch.end() - 3;
            return (mtch.end(), Text::BoldItalic(text[3..end].to_owned()));
        }

        if let Some(caps) = self.bold_pattern.captures(&text) {
            let mtch = caps.get(0).unwrap();
            let end = mtch.end() - 2;
            return (mtch.end(), Text::Bold(text[2..end].to_owned()));
        }

        if let Some(caps) = self.italic_pattern.captures(&text) {
            let mtch = caps.get(0).unwrap();
            let end = mtch.end() - 1;
            return (mtch.end(), Text::Italic(text[1..end].to_owned()));
        }

        if let Some(caps) = self.strikethrough_pattern.captures(&text) {
            let mtch = caps.get(0).unwrap();
            let end = mtch.end() - 2;
            return (mtch.end(), Text::Strikethrough(text[2..end].to_owned()));
        }

        (text.len(), Text::Regular(text))
    }
}

pub(crate) struct Tokenizer {
    line: String,
    cursor: usize,
    state: State,
    text_parser: TextParser,
    header_pattern: Regex,
    ulist_pattern: Regex,
}

impl Tokenizer {
    pub(crate) fn new(line: String) -> Self {
        Self {
            line,
            cursor: 0,
            state: State::Start,
            text_parser: TextParser::new(),
            header_pattern: Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap(),
            ulist_pattern: Regex::new(r"^\s*(-|\*|\+){1}\s+").unwrap(),
        }
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let mut chars = self.line.chars();
        loop {
            let Some(next_char) = chars.nth(self.cursor) else {
                if self.state != State::Start {
                    return None;
                };
                self.state = State::End;
                return Some(Token::Blank);
            };

            match (next_char, self.state) {
                ('#', State::Start) => return Some(self.handle_header()),
                (' ' | '\t' | '-' | '_' | '*' | '+', State::Start) => {
                    return if let Some(token) = self.handle_horizontal_rule() {
                        Some(token)
                    } else {
                        Some(self.handle_ulist())
                    }
                }
                (_, State::Start) => {
                    self.state = State::Text;
                    return Some(Token::Paragraph);
                }
                (_, State::Text) => return Some(self.handle_text()),
                (_, _) => return None,
            }
        }
    }

    fn handle_header(&mut self) -> Token {
        self.state = State::Text;

        let caps = match self.header_pattern.captures(&self.line) {
            Some(caps) => caps,
            None => {
                return Token::Paragraph;
            }
        };
        let level = caps[1].len() as u8;
        self.cursor += caps[1].len() + 1;

        Token::Header(level)
    }

    fn handle_ulist(&mut self) -> Token {
        self.state = State::Text;
        match self.ulist_pattern.captures(&self.line) {
            Some(caps) => {
                self.cursor += caps[0].len();
                Token::UnorderedList
            }
            None => Token::Paragraph,
        }
    }

    fn handle_horizontal_rule(&mut self) -> Option<Token> {
        if self.line != "---" && self.line != "___" && self.line != "***" {
            return None;
        }
        println!("horizontal rule");
        self.state = State::End;
        Some(Token::HorizontalRule)
    }

    fn handle_text(&mut self) -> Token {
        let content = if self.cursor == 0 {
            self.line.clone()
        } else {
            self.line.get(self.cursor..).unwrap_or_default().to_owned()
        };

        let (len, text) = self.text_parser.parse(content);
        self.cursor += len;
        Token::Text(text)
    }
}
