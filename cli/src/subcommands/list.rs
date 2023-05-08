use console::style;

use fimfic_tracker::{Story, StoryData};

use crate::args::{List, SortKey};
use crate::readable::ReadableDate;

macro_rules! sort_by_attr_funcs {
    ($(fn $name:ident (.$attr:ident) -> Ordering;)+) => {
        $(
            fn $name(a: &Story, b: &Story) -> std::cmp::Ordering {
                a.$attr.cmp(&b.$attr)
            }
        )+
    }
}

sort_by_attr_funcs! {
    fn sort_by_id(.id) -> Ordering;
    fn sort_by_title(.title) -> Ordering;
    fn sort_by_author(.author) -> Ordering;
    fn sort_by_chapters(.chapter_count) -> Ordering;
    fn sort_by_words(.words) -> Ordering;
    fn sort_by_update(.update_datetime) -> Ordering;
}

pub fn list(
    story_data: &StoryData,
    List {
        short,
        sort_by,
        reverse,
    }: List,
) {
    let mut stories = story_data.values().collect::<Vec<&Story>>();

    if let Some(sort) = sort_by {
        let sorter = match sort {
            SortKey::Id => sort_by_id,
            SortKey::Title => sort_by_title,
            SortKey::Author => sort_by_author,
            SortKey::Chapters => sort_by_chapters,
            SortKey::Words => sort_by_words,
            SortKey::Update => sort_by_update,
        };

        stories.sort_by(|a, b| sorter(a, b));
    }

    if reverse {
        stories.reverse();
    }

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
        stories
            .drain(..)
            .map(output_format)
            .collect::<Vec<String>>()
            .join(sep),
    );
}
