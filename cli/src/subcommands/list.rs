use console::style;

use fimfic_tracker::{Story, StoryData};

use crate::args::List;
use crate::readable::ReadableDate;

pub fn list(story_data: &StoryData, List { short }: List) {
    let output_format = if short {
        |story: &Story| {
            format!(
                "{} {}",
                style(format_args!("{}", story.id)).blue(),
                style(&story.title).green()
            )
        }
    } else {
        |story: &Story| {
            [
                format!("{}", style(format_args!("[{}]", story.id)).blue().bold()),
                format!("url = {}", style(story.url()).cyan()),
                format!("title = {}", style(&story.title).green()),
                format!("author = {}", style(&story.author).green()),
                format!("chapter-amt = {}", style(story.chapter_count).blue()),
                format!("words = {}", style(story.words).blue()),
                format!(
                    "last-update-date = {}",
                    style(ReadableDate(story.update_datetime)).yellow()
                ),
                format!("status = {}", format_status!(story)),
            ]
            .join("\n")
        }
    };

    let sep = if short { "\n" } else { "\n\n" };

    println!(
        "{}",
        story_data
            .values()
            .map(output_format)
            .collect::<Vec<String>>()
            .join(sep),
    );
}
