use std::cmp::Ordering;
use std::hash::{Hash,Hasher};

#[derive(Debug,Clone)]
pub struct CaseStr<'s>(&'s str);

macro_rules! delegate {
    // 0 additional arguments
    ( $( $f:ident ( ) -> $r:ty ),* ) => {
        $( pub fn $f(&self) -> $r { (self.0).$f() } )*
    };
    // 1+ additional arguments
    ( $( $f:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ),* ) => {
        $( pub fn $f (&self, $( $a: $at),* ) -> $r { (self.0).$f( $( $a ),* ) } )*
    };
}

impl<'s> CaseStr<'s> {
    pub fn from(s: &'s str) -> CaseStr<'s> {
        CaseStr(s)
    }

    delegate!( len() -> usize,
               is_empty() -> bool );
    delegate!( split_at(mid:usize) -> (&str,&str) );
}

#[test]
fn test_case_str_1() {
    let s = CaseStr::from("abc");
    assert_eq!(s.len(), 3);
    assert!(!s.is_empty());
}

impl<'s> ToString for CaseStr<'s> {
    fn to_string(&self) -> String {
        self.0.to_lowercase()
    }
}

impl<'s> PartialEq for CaseStr<'s> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() == other.len() {
            let it_l = self.0.chars().flat_map(|c| c.to_lowercase());
            let it_r = other.0.chars().flat_map(|c| c.to_lowercase());
            for (ch_l,ch_r) in it_l.zip( it_r ) {
                if ch_l != ch_r { return false; }
            }
            return true;
        }
        return false;
    }
}

impl<'s> Eq for CaseStr<'s> { }

#[test]
fn test_case_str_eq() {
    let s = CaseStr::from("abc");
    let t = CaseStr::from("abc");
    let u = CaseStr::from("ABC");
    assert_eq!(s,t);
    assert_eq!(s,u);
}

impl<'s> PartialOrd for CaseStr<'s> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let it_l = self.0.chars().flat_map(|c| c.to_lowercase());
        let it_r = other.0.chars().flat_map(|c| c.to_lowercase());
        for (ch_l,ch_r) in it_l.zip( it_r ) {
            match ch_l.partial_cmp( &ch_r ) {
                Some(Ordering::Equal) => { }
                ord                             => { return ord; }
            }
        }
        return self.len().partial_cmp( &other.len() );
    }
}

#[test]
fn test_case_str_partialord() {
    let s = CaseStr::from("abc");
    let t = CaseStr::from("abc");
    let u = CaseStr::from("ABC");
    let v = CaseStr::from("AB");
    let w = CaseStr::from("uvw");
    assert_eq!(s.partial_cmp(&t), Some(Ordering::Equal));
    assert_eq!(s.partial_cmp(&u), Some(Ordering::Equal));
    assert_eq!(s.partial_cmp(&v), Some(Ordering::Greater));
    assert_eq!(v.partial_cmp(&s), Some(Ordering::Less));
    assert_eq!(s.partial_cmp(&w), Some(Ordering::Less));
    assert_eq!(w.partial_cmp(&s), Some(Ordering::Greater));
}

impl<'s> Ord for CaseStr<'s> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(ord) => ord,
            None      => { panic!("incomparible strings?") }
        }
    }
}

#[test]
fn test_case_str_ord() {
    let s = CaseStr::from("abc");
    let t = CaseStr::from("abc");
    let u = CaseStr::from("ABC");
    let v = CaseStr::from("AB");
    let w = CaseStr::from("uvw");
    assert_eq!(s.cmp(&t), Ordering::Equal);
    assert_eq!(s.cmp(&u), Ordering::Equal);
    assert_eq!(s.cmp(&v), Ordering::Greater);
    assert_eq!(v.cmp(&s), Ordering::Less);
    assert_eq!(s.cmp(&w), Ordering::Less);
    assert_eq!(w.cmp(&s), Ordering::Greater);
}

impl<'s> Hash for CaseStr<'s> {
    fn hash<H:Hasher>(&self, state: &mut H) {
        for ch in self.0.chars().flat_map(|c| c.to_lowercase()) {
            ch.hash(state);
        }
    }
}

#[test]
fn test_case_str_hash() {
    fn do_hash<'s>(s: &CaseStr<'s>) -> u64 {
        use std::hash::SipHasher;
        let mut hasher = SipHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    let s = do_hash(&CaseStr::from("abc"));
    let t = do_hash(&CaseStr::from("abc"));
    let u = do_hash(&CaseStr::from("ABC"));
    assert_eq!(s, t);
    assert_eq!(s, u);
    assert_eq!(t, u);
}

#[test]
fn test_hashmap() {
    use std::collections::HashMap;
    let mut hm: HashMap<CaseStr,usize> = HashMap::new();
    hm.insert(CaseStr::from("one"), 1);
    hm.insert(CaseStr::from("TWO"), 2);
    assert_eq!(hm.get(&CaseStr::from("one")), Some(&1));
    assert_eq!(hm.get(&CaseStr::from("One")), Some(&1));
    assert_eq!(hm.get(&CaseStr::from("two")), Some(&2));
    assert_eq!(hm.get(&CaseStr::from("tWO")), Some(&2));
    assert_eq!(hm.get(&CaseStr::from("owO")), None);
}

// ---------------------------

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for str {
    fn as_str<'s>(&'s self) -> &'s str { self }
}

impl<'s> AsStr for &'s str {
    fn as_str<'t>(&'t self) -> &'t str { self }
}

impl<'s> AsStr for CaseStr<'s> {
    fn as_str<'t>(&'t self) -> &'t str { self.0 }
}
