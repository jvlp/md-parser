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
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum State {
    Start,
    Text,
    End,
}

struct TextParser {
    bold_pattern: Regex,
    italic_pattern: Regex,
}

impl TextParser {
    fn new() -> Self {
        Self {
            bold_pattern: Regex::new(r"^(\*\*|--|__).*(\*\*|--|__)$").unwrap(),
            italic_pattern: Regex::new(r"^(\*|-|_).*(\*|-|_)$").unwrap(),
        }
    }

    fn parse(&self, text: String) -> Text {
        if self.bold_pattern.is_match(&text) {
            Text::Bold(text)
        } else if self.italic_pattern.is_match(&text) {
            Text::Italic(text)
        } else {
            Text::Regular(text)
        }
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
        let mut chars = self.line.chars().skip(self.cursor);
        loop {
            match (chars.next(), self.state) {
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
                    self.cursor += caps[1].len();

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
                            self.cursor += caps[0].len() - 1;
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
                    // TODO: consider case when Text does not take the remaining characters

                    let content = if self.cursor == 0 {
                        self.line.clone()
                    } else {
                        String::from_iter(chars)
                    };
                    self.cursor = self.line.len();
                    let text = self.text_parser.parse(content);
                    return Some(Token::Text(text));
                }
                // Blank line
                (None, State::Start) => {
                    println!("Blank");
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
