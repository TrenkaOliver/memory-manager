mod manager;
use manager ::Manager;

mod collections;
use crate::collections::MyVec;

fn main() {
    let mut heap = [0u8; 8192];
    let ptr = heap.as_ptr();

    let last = unsafe {
        ptr.add(8192)
    };

    println!("start: {}", ptr as usize);
    println!("end: {}", last as usize);
    let mut manager = Manager::new(&mut heap);

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

    let mut v1 = MyVec::from_slice(&s, &mut manager);

    v1.push(Foo::new(10, 10));

    for (i, f) in v1.into_iter().enumerate() {
        println!("{}.: {:?}", i, f);
    }

    manager.debug_free();
}

#[repr(align(128))]
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

