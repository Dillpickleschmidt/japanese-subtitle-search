/*
1. Read all transcripts_raw files and add entries to sqlite database transcripts table following example.json
2. Create a reverse index by extracting all unique words from the transcripts_raw files storing them in a new
table called words. Find the matching show_name, episode_number, and time_start for each word and store the
found transcript_id belonging to those properties in an additional column in the words table as a foreign key.
3. Create a parser that tokenizes Japanese text.
4. Create a search function that uses the parsed text and finds transcript lines that contain all parsed words
using the reverse index.
  4.1. Get the show_name, season, episode_number, line_id, & transcript_id of the parsed text.
  4.2. Using the show_name, season, episide_number, and line_id, get the text for 5 previous lines (if they
  exist) and 2 next lines. Output a json object with the following structure:
[
  "item 1": [
    {
      "id": "3476"
      "ts_num": "-5",
      "text": "text",
    },
    {
      "id": "3477"
      "ts_num": "-4",
      "text": "text"
    },
    ...
    {
      "id": "3481"
      "ts_num": "0",
      "text": "text"
    },
    {
      "id": "3482"
      "ts_num": "1",
      "text": "text"
    },
    ...
  ],
  ...
]
5. Feed the LLM with the output of the search function. Tell it it's looking for the 10 most interesting/memorable
ts_num 0 lines, returned in descending order of interest/memorability. Use the surrounding lines as context in
determening that. Return just the ids in your output.
6. Search the transcripts table for those 10 ids and return the full text of the matching lines.
*/

mod db;
mod srt_parser;
use db::DbHandler;
use rusqlite::Result;
use srt_parser::{process_srt_directory, EpisodeNameMethod, EpisodeNumberMethod};
use std::path::Path;

fn main() -> Result<()> {
    let mut db = DbHandler::new("transcripts.db")?;
    db.create_tables()?;

    let root_dir = Path::new("data/transcripts_raw");
    let number_method = EpisodeNumberMethod::FromFileOrder;
    let name_method = EpisodeNameMethod::FromEpisodeNumber;

    let show_entries = process_srt_directory(root_dir, &number_method, &name_method);
    println!(
        "Processed {} entries.",
        show_entries.values().flatten().count()
    );

    // Prepare data for batch insertion
    let mut shows = Vec::new();
    let mut episodes = Vec::new();
    let mut transcripts = Vec::new();

    for (show_name, show_episodes) in show_entries {
        let show_id = (shows.len() + 1) as i64;
        shows.push((show_name.clone(), "Anime".to_string()));

        for episode in show_episodes {
            let episode_id = (episodes.len() + 1) as i64;
            episodes.push((
                show_id,
                episode.episode_name.clone(),
                1, // Assuming all episodes are in season 1
                episode.episode_number as i32,
            ));

            for (index, subtitle) in episode.content.0.iter().enumerate() {
                transcripts.push((
                    episode_id,
                    (index + 1) as i32,
                    subtitle.start_time.to_string(),
                    subtitle.end_time.to_string(),
                    subtitle.text.clone(),
                ));
            }
        }
    }

    // Perform batch insertions
    db.batch_insert_shows(&shows)?;
    db.batch_insert_episodes(&episodes)?;
    db.batch_insert_transcripts(&transcripts)?;

    println!("All data has been inserted into the database.");
    Ok(())
}
