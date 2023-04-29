use console::style;
use dialoguer::Confirm;

use fimfic_tracker::{Config, Id, Result, Story, StoryData, TrackerError};

use crate::args::Track;
use crate::Requester;

pub fn track(
    config: &Config,
    requester: &Requester,
    story_data: &mut StoryData,
    Track {
        overwrite,
        skip_download,
        ref ids,
    }: Track,
) -> Result<()> {
    let mut to_track: Vec<Id> = Vec::with_capacity(ids.len());
    let mut printed = false;

    for id in ids {
        if let Some(story) = story_data.get(id) {
            if !printed {
                printed = true;
            }

            let story_notice = format!("{} is already on the tracking list", format_story!(story));

            if overwrite {
                info!("{}. Overwriting.", story_notice);
            } else {
                let confirm = Confirm::new()
                    .with_prompt(format!("{}. Do you want to overwrite it?", story_notice))
                    .interact()
                    .map_err(|err| {
                        TrackerError::io(err)
                            .context("failed to launch overwrite confirmation prompt")
                    })?;

                if !confirm {
                    continue;
                }
            }
        }

        to_track.push(*id);
    }

    if to_track.is_empty() {
        return Ok(());
    }

    if printed {
        separate!();
    }

    let mut stories: Vec<Story> = Vec::with_capacity(to_track.len());

    for id in to_track {
        progress_or_info!("Downloading story data for {}", style(id).blue());
        let story: Story = requester.get_story_response(id)?.into();

        story_data.insert(id, story.clone());

        clear_last_lines!();
        info!("{} added to the tracking list", format_story!(story));

        stories.push(story);
    }

    if skip_download {
        return Ok(());
    }

    separate!();

    // By this point, all stories given for tracking are already inserted into the tracking list.
    // This would make it so that if an error were to happen here, they would still be saved.
    // That seems like a pretty good behavior.
    let use_separator = config.exec.is_some() && !config.quiet;
    let delay = std::time::Duration::from_secs(config.download_delay);

    for (is_first, story) in stories
        .drain(..)
        .enumerate()
        .map(|(index, story)| (index == 0, story))
    {
        download_delay!(!is_first, use_separator, delay);
        requester.download(&story)?;
    }

    Ok(())
}
