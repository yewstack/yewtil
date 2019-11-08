use crate::ptr::rc_box::{RcBox, try_unwrap, get_count, is_exclusive, get_mut_boxed_content, clone_impl, clone_inner, unwrap_clone, get_ref_boxed_content};
use std::ops::Deref;
use failure::_core::ops::DerefMut;
use failure::_core::borrow::{Borrow, BorrowMut};
use crate::ptr::Irc;
use failure::_core::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ptr::NonNull;

/// Mutable rc pointer
///
/// It functions similarly to an `std::rc::Rc` pointer,
/// except that it does not support weak pointers,
/// it supports `std::ops::DerefMut` via the possibly allocating `make_mut` function,
/// and that it can create immutable handles to its data (`Irc`).
///
/// This should make it just slightly more size efficient and performant than `Rc`,
/// and should be more ergonomic to use than `Rc` given that you can mutably
/// assign to it without much ceremony.
/// Passing `Irc` pointers to children guarantee that no intermediate component can modify the value
/// behind the pointer.
/// This makes it ideal for passing around configuration data where some components can ergonomicly
/// "modify" and cheaply pass the pointers back to parent components, while other components can only read it.
pub struct Mrc<T>{
    ptr: NonNull<RcBox<T>>
}


impl <T> Mrc<T> {
    /// Allocates a value behind a Mrc.
    pub fn new(value: T) -> Self {
        let rc_box = RcBox::new(value);
        let ptr = rc_box.into_non_null();
        Self {
            ptr
        }
    }

    /// Attempts to get a mutable reference to the wrapped value.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_exclusive() {
            Some(get_mut_boxed_content(&mut self.ptr).value.as_mut())
        } else {
            None
        }
    }

    /// Tries to extract the value from the Mrc, returning the Mrc if there is one or
    /// more other pointers to the value.
    pub fn try_unwrap(self) -> Result<T, Self> {
        try_unwrap(self.ptr)
            .map_err(|ptr|{
                Self {ptr} // Recover the ptr
            })
    }

    /// Gets the reference count of the Mrc
    pub fn get_count(&self) -> usize {
        get_count(self.ptr)
    }

    /// Returns true if no other pointers to the value exist.
    pub fn is_exclusive(&self) -> bool {
        is_exclusive(self.ptr)
    }

    /// Returns an immutable reference counted pointer,
    /// pointing to the same value and reference count.
    pub fn irc(&self) -> Irc<T> {
        get_ref_boxed_content(&self.ptr).inc_count();
        Irc {
            ptr: self.ptr
        }
    }

    /// Converts this Mrc into an Irc.
    pub fn into_irc(self) -> Irc<T> {
        Irc {
            ptr: self.ptr
        }
    }

}

impl <T: Clone> Mrc<T> {
    /// Returns a mutable reference to the value if it has exclusive access.
    /// If it does not have exclusive access, it will make a clone to get exclusive access
    pub fn make_mut(&mut self) -> &mut T {
        if !self.is_exclusive() {
            let rc_box = RcBox::new(self.clone_inner());
            let ptr = rc_box.into_non_null();

            // decrement the count for the boxed content at the current pointer
            // because this Mrc will point to a new value.

            // This doesn't need to check to deallocate, because the count is guaranteed to be > 1.
            get_ref_boxed_content(&self.ptr).dec_count();

            // Replace the pointers
            self.ptr = ptr;
        }

        get_mut_boxed_content(&mut self.ptr).value.as_mut()
    }

    pub fn unwrap_clone(self) -> T {
        unwrap_clone(self.ptr)
    }
    /// Clones the wrapped value at the `Lrc`'s head.
    pub fn clone_inner(&self) -> T {
        clone_inner(self.ptr)
    }
}



impl <T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: clone_impl(self.ptr)
        }
    }
}

impl <T: Clone> DerefMut for Mrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.make_mut()
    }
}


impl <T: Clone> AsMut<T> for Mrc<T> {
    fn as_mut(&mut self) -> &mut T {
        self.make_mut()
    }
}

impl <T: Clone> BorrowMut<T> for Mrc<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.make_mut()
    }
}


impl <T> AsRef<T> for Mrc<T> {
    fn as_ref(&self) -> &T {
        get_ref_boxed_content(&self.ptr).value.as_ref()
    }
}

impl <T> Deref for Mrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl <T> Borrow<T> for Mrc<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}


impl <T: PartialEq> PartialEq for Mrc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl <T: Eq> Eq for Mrc<T> {}

impl <T: PartialOrd> PartialOrd for Mrc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl <T: Ord> Ord for Mrc<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl <T: Hash> Hash for Mrc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}