use std::collections::VecDeque;
use std::ops::Deref;

// TODO when const generics lands, it would be useful to add a usize type parameter over the max
// number of elements.

// It would also be interesting to see if a diffing history implementation could be built over types
// that can represent themselves as diffs - where reversible transitions can be recorded instead of values
// and the transitions can be rolled back.

/// Keeps track of prior values.
pub struct History<T>(VecDeque<T>);

impl <T> History<T> {

    pub fn new(value: T) -> Self {
        let mut vec = VecDeque::new();
        vec.push_front(value);
        Self(vec)
    }

    /// Set the value represented by the `History` struct.
    ///
    /// This pushes the new value into the front of a list,
    /// where the front-most value represents the most recent value.
    pub fn set(&mut self, value: T) {
        self.0.push_front(value)
    }

    // TODO, maybe remove this, hist.set(create_new_value(hist)) is acceptable enough.
    /// Use a reference to the current value to determine what the next value will be.
    pub fn alter<F: Fn(&T)->T>(&mut self, f: F) {
        let current= self.as_ref();
        let value= f(current);
        self.set(value)
    }

    /// Removes all prior values.
    pub fn forget(&mut self) {
        self.0.drain(1..);
    }

    /// Remove all elements except the last one, making the oldest
    pub fn reset(&mut self) {
        self.0.drain(..self.0.len() - 1);
    }

    /// Returns true if there is more than one element in the list.
    pub fn dirty(&mut self) -> bool {
        self.0.len() > 1
    }

    /// Produces an iterator over references to history items ordered from newest to oldest.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.0.iter()
    }
}

impl <T> IntoIterator for History<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


impl <T> AsRef<T> for History<T> {
    fn as_ref(&self) -> &T {
        // Get the first element
        &self.0[0]
    }
}

impl <T> Deref for History<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}