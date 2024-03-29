use std::collections::{HashMap, HashSet};

use console::style;
use dialoguer::Confirm;

use fimfic_tracker::{
    Config, Id, Result, SensibilityLevel, Story, StoryData, StoryStatus, StoryUpdate, TrackerError,
};

use crate::args::{Download, Prompt};
use crate::readable::ReadableDate;
use crate::Requester;

macro_rules! format_update {
    (author, $before:expr => $after:expr) => {
        format_update!([green] &$before, &$after)
    };
    (chapters, $before:expr => $after:expr) => {
        format_update!([blue] $before, $after)
    };
    (words, $before:expr => $after:expr) => {
        format_update!([blue] $before, $after)
    };
    (timestamp, $before:expr => $after:expr) => {
        format_update!([yellow] ReadableDate($before), ReadableDate($after))
    };
    (status, $before:expr => $after:expr) => {
        format_update!([yellow] $before, $after)
    };
    ([$color:ident] $before:expr, $after:expr) => {
        format_args!(
            "{} {} {}",
            style($before).$color(),
            style("=>").cyan(),
            style($after).$color().bold()
        )
    };
}

macro_rules! info_story_checking {
    ($story:expr) => {
        info!("Checking for {} ...", format_story!($story));
    };
}

macro_rules! info_update {
    ([ignored] $story:expr, $on:ident, $before:expr => $after:expr) => {
        info_update!($story, $on, $before, $after, ". Ignoring")
    };
    ($story:expr, $on:ident, $before:expr => $after:expr) => {
        info_update!($story, $on, $before, $after, "")
    };
    ($story:expr, $on:ident, $before:expr, $after:expr, $extra:expr) => {
        info!(
            "{} has an update on {} ({}){}",
            format_story!($story),
            stringify!($on),
            format_update!($on, $before => $after),
            $extra
        );
    };
}

#[derive(Debug)]
enum StoryDownload {
    Update(Id, Story),
    Forced(Id),
}

pub fn download(
    config: &Config,
    requester: &Requester,
    story_data: &mut StoryData,
    Download {
        force,
        prompt,
        ref ids,
    }: Download,
) -> Result<()> {
    let selected_ids: Vec<Id> = if ids.is_empty() {
        story_data.keys().cloned().collect()
    } else {
        story_data
            .keys()
            .filter(|id| ids.contains(id))
            .cloned()
            .collect()
    };
    let mut ignored_ids: HashSet<Id> = HashSet::with_capacity(selected_ids.len());

    let mut printed = false;

    macro_rules! set_printed {
        () => {
            if !printed {
                printed = true;
            }
        };
    }

    for (id, story) in story_data.iter().filter_map(|(id, story)| {
        if selected_ids.contains(id) {
            Some((*id, story))
        } else {
            None
        }
    }) {
        if let StoryStatus::Incomplete = story.status {
            continue;
        }

        set_printed!();

        let status_notice = format!(
            "{} has been marked as {} by the author",
            format_story!(story),
            format_status!(story)
        );

        match prompt {
            Prompt::AssumeYes => {
                info!("{}. Checking for an update on it anyways.", status_notice);
            }
            Prompt::AssumeNo => {
                info!("{}. Skipping checking for an update on it.", status_notice);
                ignored_ids.insert(id);
            }
            Prompt::Ask => {
                let confirm = Confirm::new()
                    .with_prompt(format!(
                        "{}. Do you want to still check for an update on it?",
                        status_notice
                    ))
                    .interact()
                    .map_err(|err| {
                        TrackerError::io(err)
                            .context("failed to launch overwrite confirmation prompt")
                    })?;

                if !confirm {
                    ignored_ids.insert(id);
                }
            }
        }
    }

    if printed {
        separate!();
        printed = false;
    }

    let mut updated_stories: HashMap<Id, Story> = HashMap::with_capacity(selected_ids.len());
    let mut ids_to_download: HashSet<Id> = HashSet::with_capacity(selected_ids.len());

    for (id, story) in story_data
        .iter()
        .filter(|(id, _)| selected_ids.contains(id) && !ignored_ids.contains(id))
        .map(|(id, story)| (*id, story))
    {
        info_story_checking!(story);
        let updated_story: Story = requester.get_story_response(id)?.into();

        let title_changed = story.title != updated_story.title;
        let author_changed = story.author != updated_story.author;
        let status_changed = story.status != updated_story.status;
        let story_update = story.compare_to(&updated_story)?;

        if story_update.is_some() || title_changed || author_changed || status_changed {
            // If we are here, something will be printed to stderr. Be it by the specific cases
            // just below or by the resulting StoryUpdate comparison.
            set_printed!();

            if title_changed || author_changed || status_changed {
                clear_last_lines!();

                if title_changed {
                    info!(
                        "{} has changed its title to {}",
                        format_story!(story),
                        style(&updated_story.title).green().bold()
                    );
                }

                if author_changed {
                    info!(
                        "{} has changed its author ({})",
                        format_story!(story),
                        format_update!(author, story.author => updated_story.author)
                    );
                }

                if status_changed {
                    info!(
                        "{} has changed its status ({})",
                        format_story!(story),
                        format_update!(status, story.status => updated_story.status),
                    );
                }

                // Avoid this message from being repeated twice in verbose output.
                if verbose_disabled!() {
                    info_story_checking!(story);
                }
            }

            updated_stories.insert(id, updated_story);
        }

        clear_last_lines!();

        match story_update {
            Some(StoryUpdate::Chapters { before, after }) => {
                info_update!(story, chapters, before => after);
            }
            Some(StoryUpdate::Words { before, after })
                if config.sensibility_level >= SensibilityLevel::IncludeWords =>
            {
                info_update!(story, words, before => after);
            }
            Some(StoryUpdate::DateTime { before, after })
                if config.sensibility_level == SensibilityLevel::Anything =>
            {
                info_update!(story, timestamp, before => after);
            }
            Some(StoryUpdate::Words { before, after }) => {
                info_update!([ignored] story, words, before => after);
                continue;
            }
            Some(StoryUpdate::DateTime { before, after }) => {
                info_update!([ignored] story, timestamp, before => after);
                continue;
            }
            None => continue,
        };

        ids_to_download.insert(id);
    }

    // Update stories with ignored updates.
    // This way if the downloads fail, these should be saved by the "emergency save".
    //
    // After this block, `updated_stories` should only contain stories whose IDs are in
    // `ids_to_download`.
    {
        let mut updated_ids = story_data
            .keys()
            .filter(|id| !ids_to_download.contains(id))
            .filter_map(|id| updated_stories.remove_entry(id))
            .collect::<Vec<(Id, Story)>>();

        debug!("Ignored updates: {:?}", &updated_ids);

        for (id, story) in updated_ids.drain(..) {
            story_data.insert(id, story);
        }
    }

    if printed {
        separate!();
    }

    if !force && ids_to_download.is_empty() {
        info!("There is nothing to download");
    } else if force {
        progress_or_info!(
            "{}",
            style(format!(
                "Force downloading {}",
                if ids.is_empty() && ignored_ids.is_empty() {
                    "every story on the tracking list"
                } else {
                    "selected stories"
                }
            ))
            .bold(),
        );

        separate!();
    }

    let use_separator = config.exec.is_some() && !config.quiet;
    let delay = std::time::Duration::from_secs(config.download_delay);

    let mut stories_to_download: Vec<StoryDownload> = story_data
        .keys()
        // Only download the stories that:
        // (1) Whose IDs were given by the user if any.
        // (2) The user responded to its prompt with Y.
        .filter(|id| selected_ids.contains(id) && !ignored_ids.contains(id))
        // Download all stories if the user forced it, otherwise only those who passed the update
        // sensibility test.
        .filter(|id| force || ids_to_download.contains(id))
        .map(|id| match updated_stories.remove(id) {
            Some(story) => StoryDownload::Update(*id, story),
            None => StoryDownload::Forced(*id),
        })
        .collect();

    debug!("Stories to download: {:?}", &stories_to_download);

    for (is_first, story_download) in stories_to_download
        .drain(..)
        .enumerate()
        .map(|(index, story_download)| (index == 0, story_download))
    {
        download_delay!(!is_first, use_separator, delay);

        match &story_download {
            StoryDownload::Update(_, story) => requester.download(story)?,
            // While this should be safe to unwrap, in the unlikely event that it panics the
            // "emergency save" would be skipped.
            // So I throw in a `match` to "safely" unwrap it and throw a warning if it is not
            // present.
            StoryDownload::Forced(id) => match story_data.get(id) {
                Some(story) => requester.download(story)?,
                None => warn!("{} is not present in the tracker file.", id),
            },
        };

        // Insert the update once it downloads.
        if let StoryDownload::Update(id, story) = story_download {
            story_data.insert(id, story);
        }
    }

    Ok(())
}
