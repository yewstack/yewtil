use crate::ptr::rc_box::{RcBox};
use std::ops::Deref;
use failure::_core::ops::DerefMut;
use failure::_core::borrow::{Borrow, BorrowMut};
use crate::ptr::Irc;
use failure::_core::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Mutable rc pointer
///
/// It wraps a Irc, but provides
pub struct Mrc<T>(Irc<T>);


impl <T> Mrc<T> {
    pub fn new(value: T) -> Self {
        Mrc(Irc::new(value))
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_exclusive() {
            Some(self.0.get_mut_boxed_content().value.as_mut())
        } else {
            None
        }
    }

    pub fn try_unwrap(self) -> Option<T> {
        self.0.try_unwrap()
    }

    pub fn get_count(&self) -> usize {
        self.0.get_count()
    }

    pub fn is_exclusive(&self) -> bool {
        self.0.is_exclusive()
    }

    /// Returns an immutable reference counted pointer, pointing to the same value and reference count.
    pub fn irc(&self) -> Irc<T> {
        self.0.clone()
    }

    pub fn into_irc(self) -> Irc<T> {
        self.0
    }
}

impl <T: Clone> Mrc<T> {
    pub fn make_mut(&mut self) -> &mut T {
        if !self.is_exclusive() {
            let rc_box = RcBox::new(self.clone_inner());
            let ptr = rc_box.into_non_null();
            // Decrement the count for the boxed content at the current pointer.
            self.0.get_ref_boxed_content().dec_count();
            // Replace the pointers
            self.0.ptr = ptr;
        }

        self.0.get_mut_boxed_content().value.as_mut()
    }

    pub fn unwrap_clone(mut self) -> T {
        if self.is_exclusive() {
            // Don't need to decrement the count, because this structure is getting destroyed anyways.
            self.0.get_mut_boxed_content().value.take()
        } else {
            self.clone_inner()
        }
    }
    /// Clones the wrapped value at the `Lrc`'s head.
    pub fn clone_inner(&self) -> T {
        self.0.get_ref_boxed_content().value.as_ref().clone()
    }
}


impl <T> AsRef<Irc<T>> for Mrc<T> {
    fn as_ref(&self) -> &Irc<T> {
        &self.0
    }
}

impl <T> Clone for Mrc<T> {
    fn clone(&self) -> Self {
        Mrc(self.0.clone())
    }
}

impl <T> Deref for Mrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl <T: Clone> DerefMut for Mrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.make_mut()
    }
}

//impl <T> AsRef<T> for Mrc<T> {
//    fn as_ref(&self) -> &T {
//        self.get_ref_boxed_content().value.as_ref()
//    }
//}

// TODO should this be implemented?
impl <T: Clone> AsMut<T> for Mrc<T> {
    fn as_mut(&mut self) -> &mut T {
        self.make_mut()
    }
}

impl <T> Borrow<T> for Mrc<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl <T: Clone> BorrowMut<T> for Mrc<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.make_mut()
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

impl <T: Hash> Hash for Mrc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}