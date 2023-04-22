use console::style;

use fimfic_tracker::StoryData;

use crate::args::Untrack;

pub fn untrack(story_data: &mut StoryData, Untrack { ref ids }: Untrack) {
    for id in ids {
        match story_data.shift_remove(id) {
            Some(story) => info!("{} untracked", format_story!(story)),
            None => warn!(
                "There is no story of ID {} on the tracking list.",
                style(id).bold(),
            ),
        };
    }
}
