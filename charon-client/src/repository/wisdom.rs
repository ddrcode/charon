// SPDX-License-Identifier: GPL-3.0-or-later
use rand::prelude::IndexedRandom;
use serde::Deserialize;
use std::{collections::HashMap, fs};

type DB = HashMap<String, Vec<String>>;

#[derive(Debug, Deserialize)]
pub struct WisdomCategory {
    name: String,
    quotes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WisdomData {
    categories: Vec<WisdomCategory>,
}

pub struct WisdomDb {
    db: DB,
    default_wisdom: String,
}

impl WisdomDb {
    pub fn from_file(path: &str) -> std::io::Result<WisdomDb> {
        let contents = fs::read_to_string(path)?;
        let db: WisdomData = serde_json::from_str(&contents)?;
        let db: DB = db
            .categories
            .into_iter()
            .map(|cat| (cat.name, cat.quotes))
            .collect();
        Ok(WisdomDb {
            db,
            default_wisdom: String::from(
                "Charon is rowing...\n\nPress the <[magic key]> to take control",
            ),
        })
    }

    pub fn get_random_wisdom(&self, category: &str) -> &String {
        self.db
            .get(category)
            .and_then(|list| list.choose(&mut rand::rng()))
            .unwrap_or(&self.default_wisdom)
    }
}
