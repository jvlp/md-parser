use std::iter::repeat;


#[derive(Debug)]
pub(crate) enum Token {
    Paragraph(Text),
    H1(Text),
    H2(Text),
    H3(Text),
    H4(Text),
    H5(Text),
    H6(Text),
}

#[derive(Debug)]
pub(crate) enum Text {
    Regular(String),
    Bold(String),
    Italic(String),
}

impl Text {
    fn new(text: String) -> Self {
        let bold_pattern = regex::Regex::new(r"^(\*\*|--|__).*(\*\*|--|__)$").unwrap();
        let italic_pattern = regex::Regex::new(r"^(\*|-|_).*(\*|-|_)$").unwrap();
        
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
                    let mut count = 1;
                    while let Some('#') = chars.next() {
                        count += 1;
                    }
                    let content: String = chars.collect();
                    return match count {
                        1 => Token::H1(Text::new(content)),
                        2 => Token::H2(Text::new(content)),
                        3 => Token::H3(Text::new(content)),
                        4 => Token::H4(Text::new(content)),
                        5 => Token::H5(Text::new(content)),
                        6 => Token::H6(Text::new(content)),
                        _ => Token::Paragraph(Text::new(repeat("#").take(count).collect::<String>()+ " " + &content))
                    };
                }
                Some(_) => {
                    self.cursor += 1;
                }
                None => {
                    return Token::H1(Text::new("".to_string()));
                }
            }
        }

    }
}