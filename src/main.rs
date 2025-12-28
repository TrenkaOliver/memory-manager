mod manager;
use manager ::Manager;

mod collections;
use crate::collections::MyVec;

fn main() {
    let mut heap = [0u8; 8192];
    let mut manager = Manager::new(&mut heap);

    let mut main = MyVec::new(&mut manager);

    let mut sub1 = MyVec::new(&mut manager);
    (0..10).into_iter().for_each(|n| sub1.push(n));

    manager.debug_free();


    let mut sub2 = MyVec::new(&mut manager);
    (10..20).into_iter().for_each(|n| sub2.push(n));

    manager.debug_free();

    println!("ss{}", size_of::<MyVec<'_, i32>>());

    main.push(sub1);
    main.push(sub2);
    main.push(MyVec::new(&mut manager));

    (20..30).into_iter().for_each(|n| main[2].push(n));

    manager.debug_free();


    for (v_idx, v) in main.iter().enumerate() {
        println!("{}. subvec:", v_idx);
        for (e_idx, e) in v.iter().enumerate() {
            println!("{}. element: {}", e_idx, e);
        }
        println!()
    }

    let a = main.into_iter().next().unwrap();
    drop(a);

    manager.debug_free();
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
    b: i64
}

impl Foo {
    fn new(a: usize, b: i64) -> Foo {
        Foo { a, b, c: Deep { a, b } }
    }
}

