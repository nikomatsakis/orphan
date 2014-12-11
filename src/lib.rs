#![feature(globs, slicing_syntax, phase)]
#![allow(dead_code)]

#[phase(plugin, link)] extern crate log;

pub use self::Crate::*;
pub use self::Type::*;

#[deriving(PartialEq, Show)]
enum Type {
    Concrete(Crate, Vec<Type>),
    Parameter
}

#[deriving(PartialEq, Show)]
enum Crate {
    Local,
    Remote,
}

fn orphan(krate: Crate, types: &[Type]) -> bool {
    !not_orphan(krate, types)
}

fn not_orphan(krate: Crate, types: &[Type]) -> bool {
    let result = krate == Local || {
        types.iter().any(|t| type_local(t)) &&
            types.iter().all(|t| *t != Parameter)
    };

    debug!("not_orphan({},{}) = {}",
           krate, types, result);

    result
}

fn type_local(ty: &Type) -> bool {
    let result = match *ty {
        Concrete(krate, ref types) => not_orphan(krate, types[]),
        Parameter => false
    };

    debug!("type_local({}) = {}",
           ty, result);

    result
}

#[test]
fn lone_type_parameter() {
    /*! `impl<T> Show for T` -- orphan */
    assert!(orphan(Remote, &[Parameter]));
}

#[test]
fn type_parameter() {
    /*! `impl<T> Show for Foo<T>` -- OK */
    assert!(not_orphan(Remote, &[Concrete(Local, vec!(Parameter))]));
}

#[test]
fn conflict() {
    /*! `impl<T> Show for Pair<Option<T>, Option<Foo>>` -- Bad */

    // Bad because another crate could do:
    // impl<T> Show for Pair<Option<Bar>, Option<T>>

    assert!(not_orphan(Remote,
                       &[Concrete(Remote, // Pair
                                  vec!(Concrete(Remote, // Option
                                                vec!(Parameter)), // T
                                       Concrete(Local, // Foo
                                                vec!())))]));
}
