use std::{io, str::FromStr, fmt::Display};

use anyhow::{Result, Ok};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Debug, DeserializeFromStr, SerializeDisplay)]
struct TagList(Box<[Box<str>]>);

impl FromStr for TagList {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.trim().is_empty() {
            return Ok(Self(Box::new([])));
        }
        let mut tags: Vec<_> = s.split(',')
            .map(|tag| tag.trim().replace(' ', "_").into())
            .collect();
        tags.sort();
        tags.dedup();
        Ok(Self(tags.into()))
    }
}

impl Display for TagList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

impl TagList {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn contains(&self, tag: &str) -> bool {
        self.0.iter().any(|t| &**t == tag)
    }

    fn filter<F>(&self, f: F) -> Self
    where
        F: Fn(&str) -> bool
    {
        TagList(self.0.iter()
            .filter(|&tag| f(tag))
            .cloned()
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct LuteTerm {
    term: String,
    parent: Option<String>,
    translation: String,
    language: String,
    pronunciation: Option<String>,
    tags: TagList,
}

#[derive(Debug, Serialize)]
struct AnkiNote {
    front: String,
    back: String,
    tags: TagList,
    deck: String,
}

fn convert(lt: LuteTerm) -> Option<AnkiNote> {
    if (lt.parent.is_some() || lt.tags.contains("generated")) && !lt.tags.contains("anki") {
        None?;
    }
    if lt.tags.contains("noanki") {
        None?;
    }
    if lt.translation.trim().is_empty() {
        None?;
    }
    let tags: TagList = lt.tags.filter(|tag| tag != "loan" && tag != "anki" && tag != "generated");
    let mut front: String = lt.term.replace('\u{200b}', "");
    if let Some(p) = lt.pronunciation {
        front += &format!("\n\npronunciation: {p}");
    }
    let mut back: String = lt.translation;
    if !tags.is_empty() {
        back += &format!("\n\ntags: {tags}");
    }
    let deck = lt.language;

    Some(AnkiNote { deck, front, back, tags })
}

fn main() -> Result<()> {
    println!("#separator: Comma");
    println!("#html: false");
    println!("#columns: Front,Back,Tags,Deck");
    println!("#notetype: Basic (and reversed card)");
    println!("#deck column: 4");
    println!("#tags column: 3");
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut wtr = csv::WriterBuilder::new().has_headers(false).from_writer(io::stdout());
    for rec in rdr.deserialize() {
        if let Some(note) = convert(rec?) {
            wtr.serialize(note)?;
        }
    }
    wtr.flush()?;
    Ok(())
}
