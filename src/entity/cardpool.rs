use crate::entity::{TagKey, TagVal};
use serde_json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::io::ErrorKind;

/// This is all the data needed to create a card
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PooledCardData {
    name: String,
    text: String,
    script: String,
    tags: HashMap<TagKey, TagVal>,
}
impl PooledCardData {
    pub fn new(name: &str) -> PooledCardData {
        PooledCardData {
            name: String::from(name),
            text: String::new(),
            script: String::from("none"),
            tags: HashMap::with_capacity(8),
        }
    }

    #[inline]
    pub fn clone_tags(&self) -> HashMap<TagKey, TagVal> {
        self.tags.clone()
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[inline]
    pub fn script(&self) -> &str {
        &self.script
    }
}

lazy_static! {
    static ref INSTANCE: CardPool = CardPool::from_disk().expect("Failed to load card pool.");
}
pub struct CardPool {
    by_name: HashMap<String, PooledCardData>,
}

impl CardPool {
    pub fn lookup_name(name: &str) -> Option<&PooledCardData> {
        INSTANCE.by_name.get(name)
    }
    pub fn from_disk() -> io::Result<CardPool> {
        let file = File::open("./output/cards_out.json")?;
        let by_name: HashMap<String, PooledCardData> = serde_json::from_reader(file)?;
        Ok(CardPool {
            by_name,
        })
    }
    pub fn write_to_disk(&self) -> io::Result<()> {
        match fs::create_dir("./output/") {
            Ok(()) => info!("Created 'output' directory."),
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    info!("The 'output' directory already exists.");
                    return Ok(());
                } else {
                    warn!("Could not create 'output' directory. Error: {}", e)
                }
            }
        }
        let writer = File::create("./output/cards_out.json")?;
        serde_json::to_writer_pretty(writer, &self.by_name)?;
        Ok(())
    }
}
