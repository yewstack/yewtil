/// Returns true if an assignment was made.
pub fn neq_assign<T: PartialEq>(assignee: &mut T, new: T) -> bool {
    if *assignee != new {
        *assignee = new;
        true
    } else {
        false
    }
}