use regex::Regex;

#[derive(Debug)]
pub(crate) enum Token {
    Blank,
    HorizontalRule,
    UnorderedList,
    Paragraph,
    Bold,
    Italic,
    Strikethrough,
    CodeBlock(String),
    Header(u8),
    Literal(String),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum State {
    Start,
    Process,
    Text,
    End,
}

pub(crate) struct Tokenizer {
    line: String,
    cursor: usize,
    state: State,
    header_pattern: Regex,
    ulist_pattern: Regex,
}

impl Tokenizer {
    pub(crate) fn new(line: String) -> Self {
        Self {
            line,
            cursor: 0,
            state: State::Start,
            header_pattern: Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap(),
            ulist_pattern: Regex::new(r"^\s*([-*+])\s+").unwrap(),
        }
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let mut literal_start = 0;
        loop {
            let Some(current) = self.line.chars().nth(self.cursor) else {
                let token = match self.state {
                    State::Text => {
                        let literal = self.line[literal_start..self.cursor].to_string();
                        Some(Token::Literal(literal))
                    }
                    State::Start => Some(Token::Blank),
                    _ => None,
                };

                self.state = State::End;
                return token;
            };

            match (current, self.state) {
                ('#', State::Start) => {
                    self.state = State::Process;
                    return Some(self.handle_header());
                }
                (' ' | '\t' | '-' | '_' | '*' | '+', State::Start) => {
                    if let Some(token) = self.handle_horizontal_rule() {
                        self.state = State::End;
                        return Some(token);
                    }

                    self.state = State::Process;
                    if let Some(token) = self.handle_ulist() {
                        return Some(token);
                    }
                }
                (_, State::Start) => {
                    self.state = State::Process;
                    return Some(Token::Paragraph);
                }
                ('_' | '*' | '~', State::Process) => {
                    let one_ahead = self.line.chars().nth(self.cursor + 1);

                    match one_ahead {
                        Some(c) if c == current && c == '~' => {
                            self.cursor += 2;
                            return Some(Token::Strikethrough);
                        }
                        Some(c) if c == current => {
                            self.cursor += 2;
                            return Some(Token::Bold);
                        }
                        _ => {
                            self.cursor += 1;
                            return Some(Token::Italic);
                        }
                    }
                }
                (_, State::Process) => {
                    self.state = State::Text;
                    literal_start = self.cursor;
                }
                (_, State::Text) => {
                    if current == '_' || current == '*' || current == '~' {
                        let literal = self.line[literal_start..self.cursor].to_string();
                        self.state = State::Process;
                        return Some(Token::Literal(literal));
                    }
                    self.cursor += 1;
                }
                (_, _) => {
                    return None;
                }
            }
        }
    }

    fn handle_header(&mut self) -> Token {
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

    fn handle_ulist(&mut self) -> Option<Token> {
        match self.ulist_pattern.captures(&self.line) {
            Some(caps) => {
                self.cursor += caps[0].len();
                Some(Token::UnorderedList)
            }
            None => None,
        }
    }

    fn handle_horizontal_rule(&mut self) -> Option<Token> {
        if self.line != "---" && self.line != "___" && self.line != "***" {
            return None;
        }
        Some(Token::HorizontalRule)
    }
}
