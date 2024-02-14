use regex::Regex;

#[derive(Debug)]
pub(crate) enum Token {
    Blank,
    HorizontalRule,
    UnorderedList,
    Paragraph(Text),
    Header(u8),
    Text(Text),
}

#[derive(Debug)]
pub(crate) enum Text {
    Regular(String),
    Bold(String),
    Italic(String),
}

impl Text {
    fn new(text: String) -> Self {
        let bold_pattern = Regex::new(r"^(\*\*|--|__).*(\*\*|--|__)$").unwrap();
        let italic_pattern = Regex::new(r"^(\*|-|_).*(\*|-|_)$").unwrap();

        if bold_pattern.is_match(&text) {
            Text::Bold(text)
        } else if italic_pattern.is_match(&text) {
            Text::Italic(text)
        } else {
            Text::Regular(text)
        }
    }
}

pub(crate) struct Tokenizer {
    line: String,
    cursor: usize,
}

impl Tokenizer {
    pub(crate) fn new(line: String) -> Self {
        Self { line, cursor: 0 }
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let mut chars = self.line.chars().skip(self.cursor);
        loop {
            match (chars.next(), self.cursor) {
                (Some('#'), 0) => {
                    let header_pattern = Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap();

                    let caps = match header_pattern.captures(&self.line[self.cursor..]) {
                        Some(caps) => caps,
                        None => {
                            self.cursor += self.line.len();
                            return Some(Token::Paragraph(Text::new(self.line.clone())));
                        }
                    };
                    let level = caps[1].len() as u8;
                    self.cursor += caps[1].len();

                    return Some(Token::Header(level));
                }
                (Some(' ') | Some('\t') | Some('-') | Some('_') | Some('*') | Some('+'), 0) => {
                    if self.line == "---" || self.line == "___" || self.line == "***" {
                        self.cursor += self.line.len();
                        return Some(Token::HorizontalRule);
                    }
                    let list_pattern = Regex::new(r"^\s*(-|\*|\+){1}\s*").unwrap();
                    if list_pattern.is_match(&self.line) {
                        self.cursor += 1;
                        return Some(Token::UnorderedList);
                    }
                }
                (Some(_), _) => {
                    // TODO: consider case when Text does not take the remaining characters
                    let cursor = self.cursor;
                    self.cursor += self.line.len();

                    if cursor == 0 {
                        return Some(Token::Paragraph(Text::new(self.line.clone())));
                    }
                    
                    let content =String::from_iter(chars);
                    return Some(Token::Text(Text::new(content)));

                }
                (None, 0) => {
                    self.cursor += 1;
                    return Some(Token::Blank);
                }
                _ => {
                    return None;
                }
            }
        }
    }
}
