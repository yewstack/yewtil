use std::ops::Deref;
use crate::ptr::rc_box::{RcBox, decrement_and_possibly_deallocate};
use std::ptr::NonNull;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Immutable rc pointer.
///
/// It is a newtype around Mrc without &mut functions exposed.
pub struct Irc<T> {
    pub(crate) ptr: NonNull<RcBox<T>>
}

impl <T> Irc<T> {
    pub fn new(value: T) -> Self {
        let rc_box = RcBox::new(value);
        let ptr = rc_box.into_non_null();
        Self {
            ptr
        }
    }


    pub fn try_unwrap(mut self) -> Option<T> {
        if self.is_exclusive() {
            Some(self.get_mut_boxed_content().value.take())
        } else {
            None
        }
    }

    pub fn get_count(&self) -> usize {
        self.get_ref_boxed_content().get_count()
    }

    pub fn is_exclusive(&self) -> bool {
        self.get_count() == 1
    }

    /// Gets a mutable reference to the head node.
    pub(crate) fn get_mut_boxed_content(&mut self) -> &mut RcBox<T> {
        unsafe { self.ptr.as_mut() }
    }

    /// Gets a reference to the head node.
    pub(crate) fn get_ref_boxed_content(&self) -> &RcBox<T> {
        unsafe { self.ptr.as_ref()}
    }
}


impl <T: Clone> Irc<T> {

    pub fn unwrap_clone(mut self) -> T {
        if self.is_exclusive() {
            // Don't need to decrement the count, because this structure is getting destroyed anyways.
            self.get_mut_boxed_content().value.take()
        } else {
            self.clone_inner()
        }
    }
    /// Clones the wrapped value at the `Lrc`'s head.
    pub fn clone_inner(&self) -> T {
        self.get_ref_boxed_content().value.as_ref().clone()
    }
}



impl <T> Drop for Irc<T> {
    fn drop(&mut self) {
        unsafe{decrement_and_possibly_deallocate(self.ptr)}
    }
}

impl <T> Clone for Irc<T> {
    fn clone(&self) -> Self {
        // Increment the ref count
        self.get_ref_boxed_content().inc_count();
        Self {
            ptr: self.ptr
        }
    }
}

impl <T> Deref for Irc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}


impl <T> AsRef<T> for Irc<T> {
    fn as_ref(&self) -> &T {
        self.get_ref_boxed_content().value.as_ref()
    }
}

impl <T> Borrow<T> for Irc<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}



impl <T: PartialEq> PartialEq for Irc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl <T: Eq> Eq for Irc<T> {}

impl <T: PartialOrd> PartialOrd for Irc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl <T: Ord> Ord for Irc<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl <T: Hash> Hash for Irc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_ref_boxed_content().value.as_ref().hash(state)
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_new() {
        let _irc = Irc::new(0);
    }
}