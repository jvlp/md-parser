use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
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
                ('`', State::Start) => {
                    if self.line.starts_with("```") {
                        let language = self.line.get(3..).unwrap_or_default().trim_start();
                        self.state = State::End;
                        return Some(Token::CodeBlock(language.to_string()));
                    }
                }
                (_, State::Start) => {
                    self.state = State::Process;
                    return Some(Token::Paragraph);
                }
                ('_' | '*' | '~', State::Process) => {
                    return self.handle_text_modifier();
                }
                (_, State::Process) => {
                    self.state = State::Text;
                    literal_start = self.cursor;
                }
                ('_' | '*' | '~', State::Text) => {
                    let literal = self.line[literal_start..self.cursor].to_string();
                    self.state = State::Process;
                    return Some(Token::Literal(literal));
                }
                (_, State::Text) => {
                    self.cursor += 1;
                }
                (_, _) => {
                    return None;
                }
            }
        }
    }

    fn handle_header(&mut self) -> Token {
        let Some(caps) = self.header_pattern.captures(&self.line) else {
            return Token::Paragraph;
        };

        let level = caps[1].len() as u8;
        self.cursor += caps[1].len() + 1;

        Token::Header(level)
    }

    fn handle_text_modifier(&mut self) -> Option<Token> {
        let current = self.line.chars().nth(self.cursor)?;
        let next = self.line.chars().nth(self.cursor + 1).unwrap_or_default();

        self.cursor += 2;
        match (current, next) {
            ('~', '~') => Some(Token::Strikethrough),
            ('*', '*') => Some(Token::Bold),
            ('_', '_') => Some(Token::Bold),
            _ => {
                self.cursor -= 1;
                Some(Token::Italic)
            }
        }
    }

    fn handle_ulist(&mut self) -> Option<Token> {
        let caps = self.ulist_pattern.captures(&self.line)?;
        self.cursor += caps[0].len();
        Some(Token::UnorderedList)
    }

    fn handle_horizontal_rule(&mut self) -> Option<Token> {
        if self.line != "---" && self.line != "___" && self.line != "***" {
            return None;
        }
        Some(Token::HorizontalRule)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header1() {
        let line = "# Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header2() {
        let line = "## Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(2)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header3() {
        let line = "### Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(3)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header4() {
        let line = "#### Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(4)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header5() {
        let line = "##### Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(5)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6() {
        let line = "###### Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header7() {
        let line = "####### Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Paragraph));
        let token = tokenizer.next();
        assert_eq!(
            token,
            Some(Token::Literal("####### Hello World".to_string()))
        );
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn header1_bold_star() {
        let line = "# **Hello World**".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6_bold_underline() {
        let line = "###### __Hello World__".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn header1_italic_star() {
        let line = "# *Hello World*".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6_italic_star() {
        let line = "###### *Hello World*".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn header1_italic_underline() {
        let line = "# _Hello World_".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6_italic_underline() {
        let line = "###### _Hello World_".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn header1_strikethrough() {
        let line = "# ~~Hello World~~".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6_strikethrough() {
        let line = "###### ~~Hello World~~".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn header1_strikethrough_bold_italic() {
        let line = "# ~~**_Hello World_**~~".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(1)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn header6_strikethrough_bold_italic() {
        let line = "###### ~~**_Hello World_**~~".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Header(6)));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn horizontal_rule_underline() {
        let line = "___".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::HorizontalRule));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn horizontal_rule_star() {
        let line = "***".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::HorizontalRule));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn horizontal_rule_dash() {
        let line = "---".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::HorizontalRule));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn ulist_dash() {
        let line = "- Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::UnorderedList));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn ulist_plus() {
        let line = "+ Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::UnorderedList));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn ulist_star() {
        let line = "* Hello World".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::UnorderedList));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
    #[test]
    fn ulist_strikethrough_bold_italic() {
        let line = "* ~~**_Hello World_**~~".to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::UnorderedList));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }

    #[test]
    fn paragraph() {
        let line =
            "Hello World _Italic HW_ Hello World, **Bold HW** blah blah blah ~~Strikethrough HW~~ blah blah blah ~~**_Hello World_**~~"
                .to_string();
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Paragraph));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World ".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Italic HW".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal(" Hello World, ".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Bold HW".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal(" blah blah blah ".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Strikethrough HW".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal(" blah blah blah ".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Literal("Hello World".to_string())));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Italic));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Bold));
        let token = tokenizer.next();
        assert_eq!(token, Some(Token::Strikethrough));
        let token = tokenizer.next();
        assert_eq!(token, None);
    }
}
