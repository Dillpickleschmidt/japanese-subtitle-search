use super::episode_info::{
    get_episode_name, get_episode_number, get_show_name, EpisodeNameMethod, EpisodeNumberMethod,
};
use super::errors::ParsingError;
use super::types::{Subtitle, Subtitles, Timestamp};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use walkdir::WalkDir;

pub struct SrtEntry {
    pub show_name: String,
    pub episode_name: String,
    pub episode_number: i32,
    pub content: Subtitles,
}

pub fn process_srt_directory(
    root_dir: &Path,
    number_method: &EpisodeNumberMethod,
    name_method: &EpisodeNameMethod,
) -> HashMap<String, Vec<SrtEntry>> {
    let mut show_entries: HashMap<String, Vec<SrtEntry>> = HashMap::new();

    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "srt") {
            println!("Processing {:?}...", path.file_name().unwrap());
            match process_srt_file(path, root_dir, number_method, name_method) {
                Ok(srt_entry) => {
                    show_entries
                        .entry(srt_entry.show_name.clone())
                        .or_insert_with(Vec::new)
                        .push(srt_entry);
                }
                Err(e) => eprintln!("Error processing file {:?}: {}", path, e),
            }
        }
    }

    // Sort episodes for each show
    for entries in show_entries.values_mut() {
        entries.sort_by_key(|entry| entry.episode_number);
    }

    show_entries
}

pub fn process_srt_file(
    file_path: &Path,
    root: &Path,
    number_method: &EpisodeNumberMethod,
    name_method: &EpisodeNameMethod,
) -> Result<SrtEntry, ParsingError> {
    let show_name = get_show_name(file_path).unwrap_or_else(|| "Unknown Show".to_string());
    let episode_number = get_episode_number(number_method, &show_name, file_path, root);
    let episode_name = get_episode_name(name_method, file_path, episode_number)
        .unwrap_or_else(|| format!("Episode {}", episode_number));

    let content = Subtitles::parse_from_file(file_path)?;

    Ok(SrtEntry {
        show_name,
        episode_name,
        episode_number,
        content,
    })
}

impl Subtitles {
    /// Parses a string containing SRT formatted subtitles into a `Subtitles` struct.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the SRT formatted subtitles
    ///
    /// # Returns
    ///
    /// * `Result<Self, ParsingError>` - Parsed subtitles or an error
    pub fn parse_from_str(input: &str) -> Result<Self, ParsingError> {
        // Remove BOM if present and normalize line endings
        let input = input.trim_start_matches('\u{feff}').replace('\r', "");

        // Define regex pattern for parsing SRT format
        // Detailed explanation of the regex pattern:
        // r"(\d+)\n                     - Group 1: Matches the subtitle number (one or more digits) followed by a newline
        //   (\d{2}:\d{2}:\d{2},\d{3})   - Group 2: Matches the start time (HH:MM:SS,mmm format)
        //   -->                         - Matches the arrow separator between timestamps
        //   (\d{2}:\d{2}:\d{2},\d{3})   - Group 3: Matches the end time (HH:MM:SS,mmm format)
        //   \n                          - Matches the newline after the timestamp line
        //   ((?s:.*?)                   - Group 4: Starts the subtitle text capture
        //     (?s:.*?)                    - Non-greedy match of any characters, including newlines (s flag)
        //   (?:\n\n|$))                 - End of Group 4: Matches either two newlines or the end of the string
        //                                 This allows for multi-line subtitles and handles the last subtitle"
        let re = Regex::new(
            r"(\d+)\n(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})\n((?s:.*?)(?:\n\n|$))",
        )
        .map_err(|_| ParsingError::MalformedSubtitle)?;

        let mut subtitles = Vec::new();

        // Iterate over each regex match in the input
        for cap in re.captures_iter(&input) {
            // Parse subtitle number (Group 1)
            let number = cap[1].parse().map_err(|_| ParsingError::InvalidNumber)?;

            // Parse start timestamp (Group 2)
            let start_time = Timestamp::from_str(&cap[2])?;

            // Parse end timestamp (Group 3)
            let end_time = Timestamp::from_str(&cap[3])?;

            // Extract and trim subtitle text (Group 4)
            let text = cap[4].trim().to_string();

            // Debug output - consider removing in production
            // println!("Number: {}", number);
            // println!("Start time: {:?}", start_time);
            // println!("End time: {:?}", end_time);
            // println!("Text: {}", text);

            // Create and add new Subtitle to the collection
            subtitles.push(Subtitle {
                number,
                start_time,
                end_time,
                text,
            });
        }

        // Check if any subtitles were parsed
        if subtitles.is_empty() {
            Err(ParsingError::MalformedSubtitle)
        } else {
            Ok(Subtitles(subtitles))
        }
    }

    pub fn parse_from_file(path: &Path) -> Result<Self, ParsingError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Self::parse_from_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_from_str() {
        let input = "1\n00:00:01,000 --> 00:00:04,000\nHello, world!\n\n2\n00:00:05,000 --> 00:00:07,000\nThis is a test.";
        let subtitles = Subtitles::parse_from_str(input).unwrap();
        assert_eq!(subtitles.len(), 2);
        assert_eq!(subtitles.0[0].number, 1);
        assert_eq!(subtitles.0[0].text, "Hello, world!");
        assert_eq!(subtitles.0[1].number, 2);
        assert_eq!(subtitles.0[1].text, "This is a test.");
    }

    #[test]
    fn test_process_srt_file() {
        // This test would require a mock file system or test SRT files
        // Implement based on your testing strategy
    }

    #[test]
    fn test_process_srt_directory() {
        // This test would require a mock file system or test SRT files
        // Implement based on your testing strategy
    }
}
