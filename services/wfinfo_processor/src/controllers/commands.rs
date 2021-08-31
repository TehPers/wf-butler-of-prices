use std::{collections::HashMap, sync::Arc};

use crate::models::{CommandError, Config};
use actix_web::{
    dev::HttpServiceFactory,
    post,
    web::{scope, Data, Json},
};
use qp_trie::{wrapper::BString, Trie};
use tracing::{debug, instrument};
use wfinfo_azure::functions::FunctionsOutput;
use wfinfo_discord::{
    models::CreateApplicationCommand, routes::CreateGlobalApplicationCommand,
    DiscordRestClient,
};
use wfinfo_wm::{
    models::ItemShort, routes::GetItems, WarframeMarketRestClient,
};

pub fn commands_service() -> impl HttpServiceFactory {
    scope("/commands").service(handle_command)
}

#[post("")]
#[instrument(skip(config, wm_client, client))]
async fn handle_command(
    config: Data<Config>,
    wm_client: Data<WarframeMarketRestClient>,
    client: Data<DiscordRestClient>,
) -> Result<Json<FunctionsOutput<()>>, CommandError> {
    // let command = CreateGlobalApplicationCommand::execute(
    //     client.as_ref(),
    //     config.app_id,
    //     CreateApplicationCommand {
    //         name: "test".into(),
    //         description: "Test command".into(),
    //         options: None,
    //         default_permission: None,
    //     },
    // )
    // .await
    // .map_err(CommandError::RequestError)?;
    // debug!(?command, "created slash command");

    debug!("building lookup trie");
    let lookup = build_lookup(wm_client.as_ref()).await?;
    // debug!(?lookup, "built trie");

    debug!(entries=?lookup.count(), "querying trie");
    let query = "mirage p sys";
    let url_name = lookup.get_str(query);
    debug!(?url_name, "done with lookup");

    Ok(Json(FunctionsOutput {
        outputs: (),
        logs: vec![],
        return_value: None,
    }))
}

async fn build_lookup(
    wm_client: &WarframeMarketRestClient,
) -> Result<Trie<BString, Arc<String>>, CommandError> {
    // Get searchable items from w.m
    let items = GetItems::execute(wm_client)
        .await
        .map_err(CommandError::RequestError)?;
    debug!(items = ?items.payload.items.len(), "got items");

    // Build abbreviation table
    let mut abbrvs = HashMap::new();
    abbrvs.insert("prime", vec!["p"]);
    abbrvs.insert("blueprint", vec!["bp"]);
    abbrvs.insert("neuroptics", vec!["neur", "helm", "helmet"]);
    abbrvs.insert("systems", vec!["sys"]);
    abbrvs.insert("chassis", vec!["chas", "chasses"]);
    abbrvs.insert("vauban", vec!["booben"]);
    abbrvs.insert("infested", vec!["inf"]);
    abbrvs.insert("corpus", vec!["corp"]);

    // Create trie
    let trie: Trie<BString, Arc<String>> = items
        .payload
        .items
        .into_iter()
        .flat_map(|item| {
            let ItemShort {
                url_name,
                item_name,
                ..
            } = item;

            let url_name = Arc::new(url_name);
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
        .collect();
    debug!(entries = ?trie.count(), "created trie");

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
                if !phrase.is_empty() {
                    phrase.push(' ');
                }
                phrase.push_str(&word.to_lowercase());
                dfs_build_phrases(phrases, phrase, remaining);
            }
        }
        None => {
            phrases.push(phrase);
        }
    }
}
