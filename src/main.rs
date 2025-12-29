mod manager;
mod collections;
use crate::collections::MyVec;


fn main() {

    let s = [
        Foo::new(0, 0),
        Foo::new(1, 1),
        Foo::new(2, 2),
        Foo::new(3, 3),
        Foo::new(4, 4),
        Foo::new(5, 5),
        Foo::new(6, 6),
        Foo::new(7, 7),
        Foo::new(8, 8),
        Foo::new(9, 9),
    ];

    let v1 = MyVec::from_slice(&s);

    for (i, f) in v1.iter().enumerate() {
        println!("{}.: {:?}", i, f);
    }

    let mut v2 = MyVec::from_slice(&s);
    for e in v2.drain(..) {
        println!("drained: {:?}", {e});
    }

    v2.iter().for_each(|e| println!("remaining: {:?}", e));


}


#[derive(Debug)]
struct Foo {
    a: usize,
    b: i64,
    c: Deep,
}

#[derive(Debug)]
struct Deep {
    a: usize,
    b: u128
}

impl Foo {
    fn new(a: usize, b: i64) -> Foo {
        Foo { a, b, c: Deep { a, b: b as u128 } }
    }
}

