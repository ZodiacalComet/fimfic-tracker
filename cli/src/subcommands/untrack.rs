use console::style;

use fimfic_tracker::StoryData;

use crate::args::Untrack;

pub fn untrack(story_data: &mut StoryData, Untrack { ref ids }: Untrack) {
    for id in ids {
        match story_data.shift_remove(id) {
            Some(story) => info!("{} untracked", format_story!(story)),
            None => warn!(
                "There is no story of ID {}{}",
                style(id).bold(),
                // Any created style ends with a complete reset on formatting, which resets the
                // yellow foreground for warning messages even though we only wanted to bold the
                // story ID.
                // The `console` crate doesn't have a mechanism in place to change this behavior as
                // of now, so the foreground needs to be reapplied.
                style(" on the tracking list.").yellow()
            ),
        };
    }
}
