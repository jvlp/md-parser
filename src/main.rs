mod tokenizer;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    vec,
};
use tokenizer::Tokenizer;

fn main() {
    let arg = std::env::args().last().unwrap();
    let path = Path::new(&arg);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut tokenizer = Tokenizer::new();

    for line in reader.lines() {
        match line {
            Ok(line) => tokenizer.set_line(&line),
            Err(_) => continue,
        };

        let mut tokens = vec![];
        while let Some(token) = tokenizer.next() {
            tokens.push(token);
            // println!("{:?}", token);
        }
        print!("{:?}\n", tokens);
    }
}
