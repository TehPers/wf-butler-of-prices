use crate::services::WarframeItemService;
use anyhow::Context;
use std::{fmt::Write, sync::Arc};
use tracing::debug;
use wfinfo_commands::{
    create_callback, CommandBuilder, CommandOptionRegistry,
    HandleInteractionError, InteractionData, SlashCommand,
};
use wfinfo_discord::{
    models::{
        AllowedMentions, CreateWebhookMessage, Embed, EmbedField,
        EmbedThumbnail, MessageFlags, Snowflake,
    },
    routes::CreateFollowupMessage,
    DiscordRestClient,
};
use wfinfo_wm::{
    models::{
        ItemFull, ItemOrdersPayload, ItemPayload, OrderType, PayloadResponse,
        Platform, UserStatus,
    },
    routes::GetItemOrders,
    WarframeMarketRestClient,
};

const WM_ASSETS_ROOT: &'static str = "http://warframe.market/static/assets/";
const PLAT: &'static str = "<:WFPlatinum:380292389798936579>";

pub fn pc_command(
    discord_client: DiscordRestClient,
    wm_client: WarframeMarketRestClient,
    item_service: WarframeItemService,
    app_id: Snowflake,
) -> SlashCommand {
    let query_item_cb = create_callback! {
        capture: {
            discord_client: DiscordRestClient = discord_client,
            wm_client: WarframeMarketRestClient = wm_client,
            item_service: WarframeItemService = item_service,
            app_id: Snowflake = app_id,
        },
        handler: async |interaction_data, _, options| {
            pc(interaction_data, options, discord_client, wm_client, item_service, app_id).await
        },
    };

    CommandBuilder::new()
        .name("pc")
        .description("Checks warframe.market for the price of an item")
        .default_permission(true)
        .subcommand_option(|builder| {
            builder
                .name("item")
                .description("Searches for the price of a normal item (blueprint, part, etc.)")
                .string_option(|builder| {
                    builder
                        .name("name")
                        .description("The name of the item to get the price of")
                        .required(true)
                        .build()
                })
                .build()
        })
        .subcommand_option(|builder| {
            builder.name("mod")
                .description("Searches for the price of a mod")
                .string_option(|builder| {
                    builder.name("name")
                        .description("The name of the mod to get the price of")
                        .required(true)
                        .build()
                }).integer_option(|builder| {
                    builder.name("rank")
                        .description("The rank of the mod")
                        .choices((0..=10).collect::<Vec<_>>())
                        .required(false)
                        .build()
                }).build()
        })
        .string_option(|builder| {
            builder
                .name("item")
                .description("The item to get the price of")
                .required(true)
                .build()
        })
        .build()
}

async fn pc<'opts>(
    interaction_data: Arc<InteractionData>,
    options: CommandOptionRegistry<'opts>,
    client: &DiscordRestClient,
    wm_client: &WarframeMarketRestClient,
    item_service: &WarframeItemService,
    app_id: &Snowflake,
) -> Result<(), HandleInteractionError> {
    debug!("handling pc command");

    let item_name: &str = options.get_option("item")?;
    let item_name = item_name.to_lowercase();
    let url_name = item_service.get_url_name(&item_name);
    let url_name = match url_name {
        Some(url_name) => url_name,
        None => {
            CreateFollowupMessage::execute(
                client,
                *app_id,
                interaction_data.token.clone(),
                error_response(format!(
                    "No item with the name '{}' found",
                    item_name
                )),
            )
            .await
            .context("error creating response")?;

            return Ok(());
        }
    };

    let response =
        GetItemOrders::execute(wm_client, url_name.as_str(), Platform::PC)
            .await
            .context("error getting item orders")?;

    let message = create_response(response);
    CreateFollowupMessage::execute(
        client,
        *app_id,
        interaction_data.token.clone(),
        message,
    )
    .await
    .context("error creating response")?;

    Ok(())
}

fn error_response(content: impl Into<String>) -> CreateWebhookMessage {
    CreateWebhookMessage {
        embeds: Some(vec![Embed {
            title: Some("Error".into()),
            description: Some(content.into()),
            ..Default::default()
        }]),
        allowed_mentions: Some(AllowedMentions {
            parse: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn partial_error_response(
    content: impl Into<String>,
    item_details: &ItemFull,
) -> CreateWebhookMessage {
    CreateWebhookMessage {
        embeds: Some(vec![Embed {
            title: Some(format!("Error ({})", item_details.en.item_name)),
            description: Some(content.into()),
            thumbnail: Some(EmbedThumbnail {
                url: Some(format!(
                    "{}{}",
                    WM_ASSETS_ROOT,
                    item_details
                        .sub_icon
                        .as_ref()
                        .unwrap_or(&item_details.icon)
                )),
                ..Default::default()
            }),
            ..Default::default()
        }]),
        allowed_mentions: Some(AllowedMentions {
            parse: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn create_response(
    wm_res: PayloadResponse<ItemOrdersPayload, ItemPayload>,
) -> CreateWebhookMessage {
    // Get item details
    let item_details = match wm_res.include.as_ref() {
        None => return error_response("Missing item details"),
        Some(item_payload) => {
            let item = item_payload
                .item
                .items_in_set
                .iter()
                .find(|item| &item.id == &item_payload.item.id);

            match item {
                None => return error_response("Missing correct item details"),
                Some(item) => item,
            }
        }
    };

    // Get orders
    let mut orders: Vec<_> = wm_res
        .payload
        .orders
        .iter()
        .filter(|order| {
            order.order_type == OrderType::Sell
                && order.user.status == UserStatus::InGame
        })
        .collect();

    // Check if no orders
    if orders.is_empty() {
        return partial_error_response("No orders found", item_details);
    }

    orders.sort_unstable_by_key(|order| order.platinum);
    let count = orders.len();
    let sum: u32 = orders.iter().map(|order| order.platinum).sum();
    let mean = sum as f64 / count as f64;
    let variance = orders
        .iter()
        .map(|order| (order.platinum as f64 - mean).powi(2))
        .sum::<f64>()
        / (count - 1) as f64;
    let deviation = variance.sqrt();
    let range =
        orders.first().unwrap().platinum..=orders.last().unwrap().platinum;
    let median = if count % 2 == 0 {
        orders[count / 2 - 1].platinum as f64 / 2.0
            + orders[count / 2].platinum as f64 / 2.0
    } else {
        orders[count / 2].platinum as f64
    };

    let main_embed = Embed {
        title: Some(item_details.en.item_name.clone()),
        description: Some(item_details.en.description.clone()),
        thumbnail: Some(EmbedThumbnail {
            url: Some(format!(
                "{}{}",
                WM_ASSETS_ROOT,
                item_details.sub_icon.as_ref().unwrap_or(&item_details.icon)
            )),
            ..Default::default()
        }),
        fields: Some(vec![
            EmbedField {
                name: "Price range".to_string(),
                value: format!(
                    "{}{p} - {}{p}",
                    range.start(),
                    range.end(),
                    p = PLAT
                ),
                inline: Some(true),
            },
            EmbedField {
                name: "Mean (x̄)".to_string(),
                value: format!("{:0.2}{p}", mean, p = PLAT),
                inline: Some(true),
            },
            EmbedField {
                name: "Median".to_string(),
                value: format!("{:.1}{p}", median, p = PLAT),
                inline: Some(true),
            },
            EmbedField {
                name: "Standard deviation (s)".to_string(),
                value: format!("{:.2}", deviation),
                inline: Some(true),
            },
        ]),
        ..Default::default()
    };
    let offers =
        orders
            .iter()
            .take(3)
            .fold(String::new(), |mut offers, order| {
                // TODO: write to offers and put into an embed
                writeln!(
                    offers,
                    "**{}** ({:+}): {}{p}, {} remaining ```",
                    order.user.ingame_name,
                    order.user.reputation,
                    order.platinum,
                    order.quantity,
                    p = PLAT,
                ).unwrap();
                writeln!(
                    offers,
                    "/w {} Hi! I want to buy: {} for {} platinum. (warframe.market)",
                    order.user.ingame_name,
                    item_details.en.item_name,
                    order.platinum,
                )
                .unwrap();
                writeln!(offers, "```").unwrap();

                offers
            });

    let offers_embed = Embed {
        title: Some("Best Offers".to_string()),
        description: Some(offers),
        ..Default::default()
    };

    CreateWebhookMessage {
        embeds: Some(vec![main_embed, offers_embed]),
        allowed_mentions: Some(AllowedMentions {
            parse: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    }
}
