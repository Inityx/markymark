#![feature(nll, entry_or_default, pattern)]

extern crate rand;

mod markov;

use std::{env, fs, io, time};
use rand::thread_rng;
use markov::Markov;

const SENTENCE_DELIMITERS: &str = ".?!";

fn sentence_end(c: char) -> bool { SENTENCE_DELIMITERS.contains(c) }

fn main() {
    eprintln!("Reading files...");
    let sources: Vec<_> = env::args()
        .skip(1)
        .map(fs::read_to_string)
        .collect::<Result<_,_>>()
        .unwrap();
    
    let mut markov = Markov::with_depth(4);

    eprintln!("Training chain...");
    let begin = time::Instant::now();
    for source in &sources { markov.train_text(source, sentence_end); }
    let duration = begin.elapsed();
    let (contexts, links) = markov.num_contexts_links();
    eprintln!(
        "Trained {} contexts and {} links in {} seconds.\nPress enter for a new sentence.",
        contexts,
        links,
        duration.as_secs() as f64 + (duration.subsec_micros() as f64 / 1_000_000f64)
    );

    let mut rng = thread_rng();
    loop {
        io::stdin().read_line(&mut String::new()).unwrap();
        println!("{}", markov.generate_sentence(&mut rng));
    }
}
