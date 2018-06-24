//! Traits describing detours and applicable functions.
//!
//! Several of the traits in this module are automatically implemented and
//! should generally not be implemented by users of this library.

/// Trait representing a function that can be used as a target or detour for
/// detouring.
pub unsafe trait Function: Sized + Copy + Sync + 'static {
  /// The argument types as a tuple.
  type Arguments;

  /// The return type.
  type Output;

  /// Constructs a `Function` from an untyped pointer.
  unsafe fn from_ptr(ptr: *const ()) -> Self;

  /// Returns an untyped pointer for this function.
  fn to_ptr(&self) -> *const ();
}

/// Trait indicating that `Self` can be detoured by the given function `D`.
pub unsafe trait HookableWith<D: Function>: Function {}

unsafe impl<T: Function> HookableWith<T> for T {}

impl_hookable! {
    __arg_0:  A, __arg_1:  B, __arg_2:  C, __arg_3:  D, __arg_4:  E, __arg_5:  F, __arg_6:  G,
    __arg_7:  H, __arg_8:  I, __arg_9:  J, __arg_10: K, __arg_11: L, __arg_12: M, __arg_13: N
}

pub trait TupCons<H> {
  type Output;
  fn tup_cons(self, h: H) -> Self::Output;
}

macro_rules! impl_tup_cons {
    (@single $($ty:ident,)*) => {
        impl <T1, $($ty),*> TupCons<T1> for ($($ty,)*) {
            type Output = (T1, $($ty),*);
            #[allow(non_snake_case)]
            fn tup_cons(self, h: T1) -> Self::Output {
                let ($($ty,)*) = self;
                (h, $($ty),*)
            }
        }
    };

    () => {
        impl_tup_cons! { @single }
    };

    ($ty1:ident, $($ty:ident,)*) => {
        impl_tup_cons!{ @single $ty1, $($ty,)* }
        impl_tup_cons!{ $($ty,)* }
    };
}

impl_tup_cons! {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N,
}
