
extern crate getopts;
extern crate nl_foundations;

use std::collections::{HashMap,HashSet};

use getopts::{Options,ParsingStyle};

use nl_foundations::mmap::MappedRegion;
use nl_foundations::word_sequence::{CharClass,Word,WordSequence};
use nl_foundations::bayesian_classification::training::Trainer;
use nl_foundations::bayesian_classification::model::Model;

fn train_model<'a>(text: &'a str) -> Model<String> {
    let mut ws: WordSequence<&str> = WordSequence::new(text, |s| s, |_| true);

    let training_marks: HashSet<Word> = ws.words.iter()
        .filter(|&word| ws.class_of_word[word] == CharClass::Other && ws[*word].contains("+"))
        .cloned()
        .collect();
    let word_map: HashMap<Word,Word> = training_marks.iter()
        .map(|&word| {
            let s = ws[word];
            let s_p = &s[..s.len()-1];
            let word_p = ws.insert_word(s_p, CharClass::Other);
            (word, word_p)
        }).collect();

    let mut trainer = Trainer::new(2);
    trainer.train(&ws.words,
                  |w| ws.class_of_word[w] == CharClass::Other,
                  |w| *word_map.get(w).unwrap_or(w),
                  |w| ws[*w].contains("+")
                  );
    let model: Model<String> = Model::new(&trainer, |&w| ws[w].to_string());
    model
}

fn process_text<'a,'m>(model: &'m Model<String>, text: &'a str) {
    let ws: WordSequence<&str> = WordSequence::new(text, |s| s, |_| true);
    let m: Model<Word> = model.localize(|w: &String| {
        ws.to_word(&w.as_ref()).unwrap_or(ws.len())
    });

    let mut len = 0;
    for (i,&w) in ws.words.iter().enumerate() {
        let word = ws[w];
        len += word.len() + 1;
        if len > 80 {
            len = 0;
            println!("{}", word);
        } else {
            print!("{} ", word);
        }
        if ws.class_of_word[&w] == CharClass::Other {
            let (p,n) = m.log_likelihood(&w, m.context(i, &ws.words));
            print!("({},{}) ", p, n);
            if m.is_instance(&w, m.context(i, &ws.words)) {
                len = 0;
                println!("\n");
            }
        }
    }
    println!("");
}

fn print_usage(program: &str, opts: &Options, short: bool) {
    if short {
        println!("{}", opts.short_usage(program));
    } else {
        let brief = format!("Usage: {} [options] training-file file...", program);
        print!("{}", opts.usage(&brief));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program: String = args[0].clone();

    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
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

    let model = MappedRegion::mmap(&matches.free[0])
        .and_then(|contents| contents.get_str().map(train_model))
        .expect(&format!("cannot read {}", matches.free[0]));
    for file in &matches.free[1..] {
        MappedRegion::mmap(file)
            .and_then(|contents| contents.get_str().map(|t| process_text(&model, t)))
            .expect(&format!("cannot read {}", file));
    }
}
