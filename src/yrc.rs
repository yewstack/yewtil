use std::ptr::NonNull;
use std::cell::Cell;
use std::ops::Deref;
use failure::_core::ops::DerefMut;


pub struct YrcBox<T> {
    count: Cell<usize>,
    inner: T
}

impl <T: Default> Default for YrcBox<T> {
    fn default() -> Self {
        YrcBox {
            count: Cell::new(1),
            inner: T::default()
        }
    }
}

/// A boolean indicating if the count is zero.
type IsZero = bool;

impl <T> YrcBox<T> {
    fn inc_count(&self) {
        let mut count = self.get_count();
        count += 1;
        self.set_count(count)
    }

    fn dec_count(&self) -> IsZero {
        let mut count = self.get_count();
        count -= 1;
        self.set_count(count);
        count == 0
    }

    fn get_count(&self) -> usize {
        self.count.get()
    }
    fn set_count(&self, count: usize) {
        self.count.set(count)
    }

}

// TODO, consider marking any mutation of the inner as unsafe, because it really is, but shouldn't ever be a problem in yew unless you really try.
// TODO: Alternatively, maybe use a linked list structure to keep old references around. Like a lot of lock-less data structures do. Don't allow mutation of the owned data, but instead allow "replacement", which adds a new head?
/// A smart pointer optimized for use in yew.
///
/// There are two variants:
/// Yrc - which has the ability to mutate the data,
/// Shared - which can only read the data.
///
/// By enforcing that there can only be one entity that can write at a time (unlike Rc, you can't clone Yrc),
/// and because this cannot be used concurrently,
/// as long as you don't hold-references to the owned data by a Yrc or Shared item and _then_ mutate it, you are fine.
///
/// One component can be responsible for updating this data,
///
/// # Warning
/// This will cause undefined behavior if the following invariants are not upheld by your program:
/// * Don't hold references.
/// * Always re-render children holding Shareds when you update a Yrc
#[derive(Debug)]
pub struct Yrc<T> {
    ptr: NonNull<YrcBox<T>>
}

// TODO see if this can just be implemented as a newtype around Yrc
#[derive(Debug)]
pub struct Shared<T> {
    ptr: NonNull<YrcBox<T>>
}


impl <T> Yrc<T> {
    pub fn new(inner: T) -> Self {
        // unstable code - although more succinct.
        // Self::from_inner(Box::into_raw_non_null(Box::new(inner)))
        Self::from_inner(unsafe {NonNull::new_unchecked(Box::into_raw(
            Box::new(YrcBox {
                count: Cell::new(1),
                inner
            })
        ))})
    }

    pub fn share(&self) -> Shared<T> {
        unsafe {
            let yrc_box = self.ptr.as_ref();
            let mut count = yrc_box.get_count();
            count += 1;
            yrc_box.set_count(count);
        };
        // Just copy the pointer, not the data.
        Shared {ptr: self.ptr}
    }



    fn from_inner(ptr: NonNull<YrcBox<T>>) -> Self {
        Self {
            ptr,
        }
    }

//    unsafe fn from_ptr(ptr: *mut YrcBox<T>) -> Self {
//        Self::from_inner(NonNull::new_unchecked(ptr))
//    }
}


impl <T: Default> Default for Yrc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T> Deref for Yrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T> DerefMut for Yrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut self.ptr.as_mut().inner}
    }
}

impl <T> AsRef<T> for Yrc<T> {
    fn as_ref(&self) -> &T {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T> AsMut<T> for Yrc<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe {&mut self.ptr.as_mut().inner}
    }
}



impl <T> Drop for Yrc<T> {
    fn drop(&mut self) {
        let is_count_zero = unsafe {
            let yrc_box = self.ptr.as_mut();
            yrc_box.dec_count()

        };
        if is_count_zero {
            unsafe {
                std::ptr::drop_in_place(self.ptr.as_mut());
            }
        }
    }
}

impl <T> Shared<T> {
    pub fn new(inner: T) -> Self {
    // unstable code - although more succinct.
    // Self::from_inner(Box::into_raw_non_null(Box::new(inner)))
        Self::from_inner(unsafe {NonNull::new_unchecked(Box::into_raw(
            Box::new(YrcBox {
                count: Cell::new(1),
                inner
            })
        ))})
    }

    fn from_inner(ptr: NonNull<YrcBox<T>>) -> Self {
        Self {
            ptr,
        }
    }

//    unsafe fn from_ptr(ptr: *mut YrcBox<T>) -> Self {
//        Self::from_inner(NonNull::new_unchecked(ptr))
//    }
}

impl <T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        unsafe {
            let yrc_box = self.ptr.as_ref();
            yrc_box.inc_count();
            Shared {
                ptr: self.ptr
            }
        }
    }
}

impl <T> Drop for Shared<T> {
    fn drop(&mut self) {
        let is_count_zero = unsafe {
            let yrc_box = self.ptr.as_mut();
            yrc_box.dec_count()
        };

        if is_count_zero {
            unsafe {
                std::ptr::drop_in_place(self.ptr.as_mut());
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_yrc() {
        let item: u32 = 42;
        let item = Yrc::new(item);
        let count = unsafe {
            item.ptr.as_ref().count.get()
        };

        assert_eq!(count, 1)
    }

    #[test]
    fn share_yrc() {
        let item: u32 = 42;
        let item = Yrc::new(item);
        let shared = item.share();
        let count = unsafe {
            shared.ptr.as_ref().count.get()
        };
        assert_eq!(count, 2)
    }

    #[test]
    fn drop_yrc_decrements_count() {
        let item: Yrc<usize> = Yrc::new(42);
        let shared = item.share();

        // This drop should decrement the count in the yrc
        std::mem::drop(item);

        let count = unsafe {
            shared.ptr.as_ref().count.get()
        };
        assert_eq!(count, 1)
    }


    #[test]
    fn drop_shared_yrc_decrements_count() {
        let item: Yrc<usize> = Yrc::new(42);
        let shared = item.share();
        let count = unsafe {
            shared.ptr.as_ref().count.get()
        };
        assert_eq!(count, 2);

        std::mem::drop(shared);
        let count = unsafe {
            item.ptr.as_ref().get_count()
        };
        assert_eq!(count, 1);
    }

    #[test]
    fn as_ref_yrc() {
        let item: Yrc<usize> = Yrc::new(42);
        assert_eq!(&42usize, item.as_ref());
    }

    #[test]
    fn as_mut_yrc() {
        let mut item: Yrc<usize> = Yrc::new(42);
        assert_eq!(&42usize, item.as_ref());

        let inner = item.as_mut();
        *inner += 1;

        assert_eq!(&43usize, item.as_ref());
    }

    #[test]
    fn clone_shared_counts_are_the_same() {
        let shared = Shared::new(42);
        let cloned = shared.clone();
        let count_shared = unsafe {
            shared.ptr.as_ref().get_count()
        };

        let count_cloned = unsafe {
            cloned.ptr.as_ref().get_count()
        };
        assert_eq!(count_cloned, count_shared);
        assert_eq!(count_shared, 2);
    }

}