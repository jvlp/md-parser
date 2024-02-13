mod tokenizer;
use std::{fs::File, io::{BufRead, BufReader}, path::Path};
use tokenizer::Tokenizer;
fn main() {
    let arg  = std::env::args().last().unwrap();
    let path = Path::new(&arg);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };
        let mut tokenizer = Tokenizer::new(line);
        let token = tokenizer.next();
        println!("{:?}", token);
    }
}

