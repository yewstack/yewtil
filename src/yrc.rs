use std::ptr::NonNull;
use std::cell::Cell;
use std::ops::Deref;
use failure::_core::ops::DerefMut;
use failure::_core::fmt::{Debug, Formatter, Error};
use std::fmt;
use std::io::{stdout, Write};

// TODO Plan
// So this is a good start, but it isn't actually safe.
// To make it safe, we can't allow the Owner to mutate its contents.
// Instead, the owner has the ability to "replace" its contents.
// Replacing doesn't actually remove the old item, it just sticks the new item as the head, and have the head point to the node that was previously the head.
// So the old head stays around, and doesn't affect the existing Shared copies.
// If someone forgets to rerender after they modify the Owned - then nothing can get corrupted.
// When the parent component re-renders, it will hand out new Shareds (ref-counted-copies of the head), then the old Shareds will be destroyed.
// When a shared is destroyed it needs to know the previous and next pointers in the linked list so it can remove itself and connect the prev and next nodes.
// This means that this is technically a doubly linked list. --aaaaaaahh
//
// Is there even a need for two different types then?
// Just one could work - Shared - OR we call this a LRC - a Linked Reference Counted pointer

#[derive(Debug)]
pub struct YrcBox<T: Debug> {
    prev: Option<Yrc<T>>,
    count: Cell<usize>,
    inner: T,
    next: Option<Yrc<T>>
}

impl <T: Default + Debug> Default for YrcBox<T> {
    fn default() -> Self {
        YrcBox {
            prev: None,
            count: Cell::new(1),
            inner: T::default(),
            next: None
        }
    }
}

/// A boolean indicating if the count is zero.
type IsZero = bool;

impl <T: Debug> YrcBox<T> {

    fn new(inner: T) -> Self {
        YrcBox {
            prev: None,
            count: Cell::new(1),
            inner,
            next: None
        }
    }

    fn inc_count(&self) {
        let mut count = self.get_count();
        count += 1;
        self.set_count(count)
    }

    fn dec_count(&self) -> IsZero {
        let mut count = self.get_count();
        dbg!(&self);
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
pub struct Yrc<T: Debug> {
    ptr: NonNull<YrcBox<T>>
}


impl <T: Debug + Default> Yrc<T> {
    pub fn new(inner: T) -> Self {
        // unstable code - although more succinct.
        // Self::from_inner(Box::into_raw_non_null(Box::new(inner)))
        Self::from_inner(unsafe {NonNull::new_unchecked(Box::into_raw(
            Box::new(YrcBox::new(inner) )
        ))})
    }

    // pushes to the head
    pub fn set(&mut self, inner: T) {
        let mut new_boxed = YrcBox::new(inner);

        new_boxed.next = Some(self.clone());
        let mut new = Self::from_inner(unsafe {
            NonNull::new_unchecked(Box::into_raw(
                Box::new(new_boxed)
            ))
        });

        let c = new.clone();
        unsafe {
//            (*self.ptr.as_ptr()).prev = Some(c);//Some(new.clone());
        }

//        std::mem::swap(self, &mut new)

//        new

//        if false {
//            let should_drop_old: bool = unsafe {
//                // decrement the old head's count, as we don't really want to keep it alive if no other references exist.
//                if self.ptr.as_ref().get_count() == 1 {
//                    println!("Last reference, dropping old head: {:#?}", self);
//                    // Destroy the old pointer if no one has borrowed it.
//                    true
//                } else if self.ptr.as_ref().get_count() > 1 {
//                    println!("decrementing ptr");
//                    // Set the prev value in the old head.
//                    self.ptr.as_ref().dec_count();
//                    false
//                } else {
//                    panic!("old_ptr count is 0!")
//                }
//            };
//            let should_attach_new_to_olds_prev = !should_drop_old;
//            if should_drop_old {
//                std::mem::drop(self);
//            } else {
//                new_boxed.next = Some(self);
//            }
//
//
//            let mut new = Self::from_inner(unsafe {
//                NonNull::new_unchecked(Box::into_raw(
//                    Box::new(new_boxed)
//                ))
//            });
//
//            if should_attach_new_to_olds_prev {
//                unsafe {
//                    let copy = new.clone();
//                    let old: &mut Yrc<T> = new.ptr.as_mut().next.as_mut().unwrap();
//                    old.ptr.as_mut().prev = Some(Yrc::default());
//                }
//            }
//            new
//        }
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


impl <T: Default + Debug> Default for Yrc<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T: PartialEq+ Debug> PartialEq for Yrc<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe{self.ptr.as_ref().inner.eq(&other.ptr.as_ref().inner)}
    }
}

impl <T: fmt::Debug> fmt::Debug for Yrc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe{
            f.debug_tuple("Yrc").field(self.ptr.as_ref()).finish()
        }
    }
}

impl <T: Debug> Deref for Yrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T: Debug> DerefMut for Yrc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut self.ptr.as_mut().inner}
    }
}

impl <T: Debug> AsRef<T> for Yrc<T> {
    fn as_ref(&self) -> &T {
        unsafe {&self.ptr.as_ref().inner}
    }
}

impl <T: Debug> AsMut<T> for Yrc<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe {&mut self.ptr.as_mut().inner}
    }
}



impl <T: Debug> Drop for Yrc<T> {
    fn drop(&mut self) {
        println!("Dropping!!!");
        let is_count_zero = unsafe {
            let yrc_box = self.ptr.as_mut();
            yrc_box.dec_count()
        };
        println!(" is count zero for {:?}: {}", unsafe{&self.ptr.as_ref().inner}, is_count_zero);
        if is_count_zero {
            unsafe {
                {
                    let ptr = self.ptr.as_mut();
                    let mut prev = ptr.prev.take();
                    let mut next = ptr.next.take();
                    println!("Dropping: Prev would be: {:?}", prev);
                    println!("Dropping: Next would be: {:?}", next);

                    match (prev, next) {
                        (Some(mut prev), Some(mut next)) => {
                            prev.ptr.as_mut().next = Some(next);
                        }
                        (Some(mut prev), None) => {
                            prev.ptr.as_mut().next = None;
                        }
                        (None, Some(next)) => {

                        }
                        (None, None) => {}
                    }
//
//                    if let Some(prev) = prev {
//                        (*prev).next = next;
//
////                        if let Some(next) = (*prev).next.as_mut() {
////                            (*next.ptr.as_ptr()).prev = Some(prev);
////                        }
//                    } else {
//                        if let Some(mut next) = next {
//                            next.ptr.as_mut().prev = prev;
//                        }
//                    }
                }
                std::mem::forget(self.ptr.as_mut().next.take());
                std::mem::forget(self.ptr.as_mut().prev.take());
                std::ptr::drop_in_place(self.ptr.as_mut());
            }
        }
    }
}




impl <T: Debug> Clone for Yrc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let yrc_box = self.ptr.as_ref();
            yrc_box.inc_count();
            Yrc {
                ptr: self.ptr
            }
        }
    }
}




#[cfg(test)]
mod test {
    use super::*;

//    #[test]
//    fn create_yrc() {
//        let item: u32 = 42;
//        let item = Yrc::new(item);
//        let count = unsafe {
//            item.ptr.as_ref().count.get()
//        };
//
//        assert_eq!(count, 1)
//    }
//
//    #[test]
//    fn share_yrc() {
//        let item: u32 = 42;
//        let item = Yrc::new(item);
//        let shared = item.share();
//        let count = unsafe {
//            shared.ptr.as_ref().count.get()
//        };
//        assert_eq!(count, 2)
//    }
//
//    #[test]
//    fn drop_yrc_decrements_count() {
//        let item: Yrc<usize> = Yrc::new(42);
//        let shared = item.share();
//
//        // This drop should decrement the count in the yrc
//        std::mem::drop(item);
//
//        let count = unsafe {
//            shared.ptr.as_ref().count.get()
//        };
//        assert_eq!(count, 1)
//    }
//
//
//    #[test]
//    fn drop_shared_yrc_decrements_count() {
//        let item: Yrc<usize> = Yrc::new(42);
//        let shared = item.share();
//        let count = unsafe {
//            shared.ptr.as_ref().count.get()
//        };
//        assert_eq!(count, 2);
//
//        std::mem::drop(shared);
//        let count = unsafe {
//            item.ptr.as_ref().get_count()
//        };
//        assert_eq!(count, 1);
//    }
//
//    #[test]
//    fn as_ref_yrc() {
//        let item: Yrc<usize> = Yrc::new(42);
//        assert_eq!(&42usize, item.as_ref());
//    }
//
//    #[test]
//    fn as_mut_yrc() {
//        let mut item: Yrc<usize> = Yrc::new(42);
//        assert_eq!(&42usize, item.as_ref());
//
//        let inner = item.as_mut();
//        *inner += 1;
//
//        assert_eq!(&43usize, item.as_ref());
//    }
//
//    #[test]
//    fn clone_shared_counts_are_the_same() {
//        let shared = Shared::new(42);
//        let cloned = shared.clone();
//        let count_shared = unsafe {
//            shared.ptr.as_ref().get_count()
//        };
//
//        let count_cloned = unsafe {
//            cloned.ptr.as_ref().get_count()
//        };
//        assert_eq!(count_cloned, count_shared);
//        assert_eq!(count_shared, 2);
//    }


    //////


    #[test]
    fn clone_no_increment() {
        let s = Yrc::new("Hello");

        let k = Yrc {
            ptr: s.ptr
        };
        std::mem::forget(k)
    }

    #[test]
    fn set_sets_value() {
        let mut s = Yrc::new("hello");
        s.set("world");
        assert_eq!(s.as_ref(), &"world")
    }

    #[test]
    fn set_without_shared_removes_head() {
        let mut s = Yrc::new("hello");
        s.set("world");
        assert_eq!(unsafe{&s.ptr.as_ref().next}, &None)
    }


    #[test]
    fn set_with_shared_clone() {
        println!("Starting");
        let mut s = Yrc::new("hello");

        println!("Cloning");
        let c = s.clone();
        s.set("world");
        println!("AAAAH");
//        assert_eq!(c.as_ref(), &"hello");
//        assert_eq!(s.as_ref(), &"world");
//        println!("Test")

    }


}