#![warn(missing_docs, clippy::all, clippy::pedantic, clippy::cargo)]
#![deny(intra_doc_link_resolution_failure, missing_debug_implementations)]

use dispose::{Disposable, Dispose, DisposeWith};

#[derive(Dispose)]
struct MyUnit;

impl DisposeWith<i32> for MyUnit {
    fn dispose_with(self, with: i32) {
        println!("disposing with {:?}", with);
    }
}

#[derive(Dispose)]
struct MyRecord {
    #[dispose(with = self.x + 1)]
    a: MyUnit,
    #[dispose(ignore)]
    x: i32,
}

#[derive(Dispose)]
struct MyTuple(#[dispose(with = .1)] MyUnit, #[dispose(ignore)] i32);

#[derive(Dispose)]
enum MyEnum {
    Unit,
    Record { #[dispose(with = self.x.pow(3))] a: MyUnit, #[dispose(ignore)] x: i32 },
    Tuple(#[dispose(with = .1)] MyUnit, #[dispose(ignore)] i32),
}

fn main() {
    let x = Disposable::new(MyRecord { a: MyUnit, x: 12 });
    let frick = MyUnit;

    let a = Disposable::new(MyEnum::Unit);
    let b = Disposable::new(MyEnum::Record { a: MyUnit, x: 2 });
    let c = Disposable::new(MyEnum::Tuple(MyUnit, 27));

    frick.dispose();

    println!("Hello, world!");
}
