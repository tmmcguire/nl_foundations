use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

#[derive(PartialEq,Eq,Debug)]
pub enum CharClass {
    Alphabetic,
    Numeric,
    Whitespace,
    Control,
    Other,
}

impl CharClass {
    pub fn classify(ch: char) -> CharClass {
        if ch.is_alphabetic() { CharClass::Alphabetic }
        else if ch.is_numeric() { CharClass::Numeric }
        else if ch.is_whitespace() { CharClass::Whitespace }
        else if ch.is_control() { CharClass::Control }
        else { CharClass::Other }
    }
}

#[test]
fn test_char_class() {
    assert_eq!(CharClass::classify('a'), CharClass::Alphabetic);
    assert_eq!(CharClass::classify('1'), CharClass::Numeric);
    assert_eq!(CharClass::classify(' '), CharClass::Whitespace);
    assert_eq!(CharClass::classify(';'), CharClass::Other);
}

// ----------------------------------------

pub type Word = usize;
pub type ToWord<T> = HashMap<T,Word>;
pub type FromWord<T> = HashMap<Word,T>;
pub type ClassMap = HashMap<Word,CharClass>;

#[derive(Debug)]
pub struct WordSequence<T: Eq + Hash + Clone> {
    pub to_word: ToWord<T>,
    pub from_word: FromWord<T>,
    pub class_of_word: ClassMap,
    pub words: Vec<Word>,
}

impl<T: Hash + Eq + Clone> WordSequence<T> {
    pub fn new<'l,F,P>(text: &'l str, to_t: F, is_word: P) -> WordSequence<T>
        where F: Fn(&'l str) -> T,
              P: Fn(&'l str) -> bool {
        initialize_word_sequence(text, to_t, is_word)
    }

    pub fn from_word(&self, word: &Word) -> Option<&T> { self.from_word.get(word) }
    pub fn from_word_default<'l>(&'l self, word: &Word, d: &'l T) -> &'l T {
        self.from_word.get(&word).unwrap_or(d)
    }

    pub fn to_word(&self, t: &T) -> Option<&usize> { self.to_word.get(t) }
}

impl<T: Hash + Eq + Clone> Index<Word> for WordSequence<T> {
    type Output = T;
    fn index(&self, idx: Word) -> &T {
        match self.from_word(&idx) {
            Some(v) => v,
            None => panic!("unknown word index {}", idx),
        }
    }
}

// ----------------------------------------

// this is a test; 1, 2, three
// aaaawaawawaaaaownownowaaaaa
// 0   45 7891   111111222
//           0   456789012
//
// [0,4), [5,7), [8,9), [10,14), [14,15)...
fn initialize_word_sequence<'l,T,F,P>(text: &'l str, trans: F, pred: P) -> WordSequence<T>
    where T: Hash + Eq + Clone,
          F: Fn(&'l str) -> T,
          P: Fn(&'l str) -> bool {
    let mut from_word  = FromWord::new();
    let mut to_word = ToWord::new();
    let mut classes = ClassMap::new();
    let mut words = Vec::new();
    let mut word_start = 0;
    let mut last_cls = CharClass::Whitespace;
    for (i,ch) in text.char_indices() {
        if i == 0 {
            word_start = i;
            last_cls = CharClass::classify(ch);
        } else if last_cls != CharClass::classify(ch) {
            if last_cls != CharClass::Whitespace && pred(&text[word_start..i]) {
                updates(trans(&text[word_start..i]),
                        last_cls,
                        &mut words,
                        &mut from_word,
                        &mut to_word,
                        &mut classes);
            }
            word_start = i;
            last_cls = CharClass::classify(ch);
        }
    }
    if last_cls != CharClass::Whitespace && pred(&text[word_start..]) {
        updates(trans(&text[word_start..]),
                last_cls,
                &mut words,
                &mut from_word,
                &mut to_word,
                &mut classes);
    }
    WordSequence {
        to_word: to_word,
        from_word: from_word,
        class_of_word: classes,
        words: words,
    }
}

fn updates<T: Hash + Eq + Clone>(word: T,
                                 class: CharClass,
                                 words: &mut Vec<usize>,
                                 from_word: &mut FromWord<T>,
                                 to_word: &mut ToWord<T>,
                                 classes: &mut HashMap<usize,CharClass>) {
    let v = to_word.entry(word.clone()).or_insert_with(|| {
        let n = from_word.len();
        from_word.insert(n, word.clone());
        classes.insert(n, class);
        n
    });
    words.push(*v);
}

#[test]
fn test_initialize_word_sequence() {
    let ws = initialize_word_sequence("abc 123 ", |s| s, |_| true);
    assert_eq!(ws.from_word.get(&0), Some(&"abc"));
    assert_eq!(ws.from_word.get(&1), Some(&"123"));
    assert_eq!(ws.from_word.get(&2), None);
    assert_eq!(ws.words,       vec!(0,1));
}

#[test]
fn test_initialize_word_sequence_2() {
    let ws = initialize_word_sequence("this is a test; 1, 2, three", |s| s, |_| true);
    assert_eq!(ws.from_word.get(&0), Some(&"this"));
    assert_eq!(ws.from_word.get(&1), Some(&"is"));
    assert_eq!(ws.from_word.get(&2), Some(&"a"));
    assert_eq!(ws.from_word.get(&3), Some(&"test"));
    assert_eq!(ws.from_word.get(&4), Some(&";"));
    assert_eq!(ws.from_word.get(&5), Some(&"1"));
    assert_eq!(ws.from_word.get(&6), Some(&","));
    assert_eq!(ws.from_word.get(&7), Some(&"2"));
    assert_eq!(ws.from_word.get(&8), Some(&"three"));
    assert_eq!(ws.words,       vec!(0,1,2,3,4,5,6,7,6,8));
}

#[test]
fn test_with_case() {
    let ws = WordSequence::new("This is a test. Is only a test.", |s| s, |_| true);
    assert_eq!(ws.words, vec!(0,1,2,3,4,5,6,2,3,4));
}

#[test]
fn test_without_case() {
    use case_string::CaseStr;
    let ws = WordSequence::new("This is a test. Is only a test.", CaseStr::from, |_| true);
    assert_eq!(ws.words, vec!(0,1,2,3,4,1,5,2,3,4));
}
