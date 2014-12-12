#![feature(globs, slicing_syntax, phase, macro_rules)]
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

macro_rules! local(
    ($($e:expr),*) => ({
        Concrete(Local, vec!($($e),*))
    });
    ($($e:expr),+,) => (local!($($e),+))
)

macro_rules! remote(
    ($($e:expr),*) => ({
        Concrete(Remote, vec!($($e),*))
    });
    ($($e:expr),+,) => (remote!($($e),+))
)

fn not_ok(krate: Crate, types: &[Type]) -> bool {
    !ok(krate, types)
}

fn ok(krate: Crate, types: &[Type]) -> bool {
    /*!
     * True if it is ok to apply a trait defined in `krate` to the types `types`
     */

    let result = krate == Local || {
        types.iter().any(|t| type_local(t)) &&
            !types.iter().any(|t| uncovered_param(t))
    };

    debug!("ok({},{}) = {}",
           krate, types, result);

    result
}

fn type_local(ty: &Type) -> bool {
    /*!
     * True if the type `ty` references anything local.
     */

    let result = match *ty {
        Concrete(Local, _) => true,
        Concrete(Remote, ref types) => types.iter().any(|t| type_local(t)),
        Parameter => false,
    };

    debug!("type_local({}) = {}",
           ty, result);

    result
}

fn uncovered_param(ty: &Type) -> bool {
    /*!
     * True if the type `ty` contains type parameters that do not appear underneath
     * something local.
     */

    let result = match *ty {
        Concrete(Local, _) => false,
        Concrete(Remote, ref v) => v.iter().any(|t| uncovered_param(t)),
        Parameter => true,
    };

    debug!("uncovered_param({}) = {}",
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
    assert!(ok(Remote, &[local!(Parameter)]));
}

#[test]
fn overlapping_pairs() {
    /*! `impl<T> Show for Pair<Option<T>, Option<Foo>>` -- Bad */

    // Bad because another crate could do:
    // impl<T> Show for Pair<Option<Bar>, Option<T>>

    assert!(not_ok(Remote,
                   &[remote!(                  // Pair<
                       remote!(Parameter),     //   Option<T>,
                       remote!(local!()))]));  //   Option<Foo> >
}

#[test]
fn bigint_int() {
    /*! `impl Add<Foo> for int` -- OK */

    assert!(ok(Remote,
               &[local!(),
                 remote!()]));
}

#[test]
fn bigint_vecint() {
    /*! `impl Add<Foo> for Vec<int>` -- OK */

    assert!(ok(Remote,
               &[local!(),
                 remote!(remote!())]));
}

#[test]
fn bigint_param() {
    /*! `impl Add<Foo> for T` -- not OK */

    assert!(not_ok(Remote,
                   &[local!(),
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

    assert!(ok(Remote, &[remote!(local!())]));
}

#[test]
fn vec_local_2() {
    /*! `impl<T> Clone for Vec<Foo<T>>` -- OK */

    assert!(ok(Remote, &[remote!(local!(Parameter))]));
}

#[test]
fn all_remote() {
    /*! `impl Clone for int` -- not OK */

    assert!(not_ok(Remote, &[remote!(remote!())]));
}
