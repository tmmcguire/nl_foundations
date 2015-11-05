extern crate getopts;
extern crate nl_foundations;

use getopts::{Options,ParsingStyle};

use nl_foundations::mmap::MappedRegion;
use nl_foundations::word_sequence::{FMap,WordSequence};

fn kwic(word: &str, window: usize, file: &str) -> Vec<(String,String,String)> {
    MappedRegion::mmap(file).and_then(|contents| {
        contents.get_str().map(|text| kwic_segments(text, word, window) )
    }).expect(&format!("cannot read {}", file))
}

fn kwic_segments(text: &str, word: &str, window: usize) -> Vec<(String,String,String)> {
    let ws = WordSequence::new(text);
    match ws.bmap.get(word) {
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
fn get_segment(word: usize, window: usize, ws: &WordSequence) -> (String,String,String) {
    let mut start = (word as isize) - (window as isize);
    start = if start < 0 { 0 } else { start };
    let mut end = word + window + 1;
    end = if end > ws.words.len() { ws.words.len() } else { end };
    let left = join(&ws.words[start as usize..word], &ws.fmap);
    let right = join(&ws.words[word+1..end], &ws.fmap);
    (left, get_word(ws.words[word], &ws.fmap).to_string(), right)
}

// Join the word ids, converted to their words and separated by spaces.
fn join(seq: &[usize], dict: &FMap) -> String {
    let mut words = Vec::new();
    for i in seq.iter() {
        words.push( get_word(*i, dict) );
    }
    words.join(" ")
}

// Get a word out of the dictionary (or use a missing word marker).
fn get_word<'s>(word: usize, dict: &FMap<'s>) -> &'s str {
    *dict.get(&word).unwrap_or(&"--MISSING--")
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
        print_segments(&kwic(&word, width, file));
    }
}
