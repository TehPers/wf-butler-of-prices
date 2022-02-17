use crate::services::WarframeItemService;
use anyhow::{bail, Context};
use std::{borrow::Cow, fmt::Write, str::FromStr, sync::Arc};
use wfbp_commands::{
    create_callback, Choice, CommandBuilder, CommandOptionRegistry,
    InteractionData, SlashCommand,
};
use wfbp_discord::{
    models::{
        AllowedMentions, CreateWebhookMessage, Embed, EmbedField,
        EmbedThumbnail, MessageFlags, Snowflake,
    },
    routes::CreateFollowupMessage,
    DiscordRestClient,
};
use wfbp_wm::{
    models::{
        ItemFull, ItemOrder, ItemOrdersPayload, ItemPayload, ItemRank,
        OrderType, PayloadResponse, Platform, RelicRefinement, UserStatus,
    },
    routes::GetItemOrders,
    WmRestClient,
};

const WM_ASSETS_ROOT: &'static str = "http://warframe.market/static/assets/";
const PLAT: &'static str = "<:WFPlatinum:380292389798936579>";

pub fn pc_command(
    discord_client: DiscordRestClient,
    wm_client: WmRestClient,
    item_service: WarframeItemService,
    app_id: Snowflake,
) -> SlashCommand {
    let pc_items_callback = create_callback! {
        capture: {
            discord_client: DiscordRestClient = discord_client.clone(),
            wm_client: WmRestClient = wm_client.clone(),
            item_service: WarframeItemService = item_service.clone(),
            app_id: Snowflake = app_id,
        },
        handler: async |interaction_data, _, options| {
            pc_items(interaction_data, options, discord_client, wm_client, item_service, app_id).await
        },
    };
    let pc_mod_callback = create_callback! {
        capture: {
            discord_client: DiscordRestClient = discord_client.clone(),
            wm_client: WmRestClient = wm_client.clone(),
            item_service: WarframeItemService = item_service.clone(),
            app_id: Snowflake = app_id,
        },
        handler: async |interaction_data, _, options| {
            pc_mod_or_arcane(interaction_data, options, discord_client, wm_client, item_service, app_id).await
        },
    };
    let pc_arcane_callback = create_callback! {
        capture: {
            discord_client: DiscordRestClient = discord_client.clone(),
            wm_client: WmRestClient = wm_client.clone(),
            item_service: WarframeItemService = item_service.clone(),
            app_id: Snowflake = app_id,
        },
        handler: async |interaction_data, _, options| {
            pc_mod_or_arcane(interaction_data, options, discord_client, wm_client, item_service, app_id).await
        },
    };
    let pc_relic_callback = create_callback! {
        capture: {
            discord_client: DiscordRestClient = discord_client.clone(),
            wm_client: WmRestClient = wm_client.clone(),
            item_service: WarframeItemService = item_service.clone(),
            app_id: Snowflake = app_id,
        },
        handler: async |interaction_data, _, options| {
            pc_relic(interaction_data, options, discord_client, wm_client, item_service, app_id).await
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
                })
                .string_option(|builder| {
                    builder.name("platform")
                        .description("The platform")
                        .choices(PlatformChoice::choices().into_iter().collect())
                        .required(false)
                })
                .callback(pc_items_callback)
        })
        .subcommand_option(|builder| {
            builder.name("mod")
                .description("Searches for the price of a mod")
                .string_option(|builder| {
                    builder.name("name")
                        .description("The name of the mod to get the price of")
                        .required(true)
                })
                .string_option(|builder| {
                    builder.name("platform")
                        .description("The platform")
                        .choices(PlatformChoice::choices().into_iter().collect())
                        .required(false)
                })
                .integer_option(|builder| {
                    builder.name("rank")
                        .description("The rank of the mod")
                        .required(false)
                })
                .callback(pc_mod_callback)
        })
        .subcommand_option(|builder| {
            builder.name("arcane")
                .description("Searches for the price of an arcane")
                .string_option(|builder| {
                    builder.name("name")
                        .description("The name of the arcane to get the price of")
                        .required(true)
                })
                .string_option(|builder| {
                    builder.name("platform")
                        .description("The platform")
                        .choices(PlatformChoice::choices().into_iter().collect())
                        .required(false)
                })
                .integer_option(|builder| {
                    builder.name("rank")
                        .description("The rank of the arcane")
                        .required(false)
                })
                .callback(pc_arcane_callback)
        })
        .subcommand_option(|builder| {
            builder.name("relic")
                .description("Searches for the price of a relic")
                .string_option(|builder| {
                    builder.name("name")
                        .description("The name of the relic to get the price of")
                        .required(true)
                })
                .string_option(|builder| {
                    builder.name("platform")
                        .description("The platform")
                        .choices(PlatformChoice::choices().into_iter().collect())
                        .required(false)
                })
                .string_option(|builder| {
                    builder.name("refinement")
                        .description("The refinement level of the relic")
                        .choices(RelicRefinementChoice::choices().into_iter().collect())
                        .required(false)
                })
                .callback(pc_relic_callback)
        })
        .build()
}

macro_rules! enum_choice {
    {
        $(#[$($attr:meta),* $(,)?])*
        enum $name:ident {
            $($variant:ident = $choice_val:literal),*
            $(,)?
        }
    } => {
        $(#[$($attr),*])*
        enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn variants() -> impl IntoIterator<Item = Self> {
                [$(Self::$variant),*]
            }

            pub fn choices() -> impl IntoIterator<Item = Choice<Cow<'static, str>>> {
                Self::variants()
                    .into_iter()
                    .map(Self::to_choice)
            }

            pub fn to_choice(self) -> Choice<Cow<'static, str>> {
                match self {
                    $(
                        Self::$variant => Choice {
                            name: stringify!($variant).into(),
                            value: $choice_val.into()
                        },
                    )*
                }
            }
        }

        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($choice_val => Ok(Self::$variant),)*
                    _ => bail!("unknown variant: '{}'", s),
                }
            }
        }
    };
}

enum_choice! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    enum RelicRefinementChoice {
        Intact = "intact",
        Exceptional = "exceptional",
        Flawless = "flawless",
        Radiant = "radiant",
    }
}

impl From<RelicRefinementChoice> for RelicRefinement {
    fn from(refinement: RelicRefinementChoice) -> Self {
        match refinement {
            RelicRefinementChoice::Intact => RelicRefinement::Intact,
            RelicRefinementChoice::Exceptional => RelicRefinement::Exceptional,
            RelicRefinementChoice::Flawless => RelicRefinement::Flawless,
            RelicRefinementChoice::Radiant => RelicRefinement::Radiant,
        }
    }
}

enum_choice! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
    enum PlatformChoice {
        PC = "pc",
        XBox = "xbox",
        PS4 = "ps4",
        Switch = "switch",
    }
}

impl From<PlatformChoice> for Platform {
    fn from(platform: PlatformChoice) -> Self {
        match platform {
            PlatformChoice::PC => Platform::PC,
            PlatformChoice::XBox => Platform::XBox,
            PlatformChoice::PS4 => Platform::PS4,
            PlatformChoice::Switch => Platform::Switch,
        }
    }
}

async fn pc_items<'opts>(
    interaction_data: Arc<InteractionData>,
    options: CommandOptionRegistry<'opts>,
    discord_client: &DiscordRestClient,
    wm_client: &WmRestClient,
    item_service: &WarframeItemService,
    app_id: &Snowflake,
) -> anyhow::Result<()> {
    // Get options
    let item_name: &str = options.get_option("name")?;
    let item_name = item_name.to_lowercase();
    let platform = options
        .get_optional_option("platform")
        .context("error getting option")?
        .map(|platform: &str| platform.parse())
        .transpose()
        .context("error parsing option")?
        .map(PlatformChoice::into);

    pc_filtered(
        interaction_data,
        discord_client,
        wm_client,
        item_service,
        app_id,
        &item_name,
        OrderFilters {
            platform,
            rank: RankFilter::Item,
        },
    )
    .await
}

async fn pc_mod_or_arcane<'opts>(
    interaction_data: Arc<InteractionData>,
    options: CommandOptionRegistry<'opts>,
    discord_client: &DiscordRestClient,
    wm_client: &WmRestClient,
    item_service: &WarframeItemService,
    app_id: &Snowflake,
) -> anyhow::Result<()> {
    // Get options
    let item_name: &str = options.get_option("name")?;
    let item_name = item_name.to_lowercase();
    let rank = options.get_optional_option("rank")?;
    let platform = options
        .get_optional_option("platform")
        .context("error getting platform")?
        .map(|platform: &str| platform.parse())
        .transpose()
        .context("error parsing platform")?
        .map(PlatformChoice::into);

    pc_filtered(
        interaction_data,
        discord_client,
        wm_client,
        item_service,
        app_id,
        &item_name,
        OrderFilters {
            platform,
            rank: RankFilter::ModOrArcane { rank },
        },
    )
    .await
}

async fn pc_relic<'opts>(
    interaction_data: Arc<InteractionData>,
    options: CommandOptionRegistry<'opts>,
    discord_client: &DiscordRestClient,
    wm_client: &WmRestClient,
    item_service: &WarframeItemService,
    app_id: &Snowflake,
) -> anyhow::Result<()> {
    // Get options
    let item_name: &str = options.get_option("name")?;
    let item_name = item_name.to_lowercase();
    let refinement = options
        .get_optional_option("refinement")
        .context("error getting refinement")?
        .map(|refinement: &str| refinement.parse())
        .transpose()
        .context("error parsing refinement")?
        .map(RelicRefinementChoice::into);
    let platform = options
        .get_optional_option("platform")
        .context("error getting platform")?
        .map(|platform: &str| platform.parse())
        .transpose()
        .context("error parsing platform")?
        .map(PlatformChoice::into);

    pc_filtered(
        interaction_data,
        discord_client,
        wm_client,
        item_service,
        app_id,
        &item_name,
        OrderFilters {
            platform,
            rank: RankFilter::Relic { refinement },
        },
    )
    .await
}

async fn pc_filtered<'opts>(
    interaction_data: Arc<InteractionData>,
    discord_client: &DiscordRestClient,
    wm_client: &WmRestClient,
    item_service: &WarframeItemService,
    app_id: &Snowflake,
    item_name: &str,
    order_filters: OrderFilters,
) -> anyhow::Result<()> {
    // Get message
    let message = process(wm_client, item_service, &item_name, order_filters)
        .await
        .unwrap_or_else(|error| {
            error_response(format!("```\n{:#?}\n```", error))
        });

    // Send response
    CreateFollowupMessage::execute(
        discord_client,
        *app_id,
        interaction_data.token.clone(),
        message,
    )
    .await
    .context("error creating response")?;

    Ok(())
}

#[derive(Clone, Debug)]
struct OrderFilters {
    pub platform: Option<Platform>,
    pub rank: RankFilter,
}

impl OrderFilters {
    pub fn matches(&self, order: &ItemOrder) -> bool {
        // Platform
        if let Some(platform) = self.platform {
            if platform != order.platform {
                return false;
            }
        }

        // Item rank/refinement
        match self.rank {
            RankFilter::Item => matches!(order.rank, ItemRank::Item {}),
            RankFilter::ModOrArcane { rank: rank_filter } => match order.rank {
                ItemRank::ModOrArcane { mod_rank } => {
                    rank_filter.map_or(true, |filter| mod_rank == filter)
                }
                _ => false,
            },
            RankFilter::Relic {
                refinement: refinement_filter,
            } => match order.rank {
                ItemRank::Relic { refinement } => refinement_filter
                    .map_or(true, |filter| refinement == filter),
                _ => false,
            },
        }
    }
}

#[derive(Clone, Debug)]
enum RankFilter {
    ModOrArcane { rank: Option<u8> },
    Relic { refinement: Option<RelicRefinement> },
    Item,
}

async fn process(
    wm_client: &WmRestClient,
    item_service: &WarframeItemService,
    item_name: &str,
    order_filters: OrderFilters,
) -> anyhow::Result<CreateWebhookMessage> {
    // Look up item name
    let url_name = item_service.get_url_name(&item_name);
    let url_name = match url_name {
        Some(url_name) => url_name,
        None => bail!("No item with the name '{item_name}' found"),
    };

    // Get orders
    let response = GetItemOrders::execute(
        wm_client,
        url_name.as_ref().to_owned(),
        order_filters.platform,
    )
    .await
    .context("error getting item orders")?;

    // Build response
    let message = create_response(response, order_filters);
    Ok(message)
}

fn create_response(
    wm_res: PayloadResponse<ItemOrdersPayload, ItemPayload>,
    order_filters: OrderFilters,
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
            // Only show sell orders by people current ingame
            order.order_type == OrderType::Sell
                && order.user.status == UserStatus::InGame
        })
        .filter(|order| order_filters.matches(order))
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
                    "{start}{PLAT} - {end}{PLAT}",
                    start = range.start(),
                    end = range.end(),
                ),
                inline: Some(true),
            },
            EmbedField {
                name: "Mean (x̄)".to_string(),
                value: format!("{mean:0.2}{PLAT}"),
                inline: Some(true),
            },
            EmbedField {
                name: "Median".to_string(),
                value: format!("{median:.1}{PLAT}"),
                inline: Some(true),
            },
            EmbedField {
                name: "Standard deviation (s)".to_string(),
                value: format!("{deviation:.2}"),
                inline: Some(true),
            },
        ]),
        ..Default::default()
    };
    let offers_description =
        orders
            .iter()
            .take(3)
            .fold(String::new(), |mut offers, order| {
                writeln!(
                    offers,
                    "**{seller}** ({rep:+}): {cost}{PLAT}, {quantity} remaining ```",
                    seller = order.user.ingame_name,
                    rep = order.user.reputation,
                    cost = order.platinum,
                    quantity = order.quantity,
                ).unwrap();
                match order.rank {
                    ItemRank::ModOrArcane { mod_rank: rank, .. } => writeln!(
                        offers,
                        "/w {seller} Hi! I want to buy: {item} (rank {rank}) for {cost} platinum. (warframe.market)",
                        seller = order.user.ingame_name,
                        item = item_details.en.item_name,
                        cost = order.platinum,
                    )
                    .unwrap(),
                    ItemRank::Relic { refinement, .. } => writeln!(
                        offers,
                        "/w {seller} Hi! I want to buy: {item} ({refinement}) for {cost} platinum. (warframe.market)",
                        seller = order.user.ingame_name,
                        item = item_details.en.item_name,
                        refinement = match refinement {
                            RelicRefinement::Intact => "intact",
                            RelicRefinement::Exceptional => "exceptional",
                            RelicRefinement::Flawless => "flawless",
                            RelicRefinement::Radiant => "radiant",
                        },
                        cost = order.platinum,
                    )
                    .unwrap(),
                    ItemRank::Item { .. } => writeln!(
                        offers,
                        "/w {seller} Hi! I want to buy: {item} for {cost} platinum. (warframe.market)",
                        seller = order.user.ingame_name,
                        item = item_details.en.item_name,
                        cost = order.platinum,
                    )
                    .unwrap(),
                }
                writeln!(offers, "```").unwrap();

                offers
            });

    let offers_embed = Embed {
        title: Some(format!("Best Offers ({} sellers)", orders.len())),
        description: Some(offers_description),
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
        flags: Some(MessageFlags::EPHEMERAL),
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
