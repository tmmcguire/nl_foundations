#[macro_export]
macro_rules! unwrap {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => panic!(concat!(stringify!($e), ": unwrap! produced None"))
        }
    }
}

#[macro_export]
macro_rules! delegate {
    ( $fld:ident : ) => {
    };

    ( $fld:ident : $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        fn $fcn ( &self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
    };

    ( $fld:ident : $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        fn $fcn ( &self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : pub $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        pub fn $fcn ( &self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
    };

    ( $fld:ident : pub $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        pub fn $fcn ( &self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        fn $fcn ( &mut self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
    };

    ( $fld:ident : mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        fn $fcn ( &mut self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : pub mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        pub fn $fcn ( &mut self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
    };

    ( $fld:ident : pub mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        pub fn $fcn ( &mut self, $( $a : $at ),* ) -> $r { (self.$fld).$fcn( $( $a ),* ) }
        delegate!($fld : $($rest)*);
    };

}

#[cfg(test)]
mod tests {

    trait A {
        fn quux(&self) -> usize;
    }

    struct Basic {
        some_field: usize,
    }

    impl Basic {
        pub fn new() -> Basic { Basic { some_field: 4} }
        pub fn foo(&self) -> usize { self.some_field - 1 }
        pub fn bar(&self, i: usize) -> f64 { self.some_field as f64 / i as f64 }
        pub fn baz(&mut self) -> usize { self.some_field }
    }

    impl A for Basic {
        fn quux(&self) -> usize { self.some_field }
    }

    mod extended {

        use super::Basic;

        pub struct Extended {
            inner: Basic,
            _red_herring: usize,
        }

        impl Extended {
            pub fn new() -> Extended {
                Extended { inner: Basic::new(), _red_herring: 12 } }

            delegate!{
                inner:
                pub foo() -> usize,
                pub bar(i:usize) -> f64,
                pub mut baz() -> usize
            }

        }

        impl super::A for Extended {
            delegate!{
                inner:
                quux() -> usize
            }
        }

    }
    #[test]
    fn test_fn() {
        let mut e = extended::Extended::new();
        assert_eq!(e.foo(), 3);
        assert_eq!(e.bar(2), 2.0);
        assert_eq!(e.baz(), 4);
        assert_eq!(e.quux(), 4);
    }

}
