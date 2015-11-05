use std::collections::HashMap;

#[derive(PartialEq,Eq,Debug)]
enum CharClass {
    Alphabetic,
    Numeric,
    Whitespace,
    Control,
    Other,
}

fn char_class(ch: char) -> CharClass {
    if      ch.is_alphabetic() { CharClass::Alphabetic }
    else if ch.is_numeric()    { CharClass::Numeric }
    else if ch.is_whitespace() { CharClass::Whitespace }
    else if ch.is_control()    { CharClass::Control }
    else                       { CharClass::Other }
}

#[test]
fn test_char_class() {
    assert_eq!(char_class('a'), CharClass::Alphabetic);
    assert_eq!(char_class('1'), CharClass::Numeric);
    assert_eq!(char_class(' '), CharClass::Whitespace);
    assert_eq!(char_class(';'), CharClass::Other);
}

// ---------------------------

pub type FMap<'w> = HashMap<usize, &'w str>;
pub type BMap<'w> = HashMap<&'w str, usize>;

pub struct WordSequence<'w> {
    pub fmap: FMap<'w>,
    pub bmap: BMap<'w>,
    pub words: Vec<usize>,
}

impl <'s> WordSequence<'s> {
    pub fn new(text: &'s str) -> WordSequence<'s> {
        initialize_word_sequence(text)
    }
}

// ---------------------------

// this is a test; 1, 2, three
// aaaawaawawaaaaownownowaaaaa
// 0   45 7891   111111222
//           0   456789012
//
// [0,4), [5,7), [9,8), [10,14), [14,15)...
fn initialize_word_sequence<'s>(text: &'s str) -> WordSequence<'s> {
    let mut forward = HashMap::new();
    let mut words = Vec::new();
    let mut backward = HashMap::new();
    let mut word_start = 0;
    let mut last_cls = CharClass::Whitespace;
    for (i,ch) in text.char_indices() {
        if i == 0 {
            word_start = i;
            last_cls = char_class(ch);
        } else if last_cls != char_class(ch) {
            if last_cls != CharClass::Whitespace {
                updates(&text[word_start..i], &mut words, &mut forward, &mut backward);
            }
            word_start = i;
            last_cls = char_class(ch);
        }
    }
    if last_cls != CharClass::Whitespace {
        updates(&text[word_start..], &mut words, &mut forward, &mut backward);
    }
    WordSequence {
        fmap: forward,
        bmap: backward,
        words: words,
    }
}

fn updates<'s>(word: &'s str, words: &mut Vec<usize>,
               forward: &mut FMap<'s>, backward: &mut BMap<'s>) {
    let v = backward.entry(word).or_insert_with(|| {
        let n = forward.len();
        forward.insert(n, word);
        n
    });
    words.push(*v);
}

#[test]
fn test_initialize_word_sequence() {
    let ws = initialize_word_sequence("abc 123 ");
    assert_eq!(ws.fmap.get(&0), Some(&"abc"));
    assert_eq!(ws.fmap.get(&1), Some(&"123"));
    assert_eq!(ws.fmap.get(&2), None);
    assert_eq!(ws.words,       vec!(0,1));
}

#[test]
fn test_initialize_word_sequence_2() {
    let ws = initialize_word_sequence("this is a test; 1, 2, three");
    assert_eq!(ws.fmap.get(&0), Some(&"this"));
    assert_eq!(ws.fmap.get(&1), Some(&"is"));
    assert_eq!(ws.fmap.get(&2), Some(&"a"));
    assert_eq!(ws.fmap.get(&3), Some(&"test"));
    assert_eq!(ws.fmap.get(&4), Some(&";"));
    assert_eq!(ws.fmap.get(&5), Some(&"1"));
    assert_eq!(ws.fmap.get(&6), Some(&","));
    assert_eq!(ws.fmap.get(&7), Some(&"2"));
    assert_eq!(ws.fmap.get(&8), Some(&"three"));
    assert_eq!(ws.words,       vec!(0,1,2,3,4,5,6,7,6,8));
}
