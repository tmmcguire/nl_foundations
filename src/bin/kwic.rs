extern crate getopts;
extern crate nl_foundations;

use std::hash::Hash;
use std::borrow::Borrow;

use getopts::{Options,ParsingStyle};

use nl_foundations::mmap::MappedRegion;
use nl_foundations::case_string::CaseStr;
use nl_foundations::word_sequence::WordSequence;

type Context = (String,String,String);
type Contexts = Vec<Context>;

fn kwic(word: &str, window: usize, file: &str, case: bool) -> Contexts {
    MappedRegion::mmap(file).and_then(|contents| {
        contents.get_str().map(|text| {
            if case {
                segments(&WordSequence::new(text, |s| s, |_| true), word, window)
            } else {
                segments(&WordSequence::new(text, CaseStr::from, |_| true), CaseStr::from(word), window)
            }
        })
    }).expect(&format!("cannot read {}", file))
}

fn segments<T>(ws: &WordSequence<T>, word: T, window: usize) -> Contexts
    where T: Hash+Eq+ToString+Clone+Borrow<str> {
    match ws.to_word(&word) {
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
fn get_segment<T>(word: usize, window: usize, ws: &WordSequence<T>) -> Context
    where T: Eq+Hash+ToString+Clone+Borrow<str> {
    let mut start = (word as isize) - (window as isize);
    start = if start < 0 { 0 } else { start };
    let mut end = word + window + 1;
    end = if end > ws.words.len() { ws.words.len() } else { end };
    // Join the word ids, converted to their words and separated by spaces.
    let left = ws.words[start as usize..word].iter()
        .map(|&w| ws[w].clone())
        .collect::<Vec<_>>()
        .join(" ");
    let right = ws.words[word+1..end].iter()
        .map(|&w| ws[w].clone())
        .collect::<Vec<_>>()
        .join(" ");
    // Get the middle word out of the dictionary.
    (left, ws[ws.words[word]].to_string(), right)
}

// Print a KWIC segment: left window (right justified), the word, and right window.
fn print_segments(segments: &[(String,String,String)]) {
    let lmax = segments.iter().map(|&(ref l,_,_)| l.len()).max().unwrap_or(0);
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
