extern crate getopts;
#[macro_use] extern crate nl_foundations;

use std::cmp::Ordering;

use getopts::{Options,ParsingStyle};

use nl_foundations::case_string::CaseStr;
use nl_foundations::mmap::MappedRegion;
use nl_foundations::word_sequence::WordSequence;
use nl_foundations::sample::Sample;

fn t_significant_bigrams(file: &str) {
    MappedRegion::mmap(file)
        .and_then(|contents| contents.get_str().map(process_text) )
        .expect(&format!("cannot read {}", file))
}

// Filter for words containing any alphabetic characters.
fn any_alphabetic(s: &str) -> bool { s.chars().any(|c| c.is_alphabetic()) }

fn process_text<'a>(text: &'a str) {
    let ws: WordSequence<CaseStr<'a>> = WordSequence::new(text, CaseStr::from, any_alphabetic);
    // Collect word samples and bigram samples.
    let word_samples: Sample<usize> = ws.words.iter().cloned().collect();
    let f = ws.words.iter().cloned();
    let s = ws.words.iter().skip(1).cloned();
    let bigram_samples: Sample<(usize,usize)> = f.zip(s).collect();
    // Compute the t-value for each bigram.
    let mut scored: Vec<(f64,(usize,usize))> = compute_t(&word_samples, &bigram_samples);
    // Sort by t-value in decreasing order, assuming f64 is ordered.
    scored.sort_by(|l,r| l.partial_cmp(r).unwrap_or(Ordering::Equal).reverse());
    // Print each bigram in order, along with statistical information.
    for (t,pair) in scored {
        let c_obs = unwrap!( bigram_samples.counts.get(&pair) );
        let c_l = unwrap!( word_samples.counts.get(&pair.0) );
        let c_r = unwrap!( word_samples.counts.get(&pair.1) );
        let first = ws[pair.0].to_string();
        let second = ws[pair.1].to_string();
        println!("{:2.2}\t{:6}\t{:6}\t{:6}\t{} {}", t, c_l, c_r, c_obs, first, second);
    }
}

fn t(mean: f64, variance: f64, size: f64, distribution_mean: f64) -> f64 {
    let n = mean - distribution_mean;
    let d = (variance / size).sqrt();
    n/d
}

fn compute_t(words: &Sample<usize>, bigrams: &Sample<(usize,usize)>)
      -> Vec<(f64,(usize,usize))> {
    bigrams.counts.keys().map(|pair| {
        let p_obs = bigrams.p(&pair);
        let p_indep = words.p(&pair.0) * words.p(&pair.1);
        let t = t(p_obs, p_indep, bigrams.total as f64, p_indep);
        (t, *pair)
    }).collect()
}

fn print_usage(program: &str, opts: &Options, short: bool) {
    if short {
        println!("{}", opts.short_usage(program));
    } else {
        let brief = format!("Usage: {} [options] file...", program);
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
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            print_usage(&program, &opts, true);
            return;
        }
    };
    if matches.opt_present("h") || matches.free.len() < 1 {
        print_usage(&program, &opts, false);
        return;
    }

    for file in &matches.free[0..] {
        t_significant_bigrams(file);
    }
}

//  LocalWords:  bigram
