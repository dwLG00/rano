pub enum Action {
    Insert(usize, char, usize), // Insert(position, character, end_position)
    Newline(usize, usize), // Newline(position, end_position
    Delete(usize, usize), // Delete(position, end_position)
    Replace(usize, usize, String, usize), // Replace(range_left, range_right, string, end_position)
    Cut(usize, usize, usize) // Cut(range_left, range_right, end_position)
}

pub enum ActionGroup {
    // This allows us to group multiple actions together
    // e.g. tab actions all into one
    Singleton(Action),
    Multiple(Vec<Action>)
}
