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
            bold_pattern: Regex::new(r"(^__.*__)|(^\*\*.*\*\*)").unwrap(),
            italic_pattern: Regex::new(r"(^_.*_)|(^\*.*\*)").unwrap(),
            bold_italic_pattern: Regex::new(r"^_(\*\*.*\*\*)_|^\*(__.*__)\*|^__(\*.*\*)__|^\*\*(_.*_)\*\*").unwrap(),
            strikethrough_pattern: Regex::new(r"^~~.*~~").unwrap(),
        }
    }

    fn parse(&self, text: String) -> (usize, Text) {
        match self.regular_pattern.captures(&text) {
            Some(caps) => {
                let mtch = caps.get(0).unwrap();
                return (mtch.end(), Text::Regular(text[..mtch.end()].to_owned()))
            },
            None => {}
        }
        
        match self.bold_italic_pattern.captures(&text) {
            Some(caps) => {
                let mtch = caps.get(0).unwrap();
                return (mtch.end(), Text::BoldItalic(text[3..mtch.end()-3].to_owned()))
            },
            None => {}
        }

        match self.bold_pattern.captures(&text) {
            Some(caps) => {
                let mtch = caps.get(0).unwrap();
                return (mtch.end(), Text::Bold(text[2..mtch.end()-2].to_owned()))
            },
            None => {}
        }

        match self.italic_pattern.captures(&text) {
            Some(caps) => {
                let mtch = caps.get(0).unwrap();
                return (mtch.end(), Text::Italic(text[1..mtch.end()-1].to_owned()))
            },
            None => {}
        };

        match self.strikethrough_pattern.captures(&text) {
            Some(caps) => {
                let mtch = caps.get(0).unwrap();
                return (mtch.end(), Text::Strikethrough(text[2..mtch.end()-2].to_owned()))
            },
            None => {}
        };
        (text.len(), Text::Regular(text))
    }
}

pub(crate) struct Tokenizer {
    line: String,
    cursor: usize,
    state: State,
    text_parser: TextParser,
}

impl Tokenizer {
    pub(crate) fn new(line: String) -> Self {
        Self {
            line,
            cursor: 0,
            state: State::Start,
            text_parser: TextParser::new()
        }
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let mut chars = self.line.chars();
        loop {
            match (chars.nth(self.cursor), self.state) {
                // H1 ~ H6
                (Some('#'), State::Start) => {
                    self.state = State::Text;
                    let header_pattern = Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap();

                    let caps = match header_pattern.captures(&self.line) {
                        Some(caps) => caps,
                        None => {
                            return Some(Token::Paragraph);
                        }
                    };
                    let level = caps[1].len() as u8;
                    self.cursor += caps[1].len() + 1;

                    return Some(Token::Header(level));
                }
                // Horizontal Rule or Unordered List
                (
                    Some(' ') | Some('\t') | Some('-') | Some('_') | Some('*') | Some('+'),
                    State::Start,
                ) => {
                    if self.line == "---" || self.line == "___" || self.line == "***" {
                        self.state = State::End;
                        return Some(Token::HorizontalRule);
                    }

                    self.state = State::Text;
                    let list_pattern: Regex = Regex::new(r"^\s*(-|\*|\+){1}\s+").unwrap();
                    match list_pattern.captures(&self.line) {
                        Some(caps) => {
                            self.cursor += caps[0].len();
                            return Some(Token::UnorderedList);
                        }
                        None => {
                            return Some(Token::Paragraph);
                        }
                    };
                }
                // Paragraph
                (Some(_), State::Start) => {
                    self.state = State::Text;
                    return Some(Token::Paragraph);
                }
                // Text
                (Some(_), State::Text) => {
                    let content = if self.cursor == 0 {
                        self.line.clone()
                    } else {
                        self.line.get(self.cursor..).unwrap_or_default().to_owned()
                    };

                    let (len, text) = self.text_parser.parse(content);
                    self.cursor += len;
                    return Some(Token::Text(text));
                }
                // Blank line
                (None, State::Start) => {
                    self.state = State::End;
                    return Some(Token::Blank);
                }
                // End of line
                _ => {
                    return None;
                }
            }
        }
    }
}
