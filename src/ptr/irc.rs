use std::ops::Deref;
use crate::ptr::rc_box::{RcBox, decrement_and_possibly_deallocate, get_ref_boxed_content, is_exclusive, get_count, try_unwrap, clone_inner, unwrap_clone, clone_impl};
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


    pub fn try_unwrap(self) -> Result<T, Self> {
        try_unwrap(self.ptr)
            .map_err(|ptr|{
                Self {ptr} // Recover the ptr
            })
    }

    pub fn get_count(&self) -> usize {
        get_count(self.ptr)
    }

    pub fn is_exclusive(&self) -> bool {
        is_exclusive(self.ptr)
    }
}


impl <T: Clone> Irc<T> {

    pub fn unwrap_clone(self) -> T {
        unwrap_clone(self.ptr)
    }
    /// Clones the wrapped value at the `Lrc`'s head.
    pub fn clone_inner(&self) -> T {
        clone_inner(self.ptr)
    }
}



impl <T> Drop for Irc<T> {
    fn drop(&mut self) {
        unsafe{decrement_and_possibly_deallocate(self.ptr)}
    }
}

impl <T> Clone for Irc<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: clone_impl(self.ptr)
        }
    }
}

impl <T> AsRef<T> for Irc<T> {
    fn as_ref(&self) -> &T {
        get_ref_boxed_content(&self.ptr).value.as_ref()
    }
}

impl <T> Deref for Irc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
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
        self.as_ref().hash(state)
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