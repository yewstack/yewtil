use std::collections::VecDeque;
use std::ops::Deref;

// TODO when const generics lands, it would be useful to add a usize type parameter over the max number of elements.

// It would also be interesting to see if a diffing history implementation could be built over types
// that can represent themselves as diffs - where reversible transitions can be recorded instead of values
// and the transitions can be rolled back.
// That would probably have worse performance in exchange for smaller size.

/// Keeps track of prior values.
///
/// It keeps values that have been `set` for it around for the duration of its lifetime,
/// or until they are dropped by calling `reset` or `forget`.
///
/// Prior values can be iterated over as well.
pub struct History<T>(VecDeque<T>);

impl<T> History<T> {
    /// Creates a new history wrapper.
    pub fn new(value: T) -> Self {
        let mut vec = VecDeque::new();
        vec.push_front(value);
        Self(vec)
    }

    /// Set the value represented by the `History` struct.
    ///
    /// This pushes the new value into the front of a list,
    /// where the front-most value represents the most recent value.
    ///
    /// # Example
    /// ```
    ///# use yewtil::History;
    /// let mut history = History::new(0);
    /// history.set(1);
    /// assert_eq!(*history, 1);
    /// ```
    pub fn set(&mut self, value: T) {
        self.0.push_front(value)
    }

    /// Replaces the current value without creating a history entry.
    pub fn replace(&mut self, value: T) {
        self.0[0] = value;
    }

    /// Removes all prior values.
    ///
    /// The returned bool indicates if any elements were removed.
    ///
    /// # Example
    /// ```
    ///# use yewtil::History;
    /// let mut history = History::new(0);
    /// history.set(1);
    /// history.set(2);
    ///
    /// history.forget();
    /// assert_eq!(*history, 2);
    /// assert_eq!(history.count(), 1);
    /// ```
    pub fn forget(&mut self) -> bool {
        if self.dirty() {
            self.0.drain(1..);
            true
        } else {
            false
        }
    }

    /// Remove all elements except the last one, making the oldest
    ///
    /// The returned bool indicates if any elements were removed.
    ///
    /// # Example
    /// ```
    ///# use yewtil::History;
    /// let mut history = History::new(0);
    /// history.set(1);
    /// history.set(2);
    ///
    /// history.reset();
    /// assert_eq!(*history, 0);
    /// assert_eq!(history.count(), 1);
    /// ```
    pub fn reset(&mut self) -> bool {
        if self.dirty() {
            self.0.drain(..self.0.len() - 1);
            true
        } else {
            false
        }
    }

    /// Returns true if there is more than one element in the list.
    ///
    /// # Example
    /// ```
    ///# use yewtil::History;
    /// let mut history = History::new(0);
    /// history.set(1);
    /// assert!(history.dirty());
    /// ```
    pub fn dirty(&mut self) -> bool {
        self.count() > 1
    }

    /// Returns the number of entries in the history.
    ///
    /// This will never be less than 1, as the first item is used
    ///
    /// # Example
    /// ```
    ///# use yewtil::History;
    /// let mut history = History::new(0);
    /// assert_eq!(history.count(), 1);
    ///
    /// history.set(1);
    /// assert_eq!(history.count(), 2);
    /// ```
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Produces an iterator over references to history items ordered from newest to oldest.
    pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.0.iter()
    }

    /// Gets the current value.
    pub fn into_inner(mut self) -> T {
        self.0
            .pop_front()
            .expect("History should have at least one item")
    }
}

impl <T: PartialEq> History<T> {
    pub fn neq_set(&mut self, value: T) -> bool {
        if self.0[0] != value {
            self.set(value);
            true
        } else {
            false
        }
    }
}

impl<T> IntoIterator for History<T> {
    type Item = T;
    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> AsRef<T> for History<T> {
    fn as_ref(&self) -> &T {
        // Get the first element
        &self.0[0]
    }
}

impl<T> Deref for History<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
