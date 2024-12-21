#![feature(specialization)]

use std::marker::PhantomData;

// bool
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TTrue;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TFalse;

pub trait TBool {
    type Then<T>: Optional<T>;

    fn as_bool() -> bool;
}

impl TBool for TTrue {
    type Then<T> = Value<T>;

    fn as_bool() -> bool {
        true
    }
}

impl TBool for TFalse {
    type Then<T> = Null;
    fn as_bool() -> bool {
        false
    }
}

pub trait TOr<RHS: TBool> {
    type Output: TBool;
}

impl<RHS: TBool> TOr<RHS> for TFalse {
    type Output = RHS;
}

impl<RHS: TBool> TOr<RHS> for TTrue {
    type Output = TTrue;
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
pub struct Null;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value<T>(T);

pub trait Optional<T> {
    fn as_option(self) -> Option<T>;

    fn from_value(value: T) -> Self;

    type Or<O: Optional<U>, U>;
}

impl<T> Optional<T> for Null {
    fn as_option(self) -> Option<T> {
        None
    }

    fn from_value(_value: T) -> Self {
        Null
    }

    type Or<O: Optional<U>, U> = O;
}

impl<T> Optional<T> for Value<T> {
    fn as_option(self) -> Option<T> {
        Some(self.0)
    }

    fn from_value(value: T) -> Self {
        Self(value)
    }

    type Or<O: Optional<U>, U> = Self;
}

pub trait VOr<RHS> {
    type Output;

    fn or(self, rhs: RHS) -> Self::Output;
}

impl<RHS> VOr<RHS> for Null {
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

pub trait Get<Key> {
    type Output;

    fn get(self) -> Self::Output;
}

impl<Key> Get<Key> for HNil {
    type Output = Null;

    fn get(self) -> Self::Output {
        Null
    }
}

impl<Key, Head, Tail, Out1, Out2> Get<Key> for HCons<Head, Tail>
where
    Head: Get<Key, Output = Out1>,
    Tail: Get<Key, Output = Out2>,
    Out1: VOr<Out2>,
{
    type Output = <Out1 as VOr<Out2>>::Output;

    fn get(self) -> Self::Output {
        let a = self.0.get();
        let b = self.1.get();
        a.or(b)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Member<Key, Value>(Value, PhantomData<Key>);

impl<Key, GKey, Value> Get<GKey> for Member<Key, Value>
where
    Key: TEqual<GKey>,
{
    type Output = <<Key as TEqual<GKey>>::Output as TBool>::Then<Value>;

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

    #[test]
    fn test() {
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct A;
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct B;
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct C;

        type Json = Json!(
            A: usize,
            B: char,
            C: String,
        );

        let json = Json::default();

        let a = <Json as Get<B>>::get(json);

        println!("{:?}", a);
    }
}
