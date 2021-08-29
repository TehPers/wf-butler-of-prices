#[macro_export]
macro_rules! routes {
    (@req_body $builder:expr, [$body_type:ident] $body:expr) => {
        $builder.$body_type($body)
    };
    (@req_body $builder:expr, $body:expr) => {
        routes!(@req_body $builder, [json] $body)
    };
    (@req_body $builder:expr,) => {
        $builder
    };
    (@major_params $params:expr) => {
        $params
    };
    (@major_params) => {
        Default::default()
    };
    (@query $builder:expr, $query:expr) => {
        $builder.query($query)
    };
    (@query $builder:expr,) => {
        $builder
    };
    (@needs_auth $needs_auth:expr) => {
        $needs_auth
    };
    (@needs_auth) => {
        true
    };
    ($(
        (
            $variant:ident {
                $($url_param:ident : $url_param_type:ty),*
                $(,)?
            },
            $(body = $([$body_type:ident])? $body_param:ident : $body_param_type:ty,)?
            $(extra = {
                $($extra_field:ident : $extra_field_type:ty),*
                $(,)?
            },)?
            method = $method:ident $route:expr,
            $(query = $query:expr,)?
            $(major_params = $major_params:expr,)?
            $(processor = |$req:ident| $processor:expr,)?
            $(needs_auth = $needs_auth:expr,)?
            helper = $helper:ident -> $response:ty
            $(,)?
        )
    ),* $(,)?) => {
        $(
            #[derive(Clone, Debug)]
            pub struct $variant {
                $(pub $url_param: $url_param_type,)*
                $(pub $body_param: $body_param_type,)?
                $($(pub $extra_field: $extra_field_type,)*)?
            }

            impl ::std::fmt::Display for $variant {
                fn fmt(
                    &self,
                    f: &mut ::std::fmt::Formatter<'_>
                ) -> ::std::fmt::Result {
                    let Self { $(ref $url_param,)* .. } = self;

                    write!(f, $route, $($url_param = $url_param),*)
                }
            }

            impl $crate::http::Route for $variant {
                #[inline]
                fn method(&self) -> $crate::reqwest::Method {
                    $crate::reqwest::Method::$method
                }

                #[inline]
                fn bucket(&self) -> $crate::request::RateLimitBucket {
                    #[allow(unused_variables)]
                    let Self { $($url_param,)* .. } = self;

                    $crate::request::RateLimitBucket::new(
                        $crate::reqwest::Method::$method,
                        $route,
                        $crate::routes!(@major_params $($major_params)?),
                    )
                }

                #[inline]
                fn needs_auth(&self) -> bool {
                    !$crate::routes!(@needs_auth $($needs_auth)?)
                }

                #[inline]
                fn make_request(
                    &self,
                    client: &$crate::reqwest::Client,
                    base_url: &str
                ) -> $crate::reqwest::RequestBuilder {
                    let Self {
                        $(ref $url_param,)*
                        $(ref $body_param,)?
                        $($(ref $extra_field,)*)?
                    } = self;

                    // Build request URL
                    let url = ::std::format!(
                        ::std::concat!("{}", $route),
                        base_url,
                        $($url_param = $url_param),*
                    );

                    // Build request
                    let request = client.request($crate::http::Route::method(self), url);

                    // Body
                    let request = $crate::routes!(@req_body request, $($([$body_type])? &$body_param)?);

                    // Query string
                    let request = $crate::routes!(@query request, $($query)?);

                    // Processor
                    $(
                        let $req = request;
                        let request = $processor;
                    )?

                    request
                }
            }


            #[tracing::instrument(
                skip(
                    client
                    $(, $url_param)*
                    $(, $body_param)?
                    $($(, $extra_field)*)?
                )
            )]
            pub async fn $helper(
                client: &$crate::client::DiscordRestClient
                $(, $url_param: $url_param_type)*
                $(, $body_param: $body_param_type)?
                $($(, $extra_field: $extra_field_type)*)?
            ) -> ::std::result::Result<
                $response,
                $crate::client::RequestError
            > {
                // Build request
                let route = $variant {
                    $($url_param,)*
                    $($body_param,)?
                    $($($extra_field,)*)?
                };

                // Execute request
                let response = client.request(route).await?;

                // Parse response
                let response = response.json()
                    .await
                    .map_err(
                        $crate::client::RequestError::ResponseParseError
                    )?;

                ::std::result::Result::Ok(response)
            }
        )*
    };
}
