extern crate getopts;
extern crate nl_foundations;

use std::hash::Hash;

use getopts::{Options,ParsingStyle};

use nl_foundations::mmap::MappedRegion;
use nl_foundations::case_string::AsStr;
use nl_foundations::word_sequence::{FMap,WordSequence,with_case,without_case};

type Context = (String,String,String);
type Contexts = Vec<Context>;

fn kwic(word: &str, window: usize, file: &str, case: bool) -> Contexts {
    MappedRegion::mmap(file).and_then(|contents| {
        contents.get_str().map(|text| {
            if case {
                segments(&WordSequence::new(text), with_case(word), window)
            } else {
                segments(&WordSequence::new_case(text), without_case(word), window)
            }
        })
    }).expect(&format!("cannot read {}", file))
}

fn segments<T: Hash+Eq+AsStr>(ws: &WordSequence<T>, word: T, window: usize) -> Contexts {
    match ws.bmap.get(&word) {
        Some(word) => {
            let lines = ws.words.iter()
                .enumerate()
                .filter_map(|(i,n)| {
                    if n == word {
                        Some(get_segment(i, window, &ws))
                    } else {
                        None
                    }
                }).collect();
            lines
        }
        None => { vec!() }
    }
}

// Find the KWIC segments (left, the word, and right) in the word sequence.
fn get_segment<T:AsStr>(word: usize, window: usize, ws: &WordSequence<T>) -> Context {
    let mut start = (word as isize) - (window as isize);
    start = if start < 0 { 0 } else { start };
    let mut end = word + window + 1;
    end = if end > ws.words.len() { ws.words.len() } else { end };
    let left = join(&ws.words[start as usize..word], &ws.fmap);
    let right = join(&ws.words[word+1..end], &ws.fmap);
    (left, get_word(ws.words[word], &ws.fmap).to_string(), right)
}

// Join the word ids, converted to their words and separated by spaces.
fn join<T:AsStr>(seq: &[usize], dict: &FMap<T>) -> String {
    let mut words = Vec::new();
    for i in seq.iter() {
        words.push( get_word(*i, dict) );
    }
    words.join(" ")
}

// Get a word out of the dictionary (or use a missing word marker).
fn get_word<'s,T:AsStr>(word: usize, dict: &'s FMap<T>) -> &'s str {
    dict.get(&word).map(|v| v.as_str()).unwrap_or("--MISSING--")
}

// Print a KWIC segment: left window (right justified), the word, and right window.
fn print_segments(segments: &[(String,String,String)]) {
    let lmax = segments.iter().map(|&(ref l,_,_)| l.len()).max().unwrap();
    for &(ref l, ref w, ref r) in segments.iter() {
        println!("{0:>1$}  {2}  {3}", l, lmax, w, r);
    }
}

fn print_usage(program: &str, opts: &Options, short: bool) {
    if short {
        println!("{}", opts.short_usage(program));
    } else {
        let brief = format!("Usage: {} [options] word file...", program);
        print!("{}", opts.usage(&brief));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program: String = args[0].clone();

    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
    opts.optflag("c", "case", "use case-sensitive comparisons");
    opts.optflag("h", "help", "print detailed help");
    opts.optopt("w", "window", "context window width", "WIDTH");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            print_usage(&program, &opts, true);
            return;
        }
    };
    if matches.opt_present("h") || matches.free.len() < 2 {
        print_usage(&program, &opts, false);
        return;
    }
    let case = matches.opt_present("c");
    let width = match matches.opt_str("w") {
        Some(width) => {
            match width.parse::<usize>() {
                Ok(width) => width,
                Err(e) => {
                    println!("width must be a number: {}", e);
                    print_usage(&program, &opts, true);
                    return;
                }
            }
        }
        None => {
            8usize
        }
    };
    let word: String = matches.free[0].clone();
    for file in &matches.free[1..] {
        print_segments(&kwic(&word, width, file, case));
    }
}
