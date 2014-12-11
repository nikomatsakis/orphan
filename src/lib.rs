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

fn not_ok(krate: Crate, types: &[Type]) -> bool {
    !ok(krate, types)
}

fn ok(krate: Crate, types: &[Type]) -> bool {
    let result = krate == Local || {
        types.iter().all(|t| type_ok(t))
    };

    debug!("ok({},{}) = {}",
           krate, types, result);

    result
}

fn type_ok(ty: &Type) -> bool {
    let result = match *ty {
        Concrete(krate, ref types) => ok(krate, types[]),
        Parameter => false,
    };

    debug!("type_ok({}) = {}",
           ty, result);

    result
}

#[test]
fn lone_type_parameter() {
    /*! `impl<T> Show for T` -- not_ok */
    assert!(not_ok(Remote, &[Parameter]));
}

#[test]
fn type_parameter() {
    /*! `impl<T> Show for Foo<T>` -- OK */
    assert!(ok(Remote, &[Concrete(Local, vec!(Parameter))]));
}

#[test]
fn overlapping_pairs() {
    /*! `impl<T> Show for Pair<Option<T>, Option<Foo>>` -- Bad */

    // Bad because another crate could do:
    // impl<T> Show for Pair<Option<Bar>, Option<T>>

    assert!(not_ok(Remote,
                   &[Concrete(Remote, // Pair
                              vec!(Concrete(Remote, // Option
                                            vec!(Parameter)), // T
                                   Concrete(Local, // Foo
                                            vec!())))]));
}

#[test]
fn bigint_int() {
    /*! `impl Add<Foo> for int` -- OK */

    assert!(ok(Remote,
               &[Concrete(Local, vec!()),
                 Concrete(Remote, vec!())]));
}

#[test]
fn bigint_param() {
    /*! `impl Add<Foo> for T` -- not OK */

    assert!(not_ok(Remote,
                   &[Concrete(Local, vec!()),
                     Parameter]));
}

#[test]
fn blanket() {
    /*! `impl<T> Foo for T` -- OK */

    assert!(ok(Local, &[Parameter]));
}

#[test]
fn vec_local_1() {
    /*! `impl Clone for Vec<Foo>` -- OK */

    assert!(ok(Remote, &[Concrete(Remote, vec!(Concrete(Local, vec!())))]));
}

#[test]
fn vec_local_2() {
    /*! `impl<T> Clone for Vec<Foo<T>>` -- OK */

    assert!(ok(Remote, &[Concrete(Remote, vec!(Concrete(Local, vec!(Parameter))))]));
}
