use regex::Regex;
use std::fs;
use std::path::Path;

pub enum EpisodeNumberMethod {
    FromFilename,
    FromFileOrder,
    FromLastNumbers,
}

pub enum EpisodeNameMethod {
    FromSecondPart,
    FromEpisodeNumber,
}

pub fn get_show_name(file_path: &Path) -> Option<String> {
    file_path.parent()?.file_name()?.to_str().map(String::from)
}

pub fn get_episode_number(
    method: &EpisodeNumberMethod,
    show_name: &str,
    file_path: &Path,
    root: &Path,
) -> i32 {
    match method {
        EpisodeNumberMethod::FromFilename => get_episode_number_from_filename(file_path)
            .unwrap_or_else(|| {
                eprintln!(
                    "Warning: Could not extract episode number from filename for {:?}. Using 0.",
                    file_path
                );
                0
            }),
        EpisodeNumberMethod::FromFileOrder => {
            get_episode_number_from_file_order(show_name, file_path, root)
        }
        EpisodeNumberMethod::FromLastNumbers => get_episode_number_from_last_numbers(file_path)
            .unwrap_or_else(|| {
                eprintln!(
                    "Warning: Could not extract episode number from last numbers for {:?}. Using 0.",
                    file_path
                );
                0
            }),
    }
}

pub fn get_episode_name(
    method: &EpisodeNameMethod,
    file_path: &Path,
    episode_number: i32,
) -> Option<String> {
    match method {
        EpisodeNameMethod::FromSecondPart => get_episode_name_from_second_part(file_path),
        EpisodeNameMethod::FromEpisodeNumber => Some(format!("Episode {}", episode_number)),
    }
}

fn get_episode_number_from_filename(file_path: &Path) -> Option<i32> {
    let file_name = file_path.file_stem()?.to_str()?;
    let re = Regex::new(r"E(\d+)").ok()?;
    re.captures(file_name)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn get_episode_number_from_file_order(show_name: &str, file_path: &Path, root: &Path) -> i32 {
    let show_dir = root.join(show_name);
    let mut episode_files: Vec<_> = fs::read_dir(show_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "srt" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    episode_files.sort();

    episode_files
        .iter()
        .position(|p| p == file_path)
        .map_or(0, |pos| pos as i32 + 1)
}

fn get_episode_number_from_last_numbers(file_path: &Path) -> Option<i32> {
    let file_name = file_path.file_stem()?.to_str()?;
    let re = Regex::new(r"(\d+)(?:[^0-9]*$)").ok()?;
    re.captures(file_name)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

fn get_episode_name_from_second_part(file_path: &Path) -> Option<String> {
    let file_name = file_path.file_name()?.to_str()?;
    let parts: Vec<&str> = file_name.split('.').collect();
    if parts.len() >= 3 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_show_name() {
        let path = PathBuf::from("/path/to/Show Name/episode.srt");
        assert_eq!(get_show_name(&path), Some("Show Name".to_string()));
    }

    #[test]
    fn test_get_episode_number_from_filename() {
        let path = PathBuf::from("ShowName.E05.Episode Title.srt");
        assert_eq!(get_episode_number_from_filename(&path), Some(5));
    }

    #[test]
    fn test_get_episode_number_from_last_numbers() {
        let path = PathBuf::from("ShowName.Episode Title.123.srt");
        assert_eq!(get_episode_number_from_last_numbers(&path), Some(123));
    }

    #[test]
    fn test_get_episode_name_from_second_part() {
        let path = PathBuf::from("ShowName.Episode Title.123.srt");
        assert_eq!(
            get_episode_name_from_second_part(&path),
            Some("Episode Title".to_string())
        );
    }

    // Note: Testing get_episode_number_from_file_order would require setting up a mock file system
    // or creating temporary files, which is beyond the scope of this simple test suite.
}
