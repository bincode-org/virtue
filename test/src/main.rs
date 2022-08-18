bitflags::bitflags! {
    #[derive(virtue_test_derive::RetHi)]
    pub struct Foo: u8 {
        const A = 1;
        const B = 1;
    }
}

#[derive(virtue_test_derive::RetHi)]
pub struct DefaultGeneric<T = ()> {
    pub t: T,
}

fn main() {
    assert_eq!("hi", Foo::A.hi());
    assert_eq!("hi", Foo::B.hi());
}
