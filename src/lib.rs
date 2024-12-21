#![feature(specialization)]

use std::marker::PhantomData;

// bool
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TTrue;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TFalse;

pub trait TBool {
    type Then<T>: Optional<Value = T>;

    fn as_bool() -> bool;

    type Or<RHS: TBool>: TBool;
    type And<RHS: TBool>: TBool;
}

impl TBool for TTrue {
    type Then<T> = Value<T>;

    fn as_bool() -> bool {
        true
    }

    type Or<RHS: TBool> = TTrue;
    type And<RHS: TBool> = RHS;
}

impl TBool for TFalse {
    type Then<T> = Null<T>;

    fn as_bool() -> bool {
        false
    }

    type Or<RHS: TBool> = RHS;
    type And<RHS: TBool> = TFalse;
}

pub trait TEqual<T> {
    type Output: TBool;
}

impl<T, U> TEqual<T> for U {
    default type Output = TFalse;
}

impl<T> TEqual<T> for T {
    type Output = TTrue;
}

// Value
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null<T>(PhantomData<T>);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<T>(T);

pub trait Optional {
    type Value;

    fn as_option(self) -> Option<Self::Value>;

    fn from_value(value: Self::Value) -> Self;

    fn or<O: Optional>(self, rhs: O) -> <Self as Optional>::Or<O>;

    type Or<O: Optional>: Optional;
}

impl<T> Optional for Null<T> {
    type Value = T;

    fn as_option(self) -> Option<Self::Value> {
        None
    }

    fn from_value(_value: Self::Value) -> Self {
        Self(PhantomData)
    }

    fn or<O: Optional>(self, rhs: O) -> <Self as Optional>::Or<O> {
        rhs
    }

    type Or<O: Optional> = O;
}

impl<T> Optional for Value<T> {
    type Value = T;

    fn as_option(self) -> Option<Self::Value> {
        Some(self.0)
    }

    fn from_value(value: Self::Value) -> Self {
        Self(value)
    }

    fn or<O: Optional>(self, _rhs: O) -> <Self as Optional>::Or<O> {
        self
    }

    type Or<O: Optional> = Self;
}

pub trait VOr<RHS> {
    type Output;

    fn or(self, rhs: RHS) -> Self::Output;
}

impl<RHS, T> VOr<RHS> for Null<T> {
    type Output = RHS;

    fn or(self, rhs: RHS) -> Self::Output {
        rhs
    }
}

impl<LHS, RHS> VOr<RHS> for Value<LHS> {
    type Output = Self;

    fn or(self, _rhs: RHS) -> Self::Output {
        self
    }
}

// Type List
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HNil;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HCons<Head, Tail>(Head, Tail);

pub trait HList {}

impl HList for HNil {}

impl<Head, Tail: HList> HList for HCons<Head, Tail> {}

pub trait Get<Key, Value> {
    type Output: Optional;

    fn get(self) -> Self::Output;
}

impl<Key, Value> Get<Key, Value> for HNil {
    type Output = Null<Value>;

    fn get(self) -> Self::Output {
        Null(PhantomData)
    }
}

impl<Key, Head, Tail, Out1, Out2, Value> Get<Key, Value> for HCons<Head, Tail>
where
    Head: Get<Key, Value, Output = Out1>,
    Tail: Get<Key, Value, Output = Out2>,
    Out1: Optional,
    Out2: Optional,
{
    type Output = <Out1 as Optional>::Or<Out2>;

    fn get(self) -> Self::Output {
        let a = self.0.get();
        let b = self.1.get();
        a.or(b)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Member<Key, Value>(Value, PhantomData<Key>);

impl<Key, GKey, Value, GValue, KeyEq, ValueEq> Get<GKey, GValue> for Member<Key, Value>
where
    Key: TEqual<GKey, Output = KeyEq>,
    Value: TEqual<GValue, Output = ValueEq>,
    KeyEq: TBool,
    ValueEq: TBool,
{
    type Output = <KeyEq::And<ValueEq> as TBool>::Then<Value>;

    fn get(self) -> Self::Output {
        Self::Output::from_value(self.0)
    }
}

#[macro_export]
macro_rules! Member {
    ($key:ident: $value:ident) => {
        Member<$key, $value>
    };
}

#[macro_export]
macro_rules! Json {
    () => {
        HNil
    };
    ($key:ident: $value:ident) => {
        HCons<Member<$key, $value>, HNil>
    };
    ($key:ident: $value:ident, $($rest_key:ident: $rest_value:ident),* $(,)?) => {
        HCons<Member<$key, $value>, Json!($($rest_key: $rest_value),*)>
    };
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct A;
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct B;
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct C;
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct D;

    type Json = Json!(
        A: usize,
        B: char,
        C: String,
    );

    #[test]
    fn first_value() {
        let json = Json::default();

        let a = <Json as Get<A, usize>>::get(json).as_option().unwrap();

        println!("{}", std::any::type_name_of_val(&a));
    }

    #[test]
    fn second_value() {
        let json = Json::default();

        let b = <Json as Get<B, char>>::get(json).as_option().unwrap();

        println!("{}", std::any::type_name_of_val(&b));
    }

    #[test]
    fn third_value() {
        let json = Json::default();

        let c = <Json as Get<C, String>>::get(json).as_option().unwrap();

        println!("{}", std::any::type_name_of_val(&c));
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn non_value() {
        let json = Json::default();

        let d = <Json as Get<D, isize>>::get(json).as_option().unwrap();

        println!("{}", std::any::type_name_of_val(&d));
    }
}
