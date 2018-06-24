use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::{Deref, DerefMut};
use std::{mem, ptr};

use error::*;
use traits::TupCons;
use {Function, GenericDetour};

#[doc(hidden)]
pub struct __StaticDetourInner<T> where T: Function, <T as Function>::Arguments: TupCons<T> {
    pub closure: Box<Fn<<<T as Function>::Arguments as TupCons<T>>::Output, Output = T::Output>>,
    pub detour: GenericDetour<T>,
}

/// An instantiator for [StaticDetour](./struct.StaticDetour.html).
///
/// This is the type used by the [static_detours!](./macro.static_detours.html)
/// macro, it cannot be created without it.
pub struct StaticDetourController<T> where T: Function, <T as Function>::Arguments: TupCons<T> {
    inner: &'static AtomicPtr<__StaticDetourInner<T>>,
    ffi: T,
}

impl<T> StaticDetourController<T>  where T: Function, <T as Function>::Arguments: TupCons<T> {
    #[doc(hidden)]
    pub const fn __new(inner: &'static AtomicPtr<__StaticDetourInner<T>>, ffi: T) -> Self {
        StaticDetourController { inner, ffi }
    }

    /// Creates a [StaticDetour](./struct.StaticDetour.html) initialized with a
    /// target and a closure.
    ///
    /// If the detour has already been initialized, but it has not gone out of
    /// scope, an `AlreadyExisting` error will be thrown.
    pub unsafe fn initialize<C>(&self, target: T, closure: C) -> Result<StaticDetour<T>>
      where T::Arguments: TupCons<T>,
          C: Fn<<<T as Function>::Arguments as TupCons<T>>::Output, Output = T::Output> + Send + 'static {
        let mut boxed = Box::new(__StaticDetourInner {
            detour: GenericDetour::new(target, self.ffi)?,
            closure: Box::new(closure),
        });

        if !self.inner.compare_and_swap(ptr::null_mut(), &mut *boxed, Ordering::SeqCst).is_null() {
            Err(Error::AlreadyExisting)?;
        }

        mem::forget(boxed);
        Ok(StaticDetour(self.inner))
    }

    /// Returns a reference to the underlying detour.
    ///
    /// It is provided so the original function can easily be called within a
    /// detour.
    ///
    /// This is unsafe because the lifetime of the detour has no relation to the
    /// actual detour returned by `initialize`. Therefore it can be dropped at
    /// any time. Prefer to use the handle returned from `initialize` whenever
    /// possible.
    pub unsafe fn get(&self) -> Option<&GenericDetour<T>> {
        self.inner.load(Ordering::SeqCst).as_ref().map(|i| &i.detour)
    }
}

/// A type-safe static detour.
///
/// It can only be instantiated using
/// [StaticDetourController::initialize](struct.StaticDetourController.html#method.initialize).
///
/// When this object is dropped, its detour is freed and the controller can be
/// initialized once again.
/// It dereferences to [GenericDetour](./struct.GenericDetour.html) and therefore
/// providing the same interface.
///
/// Beyond this it also provides a `set_detour` method, enabling the detour to be
/// changed whilst hooked.
///
/// To see an example view the [macro's page](macro.static_detours.html).
pub struct StaticDetour<T>(&'static AtomicPtr<__StaticDetourInner<T>>) where T: Function, <T as Function>::Arguments: TupCons<T> ;

impl<T> StaticDetour<T> where T: Function, <T as Function>::Arguments: TupCons<T>  {
    /// Changes the detour, regardless of whether the target is hooked or not.
    pub fn set_detour<C>(&mut self, closure: C)
    where
        T::Arguments: TupCons<T>,
        C: Fn<<T::Arguments as TupCons<T>>::Output, Output = T::Output> + Send + 'static,
    {

        // TODO: This is not really thread-safe?
        let data = unsafe { self.0.load(Ordering::SeqCst).as_mut().unwrap() };
        data.closure = Box::new(closure);
    }
}

impl<T> Drop for StaticDetour<T> where T: Function, <T as Function>::Arguments: TupCons<T>  {
    /// Removes the detour and frees the controller for new initializations.
    fn drop(&mut self) {
        // TODO: This is not really thread-safe?
        let data = self.0.swap(ptr::null_mut(), Ordering::SeqCst);
        assert_eq!(data.is_null(), false);
        unsafe { Box::from_raw(data) };
    }
}

impl<T: Function> Deref for StaticDetour<T> where T: Function, <T as Function>::Arguments: TupCons<T> {
    type Target = GenericDetour<T>;

    fn deref(&self) -> &GenericDetour<T> {
        unsafe {
            &self.0.load(Ordering::SeqCst).as_ref().unwrap().detour
        }
    }
}

impl<T: Function> DerefMut for StaticDetour<T> where T: Function, <T as Function>::Arguments: TupCons<T> {
    fn deref_mut(&mut self) -> &mut GenericDetour<T> {
        unsafe {
            &mut self.0.load(Ordering::SeqCst).as_mut().unwrap().detour
        }
    }
}
