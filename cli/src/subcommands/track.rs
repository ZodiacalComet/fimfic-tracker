use console::style;
use dialoguer::Confirm;

use fimfic_tracker::{Config, Result, Story, StoryData, TrackerError};

use crate::args::Track;
use crate::Requester;

pub fn track(
    config: &Config,
    requester: &Requester,
    story_data: &mut StoryData,
    Track {
        overwrite,
        skip_download,
        ref stories,
    }: Track,
) -> Result<()> {
    let mut to_track: Vec<&String> = Vec::with_capacity(stories.len());
    let mut printed = false;

    for id in stories {
        if let Some(story) = story_data.get(id) {
            if !printed {
                printed = true;
            }

            if overwrite {
                info!(
                    "{} is already on the tracking list. Overwriting.",
                    format_story!(story)
                );
            } else {
                let confirm = Confirm::new()
                    .with_prompt(format!(
                        "{} is already on the tracking list. Do you want to overwrite it?",
                        format_story!(story)
                    ))
                    .interact()
                    .map_err(|err| {
                        TrackerError::io(err)
                            .context("Failed to launch overwrite confirmation prompt")
                    })?;

                if !confirm {
                    continue;
                }
            }
        }

        to_track.push(id);
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

        story_data.insert(id.to_string(), story.clone());

        clear_last_lines!();
        info!("{} added to the tracking list", format_story!(story));

        stories.push(story);
    }

    if skip_download {
        return Ok(());
    }

    separate!();
    download_stories!(config, requester, stories.drain(..));

    Ok(())
}
