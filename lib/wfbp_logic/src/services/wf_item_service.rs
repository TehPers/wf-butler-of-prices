use anyhow::Context;
use qp_trie::{wrapper::BString, Trie};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, instrument, warn};
use wfbp_wm::{models::ItemShort, routes::GetItems, WmRestClient};

#[derive(Debug, Clone)]
pub struct WarframeItemService {
    lookup: Arc<Trie<BString, Arc<str>>>,
}

impl WarframeItemService {
    #[instrument(skip(wm_client))]
    pub async fn new(wm_client: WmRestClient) -> anyhow::Result<Self> {
        let lookup = build_lookup(&wm_client)
            .await
            .context("error building lookup table")?;
        debug!(entries=?lookup.count(), "created lookup trie for item queries");

        Ok(WarframeItemService {
            lookup: Arc::new(lookup),
        })
    }

    pub fn get_url_name(&self, query: &str) -> Option<Arc<str>> {
        self.lookup.get_str(query).cloned()
    }
}

#[instrument(skip(wm_client))]
async fn build_lookup(
    wm_client: &WmRestClient,
) -> anyhow::Result<Trie<BString, Arc<str>>> {
    // Get searchable items from w.m
    let items = GetItems::execute(wm_client)
        .await
        .context("error getting tradeable items from warframe.market")?;
    debug!(items = ?items.payload.items.len(), "got items");

    // Build abbreviation table
    let mut abbrvs = HashMap::new();
    abbrvs.insert("prime", vec!["p", ""]);
    abbrvs.insert("blueprint", vec!["bp"]);
    abbrvs.insert("neuroptics", vec!["neur", "helm", "helmet"]);
    abbrvs.insert("systems", vec!["sys"]);
    abbrvs.insert("chassis", vec!["chas", "chasses"]);
    abbrvs.insert("vauban", vec!["booben"]);
    abbrvs.insert("infested", vec!["inf"]);
    abbrvs.insert("corpus", vec!["corp"]);
    abbrvs.insert("set", vec![""]);
    abbrvs.insert("relic", vec![""]);

    // Create trie
    let trie = items
        .payload
        .items
        .into_iter()
        .flat_map(|item| {
            let ItemShort {
                url_name,
                item_name,
                ..
            } = item;

            let url_name: Arc<str> = url_name.into_boxed_str().into();
            let words = item_name.split_whitespace();
            let word_choices: Vec<Vec<_>> = words
                .into_iter()
                .map(|word| {
                    abbrvs
                        .get(word.to_lowercase().as_str())
                        .into_iter()
                        .flatten()
                        .copied()
                        .chain([word])
                        .collect()
                })
                .collect();

            let mut phrases = Vec::new();
            dfs_build_phrases(&mut phrases, String::new(), &word_choices);

            phrases
                .into_iter()
                .map(move |phrase| (BString::from(phrase), url_name.clone()))
        })
        .fold(Trie::new(), |mut trie, (key, value)| {
            if let Some(prev) = trie.insert(key.clone(), value) {
                warn!("duplicate key {key:?} for {prev:?} in trie");
            }
            trie
        });

    Ok(trie)
}

fn dfs_build_phrases(
    phrases: &mut Vec<String>,
    phrase: String,
    word_choices: &[Vec<&str>],
) {
    match word_choices.split_first() {
        Some((words, remaining)) => {
            for &word in words {
                let mut phrase = phrase.clone();
                if !word.is_empty() {
                    if !phrase.is_empty() {
                        phrase.push(' ');
                    }
                    phrase.push_str(&word.to_lowercase());
                }
                dfs_build_phrases(phrases, phrase, remaining);
            }
        }
        None => {
            phrases.push(phrase);
        }
    }
}
