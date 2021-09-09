use crate::models::{
    ItemOrdersPayload, ItemPayload, ItemShort, ItemsPayload, PayloadResponse,
    Platform,
};
use chrono::Duration;
use http::HeaderValue;
use wfinfo_lib::{reqwest::Method, routes};

#[derive(Clone, Debug, Hash)]
pub struct RouteInfo {
    pub bucket: Option<CacheBucket>,
    pub cache_time: Option<Duration>,
}

impl RouteInfo {
    pub fn new_uncached() -> Self {
        RouteInfo {
            bucket: None,
            cache_time: None,
        }
    }

    pub fn new_cached(
        bucket: CacheBucket,
        cache_time: Option<Duration>,
    ) -> Self {
        RouteInfo {
            bucket: Some(bucket),
            cache_time,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct CacheBucket {
    pub method: Method,
    pub route: &'static str,
    pub values: Vec<String>,
}

const MINUTE: i64 = 60;
const HOUR: i64 = MINUTE * 60;
const DAY: i64 = HOUR * 24;

pub const PLATFORM_HEADER: &'static str = "platform";

routes! {
    (
        GetItems {},
        method = GET "/items",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        helper = PayloadResponse<ItemsPayload<ItemShort>>,
    ),
    (
        GetItem { url_name: &'s str },
        generics = ['s],
        extra = { platform: Platform },
        method = GET "/items/{url_name}",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![url_name.to_string()],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        processor = |req| {
            req.header(
                PLATFORM_HEADER,
                HeaderValue::from_static(platform.name()),
            )
        },
        helper = PayloadResponse<ItemPayload>,
    ),
    (
        GetItemOrders { url_name: &'s str },
        generics = ['s],
        extra = { platform: Platform },
        method = GET "/items/{url_name}/orders",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![url_name.to_string()],
                },
                Some(Duration::seconds(HOUR)),
            )
        },
        processor = |req| {
            req
                .query(&[("include", "item")])
                .header("platform", HeaderValue::from_static(match platform {
                    Platform::XBox => "xbox",
                    Platform::PC => "pc",
                    Platform::PS4 => "ps4",
                    Platform::Switch => "switch",
                }))
        },
        helper = PayloadResponse<ItemOrdersPayload, ItemPayload>,
    ),
    // Liches
    (
        GetLichWeapons {},
        method = GET "/lich/weapons",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        // TODO
        helper = PayloadResponse<()>,
    ),
    (
        GetLichEphemeras {},
        method = GET "/lich/ephemeras",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        // TODO
        helper = PayloadResponse<()>,
    ),
    (
        GetLichQuirks {},
        method = GET "/lich/quirks",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        // TODO
        helper = PayloadResponse<()>,
    ),
    // Rivens
    (
        GetRivenItems {},
        method = GET "/rivens/items",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        // TODO
        helper = PayloadResponse<()>,
    ),
    (
        GetRivenAttributes {},
        method = GET "/rivens/attributes",
        info = |method, route| -> RouteInfo {
            RouteInfo::new_cached(
                CacheBucket {
                    method,
                    route,
                    values: vec![],
                },
                Some(Duration::seconds(DAY)),
            )
        },
        // TODO
        helper = PayloadResponse<()>,
    ),
}
