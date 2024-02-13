use regex::Regex;


#[derive(Debug)]
pub(crate) enum Token {
    Blank,
    Paragraph(Text),
    Header(u8, Text),
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
        Self {
            line,
            cursor: 0,
        }
    }

    pub(crate) fn next(&mut self) -> Token {
        let mut chars = self.line.chars().skip(self.cursor);

        loop {
            match chars.next() {
                Some('#') => {
                    let header_pattern = Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap();
                    
                    let caps = match header_pattern.captures(&self.line[self.cursor..]) {
                        Some(caps) => caps,
                        None => return Token::Paragraph(Text::new(self.line.clone())),
                    };

                    let level = caps[1].len() as u8;
                    let text = Text::new(caps[2].to_owned());

                    return Token::Header(level, text)
                }
                Some(_) => {
                    return Token::Paragraph(Text::new(self.line.clone()))
                }
                None => {
                    return Token::Blank
                }
            }
        }

    }
}