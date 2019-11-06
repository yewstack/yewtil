use std::ptr::NonNull;
use std::cell::Cell;
use std::fmt;
use failure::_core::fmt::{Formatter, Error};
use std::ops::Deref;

pub type IsZero = bool;


struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    element: T,
    count: Cell<usize>
}

impl <T> Node<T> {
    fn new(element: T) -> Self {
        Node {
            next: None,
            prev: None,
            element,
            count: Cell::new(1)
        }
    }

    fn into_not_null(self) -> NonNull<Self> {
        unsafe {
            NonNull::new_unchecked(Box::into_raw(
                Box::new(self)
            ))
        }
    }

    fn get_count(&self) -> usize {
        self.count.get()
    }

    fn inc_count(&self) {
        let mut count = self.count.get();
        count += 1;
        self.count.set(count);
    }

    fn dec_count(&self) -> IsZero {
        let mut count = self.count.get();
        count -= 1;
        self.count.set(count);
        count == 0
    }
}

impl <T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe{self.element.eq(&other.element)}
    }
}

impl <T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("element", &self.element)
            .field("count", &self.count)
            .finish()
    }
}





/// Linked Reference Counted pointer
///
/// Probably the dumbest smart pointer - A doubly linked list where the head represents the value,
/// and the other elements record a history of values of the pointer.
///
/// The LRC allows cheap cloning like an Rc pointer,
/// but mutating the value via set will reallocate the smart pointer to point to the new value,
/// and have the new value point to the old one.
///
/// The advantage of the linked-list system - I don't know,
/// that came about as part of trying to make a more limited Rc.
/// It feels like accidental complexity at this point.
pub struct Lrc<T> {
    head: Option<NonNull<Node<T>>>
}

impl <T> Lrc<T> {
    fn new(element: T) -> Self {
        let node = Node::new(element);
        Lrc {
            head: Some(node.into_not_null())
        }
    }


    /// Sets a new value as the head, pushing the previous head to the second node in the list.
    pub fn set(&mut self, element: T) {
        self.push_head(Node::new(element))
    }

    /// Sets a new value for the head, and will remove any element that doesn't have any references.
    pub fn set_and_prune(&mut self, element: T) {
        self.set(element);
        self.prune_dead_nodes()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            let node = self.head.as_mut().unwrap().as_mut();
            if node.get_count() == 0 {
                Some(&mut node.element)
            } else {
                None
            }
        }
    }

    /// Removes nodes that don't have any references.
    pub fn prune_dead_nodes(&mut self) {
        unsafe {
            let mut current: NonNull<Node<T>> = self.head.unwrap();
            while let Some(mut next) = current.as_mut().next {
                let next_node: &mut Node<T> = next.as_mut();
                if next_node.get_count() < 1 {
                    // Tell the next next node (if it exists) to point its prev value at the current node
                    next_node.next.map(|mut next| next.as_mut().prev = Some(current));
                    // tell the current node's next to point to the next next node.
                    current.as_mut().next = next_node.next;
                } else {
                    current = next;
                }
            }
        }
    }

    /// Push a new node to the head of the Lrc.
    /// Because the head represents the active value for the Lrc,
    /// it effectively marks the old head for deletion if it wasn't already copied.
    fn push_head(&mut self, mut node: Node<T>) {
        node.next = self.head;
        node.prev = None; // TODO May not be necessary.
        let node = Some(node.into_not_null());

        unsafe {
            match self.head {
                None => {}
                Some(head) => {
                    (*head.as_ptr()).prev = node;
                    // Decrement the count when a node is moved away from the head position.
                    // Unless it is owned by a cloned lrc, this will mark it as dead, and it will be pruned
                    // the next time `prune_dead_nodes` is run.
                    (*head.as_ptr()).dec_count();
                },
            }
        }

        self.head = node;
    }

    fn get_count_head(&self) -> usize {
        unsafe {
            self.head.as_ref().unwrap().as_ref().get_count()
        }
    }

    // O(n) operation to determine how long the list is.
    fn len(&self) -> usize {
        let mut count = 1;

        unsafe {
            let mut node = self.head.as_ref().unwrap().as_ref();

            while let Some(next_node) = node.next.as_ref() {
                count += 1;
                node = next_node.as_ref()
            }
        }
        count
    }

    fn get_mut_head_node(&mut self) -> &mut Node<T> {
        unsafe {
            self.head.as_mut().unwrap().as_mut()
        }
    }

    fn get_ref_head_node(&self) -> &Node<T> {
        unsafe {
            self.head.as_ref().unwrap().as_ref()
        }
    }


}

impl <T: Clone> Lrc<T> {
    fn make_mut(&mut self) -> &mut T {
        // Use this to smuggle the copy past the borrow checker.
        if self.get_count_head() > 1 {
            // Clone to ensure unique ownership
            unsafe {
                let cloned_element = self.head.as_ref().unwrap().as_ref().element.clone();
                self.push_head(Node::new(cloned_element))
            }
        }
        unsafe {
            &mut self.head.as_mut().unwrap().as_mut().element
        }
    }
}

impl <T> Drop for Lrc<T> {
    fn drop(&mut self) {
        if let Some(mut head) = self.head {
            unsafe {

                // If its less than zero
                if head.as_ref().dec_count() {
                    // Attach surrounding nodes to each other as this one is removed.
                    head.as_mut().prev = head.as_ref().next.map(|next| next.as_ref().prev).and_then(std::convert::identity);
                    head.as_mut().next = head.as_ref().prev.map(|prev| prev.as_ref().next).and_then(std::convert::identity);

                    std::ptr::drop_in_place(head.as_mut());
                }
            }
        }
    }
}
impl <T> Clone for Lrc<T> {
    fn clone(&self) -> Self {
        if let Some(head) = self.head {
            unsafe {
                head.as_ref().inc_count();
            }
        }
        Lrc {
            head: self.head.clone()
        }
    }
}

impl <T: PartialEq> PartialEq for Lrc<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe{
            println!("Eq called");
            match (self.head, other.head) {
                (Some(lhs), Some(rhs)) => {
                    lhs.as_ref().eq(rhs.as_ref())
                }
                _ => false
            }
        }
    }
}

impl <T> AsRef<T> for Lrc<T> {
    fn as_ref(&self) -> &T {
        unsafe{
            &self.head.as_ref().unwrap().as_ref().element
        }
    }
}

impl <T> Deref for Lrc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{
            &self.head.as_ref().unwrap().as_ref().element
        }
    }
}

impl <T: fmt::Debug> fmt::Debug for Lrc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Lrc").field(&self.head).finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lrc_new() {
        let lrc = Lrc::new(25);
        assert_eq!(lrc, Lrc{head: Some(Node::new(25).into_not_null())});
        assert_eq!(lrc.as_ref(), &25)
    }

    #[test]
    fn clone_lrc() {
        let lrc = Lrc::new(25);
        let copy = lrc.clone();
        assert_eq!(copy.as_ref(), &25)
    }

    #[test]
    fn set_lrc() {
        let mut lrc = Lrc::new(25);
        lrc.set(30);

        assert_eq!(lrc.as_ref(), &30)
    }

    #[test]
    fn len_1() {
        let mut lrc = Lrc::new(25);
        lrc.set(24);
        assert_eq!(lrc.len(), 2);
    }

    #[test]
    fn len_2() {
        let mut lrc = Lrc::new(25);
        lrc.set(24);
        assert_eq!(lrc.len(), 2);
        lrc.prune_dead_nodes();
        assert_eq!(lrc.len(), 1);
    }

    #[test]
    fn set_prune() {
        let mut lrc = Lrc::new(25);
        lrc.set_and_prune(24);
        assert_eq!(lrc.len(), 1);
    }

}
