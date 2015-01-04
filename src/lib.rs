#![feature(globs, slicing_syntax, phase, macro_rules)]
#![allow(dead_code)]

#[phase(plugin, link)] extern crate log;

pub use self::Crate::*;
pub use self::Type::*;

use std::collections::BitvSet;

#[deriving(PartialEq, Show)]
enum Type {
    Concrete(Crate, Vec<Type>),
    Parameter(uint)
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
);

macro_rules! remote(
    ($($e:expr),*) => ({
        Concrete(Remote, vec!($($e),*))
    });
    ($($e:expr),+,) => (remote!($($e),+))
);

fn not_ok(krate: Crate, types: &[Type]) -> bool {
    !ok(krate, types)
}

fn ok(krate: Crate, types: &[Type]) -> bool {
    /*!
     * True if it is ok to apply a trait defined in `krate` to the types `types`
     */

    let result = krate == Local || {
        types.iter().any(|t| type_local(t)) && {
            let covered_params = covered_params_in_tys(types.as_slice());
            let all_params = params_in_tys(types.as_slice());
            all_params.is_subset(&covered_params)
        }
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
        Parameter(_) => false,
    };

    debug!("type_local({}) = {}",
           ty, result);

    result
}

fn covered_params_in_tys(tys: &[Type]) -> BitvSet {
    let mut set = BitvSet::new();
    for ty in tys.iter() {
        set.union_with(&covered_params_in_ty(ty))
    }
    set
}

fn covered_params_in_ty(ty: &Type) -> BitvSet {
    match *ty {
        Concrete(Local, ref tys) => params_in_tys(tys.as_slice()),
        Concrete(Remote, ref tys) => covered_params_in_tys(tys.as_slice()),
        Parameter(_) => BitvSet::new(),
    }
}

fn params_in_tys(tys: &[Type]) -> BitvSet {
    let mut set = BitvSet::new();
    for ty in tys.iter() {
        set.union_with(&params_in_ty(ty))
    }
    set
}

fn params_in_ty(ty: &Type) -> BitvSet {
    match *ty {
        Concrete(_, ref tys) => params_in_tys(tys.as_slice()),
        Parameter(i) => {
            let mut r = BitvSet::new();
            r.insert(i);
            r
        }
    }
}

#[test]
fn lone_type_parameter() {
    /*! `impl<T> Show for T` -- not_ok */
    assert!(not_ok(Remote, &[Parameter(0)]));
}

#[test]
fn type_parameter() {
    /*! `impl<T> Show for Foo<T>` -- OK */
    assert!(ok(Remote, &[local!(Parameter(0))]));
}

#[test]
fn overlapping_pairs() {
    /*! `impl<T> Show for Pair<Option<T>, Option<Foo>>` -- Bad */

    // Bad because another crate could do:
    // impl<T> Show for Pair<Option<Bar>, Option<T>>

    assert!(not_ok(Remote,
                   &[remote!(                  // Pair<
                       remote!(Parameter(0)),  //   Option<T>,
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
                     Parameter(0)]));
}

#[test]
fn blanket() {
    /*! `impl<T> Foo for T` -- OK */

    assert!(ok(Local, &[Parameter(0)]));
}

#[test]
fn vec_local_1() {
    /*! `impl Clone for Vec<Foo>` -- OK */

    assert!(ok(Remote, &[remote!(local!())]));
}

#[test]
fn vec_local_2() {
    /*! `impl<T> Clone for Vec<Foo<T>>` -- OK */

    assert!(ok(Remote, &[remote!(local!(Parameter(0)))]));
}

#[test]
fn all_remote() {
    /*! `impl Clone for int` -- not OK */

    assert!(not_ok(Remote, &[remote!(remote!())]));
}

#[test]
fn iterator_vec() {
    /*! `impl Iterator<T> for Foo<T>` -- OK */

    assert!(ok(Remote, &[Parameter(0), local!(Parameter(0))]));
}

#[test]
fn iterator_vec_any_elem() {
    /*! `impl Iterator<U> for Foo<T>` -- not OK */

    assert!(not_ok(Remote, &[Parameter(1), local!(Parameter(0))]));
}

#[test]
fn aturon1() {
    /*!
     * Crate A: trait A<T> { ... }
     * Crate B: struct B<T> { ... } impl<T> A<B<T>> for T { ... }
     * Crate C: struct C { ... } impl<T> A<T> for C { ... }
     */

    assert!(ok(Remote, &[Parameter(0), local!(Parameter(0))]));
    assert!(not_ok(Remote, &[local!(), Parameter(0)]));
}
