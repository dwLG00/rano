pub enum Action {
    TypeChar(usize, char, usize), // TypeChar(position, character, end_position)
    Newline(usize, usize), // Newline(position, end_position
    Delete(usize, char, usize), // Delete(position, deleted_character, end_position)
    Replace(usize, String, String), // Replace(range_left, replaced_string, replace_string)
    Cut(usize, usize, String, usize), // Cut(start_position, range_left, cut_string, end_position)
    Insert(usize, usize, String, usize) // Insert(start_position, range_l, pasted_string, end_position)
}

pub enum ActionGroup {
    // This allows us to group multiple actions together
    // e.g. tab actions all into one
    Singleton(Action),
    Multiple(Vec<Action>)
}

impl Action {
    pub fn undo(&self) -> Action {
        match self {
            Self::TypeChar(pos, ch, end) => Self::Delete(*end, *ch, *pos),
            Self::Newline(pos, end) => Self::Delete(*end, '\n', *pos),
            Self::Delete(pos, ch, end) => match *ch {
                '\n' => Self::Newline(*end, *pos),
                _ => Self::TypeChar(*end, *ch, *pos)
            },
            Self::Replace(range_l, replaced, replacing) => Self::Replace(*range_l, replacing.clone(), replaced.clone()),
            Self::Cut(start, range_l, cut_string, end) => Self::Insert(*end, *range_l, cut_string.clone(), *start),
            Self::Insert(start, range_l, paste_string, end) => Self::Cut(*end, *range_l, paste_string.clone(), *start)
        }
    }
}

impl ActionGroup {
    pub fn undo(&self) -> ActionGroup {
        match self {
            Self::Singleton(action) => Self::Singleton(action.undo()),
            Self::Multiple(action_vector) => Self::Multiple(action_vector.iter().rev().map(|a| a.undo()).collect())
        }
    }
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
