pub enum Action {
    Insert(usize, char, usize), // Insert(position, character, end_position)
    Newline(usize, usize), // Newline(position, end_position
    Delete(usize, char, usize), // Delete(position, deleted_character, end_position)
    Replace(usize, String, String), // Replace(range_left, replaced_string, replace_string)
    Cut(usize, String, usize), // Cut(range_left, cut_string, end_position)
    Paste(usize, String, usize) // Paste(start_position, pasted_string, end_position)
}

pub enum ActionGroup {
    // This allows us to group multiple actions together
    // e.g. tab actions all into one
    Singleton(Action),
    Multiple(Vec<Action>)
}

pub fn merge_action_groups(action_groups: Vec<ActionGroup>) -> ActionGroup {
    // Merges mutliple action groups into a single action group
    let mut flattened_actions = Vec::<Action>::new();
    for action_group in action_groups {
        match action_group {
            ActionGroup::Singleton(action) => {
                flattened_actions.push(action);
            },
            ActionGroup::Multiple(actions) => {
                for action in actions {
                    flattened_actions.push(action);
                }
            }
        }
    }
    ActionGroup::Multiple(flattened_actions)

}
