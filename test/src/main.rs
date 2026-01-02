use std::fmt::Debug;

bitflags::bitflags! {
    #[derive(virtue_test_derive::RetHi)]
    pub struct Foo: u8 {
        const A = 1;
        const B = 1;
    }
}

#[derive(virtue_test_derive::RetHi)]
pub struct DefaultGeneric<T: Debug = ()> {
    pub t: T,
}

#[derive(virtue_test_derive::RetHi, Default)]
pub struct MyStruct<T>
where
    T: Clone,
{
    pub b: Vec<T>,
}

fn main() {
    assert_eq!("hi", Foo::A.hi());
    assert_eq!("hi", Foo::B.hi());
    assert_eq!("hi", MyStruct::<i32>::default().hi());
}
