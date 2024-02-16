use regex::Regex;

#[derive(Debug, PartialEq, Eq, Clone)]
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
    CodeBlock,
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
    pub(crate) fn new() -> Self {
        Self {
            line: String::default(),
            cursor: 0,
            state: State::Start,
            header_pattern: Regex::new(r"^(#{1,6})[^#]\s*(.+)$").unwrap(),
            ulist_pattern: Regex::new(r"^\s*([-*+])\s+").unwrap(),
        }
    }

    pub(crate) fn set_line(&mut self, line: &String) {
        println!("line: {:?}", line);
        self.line = line.to_owned();
        self.cursor = 0;
        if self.state != State::CodeBlock {
            self.state = State::Start;
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

                if self.state != State::CodeBlock {
                    self.state = State::End;
                }
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
                        self.state = State::CodeBlock;
                        self.cursor = self.line.len();
                        return Some(Token::CodeBlock(language.to_string()));
                    }
                }
                ('`', State::CodeBlock) => {
                    if self.line.ends_with("```") {
                        self.state = State::End;
                        return Some(Token::CodeBlock("".to_string()));
                    }
                }
                (_, State::Start) => {
                    self.state = State::Process;
                    return Some(Token::Paragraph);
                }
                ('_' | '*' | '~', State::Process) => {
                    return self.handle_text_modifier();
                }
                (_, State::CodeBlock) => {
                    self.cursor = self.line.len();
                    return Some(Token::Literal(self.line.clone()));
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
    use std::collections::VecDeque;

    use super::*;
    const HW: &str = "Hello World";
    const MT: &str = "Hello World _Italic HW_ Hello World, **Bold HW** blah blah blah ~~Strikethrough HW~~ blah blah blah ~~**_Hello World_**~~";

    const L: fn(&str) -> Token = |s| Token::Literal(s.to_string());
    const B: fn(&str) -> [Token; 3] = |s| [Token::Bold, L(s), Token::Bold];
    const I: fn(&str) -> [Token; 3] = |s| [Token::Italic, L(s), Token::Italic];
    const S: fn(&str) -> [Token; 3] = |s| [Token::Strikethrough, L(s), Token::Strikethrough];

    const LV: fn(&str) -> VecDeque<Token> = |s| VecDeque::from([L(s)]);
    const BV: fn(VecDeque<Token>) -> VecDeque<Token> = |t| surround(t, Token::Bold);
    const IV: fn(VecDeque<Token>) -> VecDeque<Token> = |t| surround(t, Token::Italic);
    const SV: fn(VecDeque<Token>) -> VecDeque<Token> = |t| surround(t, Token::Strikethrough);

    const SBIL: fn() -> VecDeque<Token> = || SV(BV(IV(LV(HW))));

    fn expect_multiple_tokens(start_token: Token) -> Vec<Token> {
        let mut tokens = vec![start_token, L("Hello World ")];
        tokens.extend_from_slice(&I("Italic HW"));
        tokens.push(L(" Hello World, "));
        tokens.extend_from_slice(&B("Bold HW"));
        tokens.push(L(" blah blah blah "));
        tokens.extend_from_slice(&S("Strikethrough HW"));
        tokens.push(L(" blah blah blah "));
        build_expect_tokens(tokens, SBIL())
    }

    fn surround(vec: VecDeque<Token>, token: Token) -> VecDeque<Token> {
        let mut vec = vec;
        vec.push_front(token.clone());
        vec.push_back(token);
        vec
    }

    fn build_expect_tokens(tokens: Vec<Token>, new_tokens: VecDeque<Token>) -> Vec<Token> {
        let mut tokens = tokens;
        tokens.extend(new_tokens.into_iter());
        tokens
    }

    fn assert_line(line: &str, expected_tokens: Vec<Token>) {
        let mut tokenizer = Tokenizer::new();
        tokenizer.set_line(&line.to_string());

        for expected_token in expected_tokens {
            assert_eq!(tokenizer.next(), Some(expected_token));
        }

        assert_eq!(tokenizer.next(), None);
    }

    fn assert_block(lines: Vec<&str>, expected_tokens: Vec<Token>) {
        let mut tokenizer = Tokenizer::new();
        let mut tokens = expected_tokens.into_iter();

        for line in lines {
            tokenizer.set_line(&line.to_string());
            while let Some(token) = tokenizer.next() {
                assert_eq!(Some(token), tokens.next());
            }
        }
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn header1() {
        let line = "# Hello World";
        let expected_tokens = vec![Token::Header(1), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header2() {
        let line = "## Hello World";
        let expected_tokens = vec![Token::Header(2), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header3() {
        let line = "### Hello World";
        let expected_tokens = vec![Token::Header(3), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header4() {
        let line = "#### Hello World";
        let expected_tokens = vec![Token::Header(4), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header5() {
        let line = "##### Hello World";
        let expected_tokens = vec![Token::Header(5), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header6() {
        let line = "###### Hello World";
        let expected_tokens = vec![Token::Header(6), L(HW)];
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header7() {
        let line = "####### Hello World";
        let expected_tokens = vec![Token::Paragraph, L("####### Hello World")];
        assert_line(line, expected_tokens);
    }

    #[test]
    fn header1_bold_star() {
        let line = "# **Hello World**";
        let mut expected_tokens = vec![Token::Header(1)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header2_bold_star() {
        let line = "## **Hello World**";
        let mut expected_tokens = vec![Token::Header(2)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header3_bold_star() {
        let line = "### **Hello World**";
        let mut expected_tokens = vec![Token::Header(3)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header4_bold_star() {
        let line = "#### **Hello World**";
        let mut expected_tokens = vec![Token::Header(4)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header5_bold_star() {
        let line = "##### **Hello World**";
        let mut expected_tokens = vec![Token::Header(5)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(line, expected_tokens);
    }
    #[test]
    fn header6_bold_underline() {
        let line = "###### __Hello World__";
        let mut expected_tokens = vec![Token::Header(6)];
        expected_tokens.extend_from_slice(&B(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header1_italic_star() {
        let line = "# *Hello World*";
        let mut expected_tokens = vec![Token::Header(1)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header2_italic_star() {
        let line = "## *Hello World*";
        let mut expected_tokens = vec![Token::Header(2)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header3_italic_star() {
        let line = "### *Hello World*";
        let mut expected_tokens = vec![Token::Header(3)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header4_italic_star() {
        let line = "#### *Hello World*";
        let mut expected_tokens = vec![Token::Header(4)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header5_italic_star() {
        let line = "##### *Hello World*";
        let mut expected_tokens = vec![Token::Header(5)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header6_italic_star() {
        let line = "###### *Hello World*";
        let mut expected_tokens = vec![Token::Header(6)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header1_italic_underline() {
        let line = "# _Hello World_";
        let mut expected_tokens = vec![Token::Header(1)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header2_italic_underline() {
        let line = "## _Hello World_";
        let mut expected_tokens = vec![Token::Header(2)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header3_italic_underline() {
        let line = "### _Hello World_";
        let mut expected_tokens = vec![Token::Header(3)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header4_italic_underline() {
        let line = "#### _Hello World_";
        let mut expected_tokens = vec![Token::Header(4)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header5_italic_underline() {
        let line = "##### _Hello World_";
        let mut expected_tokens = vec![Token::Header(5)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header6_italic_underline() {
        let line = "###### _Hello World_";
        let mut expected_tokens = vec![Token::Header(6)];
        expected_tokens.extend_from_slice(&I(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header1_strikethrough() {
        let line = "# ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(1)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header2_strikethrough() {
        let line = "## ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(2)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header3_strikethrough() {
        let line = "### ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(3)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header4_strikethrough() {
        let line = "#### ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(4)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header5_strikethrough() {
        let line = "##### ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(5)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header6_strikethrough() {
        let line = "###### ~~Hello World~~";
        let mut expected_tokens = vec![Token::Header(6)];
        expected_tokens.extend_from_slice(&S(HW));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header1_strikethrough_bold_italic() {
        let line = "# ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(1)], SBIL());
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header2_strikethrough_bold_italic() {
        let line = "## ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(2)], SBIL());
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header3_strikethrough_bold_italic() {
        let line = "### ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(3)], SBIL());
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header4_strikethrough_bold_italic() {
        let line = "#### ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(4)], SBIL());
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header5_strikethrough_bold_italic() {
        let line = "##### ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(5)], SBIL());
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn header6_strikethrough_bold_italic() {
        let line = "###### ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::Header(6)], SBIL());
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header1_multiple_tokens() {
        let line = "# ".to_owned() + MT;
        let expected_tokens = expect_multiple_tokens(Token::Header(1));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn header6_multiple_tokens() {
        let line = "###### ".to_owned() + MT;

        let expected_tokens = expect_multiple_tokens(Token::Header(6));
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn horizontal_rule_underline() {
        let line = "___";
        let expected_tokens = vec![Token::HorizontalRule];
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn horizontal_rule_star() {
        let line = "***";
        let expected_tokens = vec![Token::HorizontalRule];
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn horizontal_rule_dash() {
        let line = "---";
        let expected_tokens = vec![Token::HorizontalRule];
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn ulist_dash() {
        let line = "- Hello World";
        let expected_tokens = vec![Token::UnorderedList, L(HW)];
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn ulist_plus() {
        let line = "+ Hello World";
        let expected_tokens = vec![Token::UnorderedList, L(HW)];
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn ulist_star() {
        let line = "* Hello World";
        let expected_tokens = vec![Token::UnorderedList, L(HW)];
        assert_line(&line, expected_tokens);
    }
    #[test]
    fn ulist_strikethrough_bold_italic() {
        let line = "* ~~**_Hello World_**~~";
        let expected_tokens = build_expect_tokens(vec![Token::UnorderedList], SBIL());
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn paragraph_multilple_tokens() {
        let expected_tokens = expect_multiple_tokens(Token::Paragraph);
        assert_line(&MT, expected_tokens);
    }
    #[test]
    fn special_characters() {
        let line = "Special characters: & < > \" '";
        let expected_tokens = vec![Token::Paragraph, L("Special characters: & < > \" '")];
        assert_line(&line, expected_tokens);
    }

    #[test]
    fn code_block() {
        let line1 = "```rust";
        let line2 = "fn main() {";
        let line3 = "    println!(\"Hello, world!\");";
        let line4 = "}";
        let line5 = "```";

        let lines = vec![line1, line2, line3, line4, line5];
        let expected_tokens = vec![
            Token::CodeBlock("rust".to_string()),
            Token::Literal("fn main() {".to_string()),
            Token::Literal("    println!(\"Hello, world!\");".to_string()),
            Token::Literal("}".to_string()),
            Token::CodeBlock("".to_string()),
        ];
        assert_block(lines, expected_tokens);
    }
}
