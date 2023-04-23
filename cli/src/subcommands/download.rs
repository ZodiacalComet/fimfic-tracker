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
                            .context("Failed to launch overwrite confirmation prompt")
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

        let status_changed = story.status != updated_story.status;
        let story_update = story.compare_to(&updated_story)?;
        if story_update.is_some() || status_changed {
            set_printed!();

            if status_changed {
                clear_last_lines!();

                info!(
                    "{} has changed its status ({})",
                    format_story!(story),
                    format_update!(status, story.status => updated_story.status),
                );
                info_story_checking!(story);
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

    if force {
        for id in story_data
            .keys()
            .filter(|id| selected_ids.contains(id) && !ignored_ids.contains(id))
        {
            ids_to_download.insert(*id);
        }
    }

    if printed {
        separate!();
    }

    if ids_to_download.is_empty() {
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

    download_stories!(
        config,
        requester,
        story_data
            .iter()
            .filter(|(id, _)| ids_to_download.contains(id))
            .map(|(_, story)| story)
    );

    for (id, story) in updated_stories.drain() {
        story_data.insert(id, story);
    }

    Ok(())
}
