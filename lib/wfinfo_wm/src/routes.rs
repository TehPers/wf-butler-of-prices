use crate::{
    middleware::{AsCacheInfo, CacheInfo},
    models::{
        ItemOrdersPayload, ItemPayload, ItemShort, ItemsPayload,
        PayloadResponse, Platform,
    },
};
use http::HeaderValue;
use reqwest::Method;
use std::time::Duration;
use wfinfo_http::routes;

#[derive(Clone, Debug, Hash)]
pub struct WmRouteInfo {
    pub bucket: Option<CacheBucket>,
    pub cache_time: Option<Duration>,
}

impl WmRouteInfo {
    pub fn new_uncached() -> Self {
        WmRouteInfo {
            bucket: None,
            cache_time: None,
        }
    }

    pub fn new_cached(
        bucket: CacheBucket,
        cache_time: Option<Duration>,
    ) -> Self {
        WmRouteInfo {
            bucket: Some(bucket),
            cache_time,
        }
    }
}

impl AsCacheInfo for WmRouteInfo {
    fn cache_info(&self) -> Option<CacheInfo<'_>> {
        self.bucket
            .as_ref()
            .zip(self.cache_time)
            .map(|(bucket, cache_time)| {
                let values: String = bucket.values.join(":");
                let bucket = format!(
                    "wf_butler:cached:wm:{route}:{method}:{values}",
                    route = bucket.route,
                    method = bucket.method
                );
                CacheInfo {
                    bucket: bucket.into(),
                    expiry_secs: cache_time.as_secs(),
                }
            })
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct CacheBucket {
    pub method: Method,
    pub route: &'static str,
    pub values: Vec<String>,
}

const MINUTE: u64 = 60;
const HOUR: u64 = MINUTE * 60;
const DAY: u64 = HOUR * 24;

pub const PLATFORM_HEADER: &'static str = "platform";

routes! {
    (
        GetItems {},
        method = GET "/items",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        response = [json] PayloadResponse<ItemsPayload<ItemShort>>,
    ),
    (
        GetItem {
            url_name: String,
            platform: Platform,
        },
        method = GET "/items/{url_name}",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![url_name.clone()],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        processor = |req| {
            req.header(
                PLATFORM_HEADER,
                HeaderValue::from_static(platform.name()),
            )
        },
        response = [json] PayloadResponse<ItemPayload>,
    ),
    (
        GetItemOrders {
            url_name: String,
            platform: Option<Platform>,
        },
        method = GET "/items/{url_name}/orders",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![url_name.clone(), format!("{:?}", platform)],
                },
                Some(Duration::from_secs(HOUR)),
            )
        },
        processor = |req| {
            let req = req.query(&[("include", "item")]);
            let req = match platform {
                Some(platform) => req.header("platform", HeaderValue::from_static(match platform {
                    Platform::XBox => "xbox",
                    Platform::PC => "pc",
                    Platform::PS4 => "ps4",
                    Platform::Switch => "switch",
                })),
                None => req,
            };

            req
        },
        response = [json] PayloadResponse<ItemOrdersPayload, ItemPayload>,
    ),
    // Liches
    (
        GetLichWeapons {},
        method = GET "/lich/weapons",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        // TODO
        response = [json] PayloadResponse<()>,
    ),
    (
        GetLichEphemeras {},
        method = GET "/lich/ephemeras",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        // TODO
        response = [json] PayloadResponse<()>,
    ),
    (
        GetLichQuirks {},
        method = GET "/lich/quirks",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        // TODO
        response = [json] PayloadResponse<()>,
    ),
    // Rivens
    (
        GetRivenItems {},
        method = GET "/rivens/items",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        // TODO
        response = [json] PayloadResponse<()>,
    ),
    (
        GetRivenAttributes {},
        method = GET "/rivens/attributes",
        info = |method, route| -> WmRouteInfo {
            WmRouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::from_secs(DAY)),
            )
        },
        // TODO
        response = [json] PayloadResponse<()>,
    ),
}
