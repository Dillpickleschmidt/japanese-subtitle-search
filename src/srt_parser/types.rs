use super::errors::ParsingError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timestamp {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub milliseconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subtitle {
    pub number: usize,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct Subtitles(pub Vec<Subtitle>);

impl Timestamp {
    pub fn new(hours: u32, minutes: u32, seconds: u32, milliseconds: u32) -> Self {
        Timestamp {
            hours,
            minutes,
            seconds,
            milliseconds,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            self.hours, self.minutes, self.seconds, self.milliseconds
        )
    }
}

impl FromStr for Timestamp {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(&[':', ',']).collect();
        if parts.len() != 4 {
            return Err(ParsingError::InvalidTimestamp);
        }

        Ok(Timestamp {
            hours: parts[0]
                .parse()
                .map_err(|_| ParsingError::InvalidTimestamp)?,
            minutes: parts[1]
                .parse()
                .map_err(|_| ParsingError::InvalidTimestamp)?,
            seconds: parts[2]
                .parse()
                .map_err(|_| ParsingError::InvalidTimestamp)?,
            milliseconds: parts[3]
                .parse()
                .map_err(|_| ParsingError::InvalidTimestamp)?,
        })
    }
}

impl Subtitle {
    pub fn new(number: usize, start_time: Timestamp, end_time: Timestamp, text: String) -> Self {
        Subtitle {
            number,
            start_time,
            end_time,
            text,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}\n{} --> {}\n{}",
            self.number,
            self.start_time.to_string(),
            self.end_time.to_string(),
            self.text
        )
    }
}

impl Subtitles {
    pub fn new() -> Self {
        Subtitles(Vec::new())
    }

    pub fn push(&mut self, subtitle: Subtitle) {
        self.0.push(subtitle);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Subtitle> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Subtitle> {
        self.0.iter_mut()
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|subtitle| subtitle.to_string())
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

impl IntoIterator for Subtitles {
    type Item = Subtitle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Subtitles {
    type Item = &'a Subtitle;
    type IntoIter = std::slice::Iter<'a, Subtitle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Subtitles {
    type Item = &'a mut Subtitle;
    type IntoIter = std::slice::IterMut<'a, Subtitle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
