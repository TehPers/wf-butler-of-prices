#[macro_export]
macro_rules! routes {
    (@req_body $builder:expr, $body:expr, [$body_type:ident]) => {
        $builder.$body_type($body)
    };
    (@req_body $builder:expr,) => {
        $builder
    };
    (@res_body $res:expr, [$body_type:ident]) => {
        {
            trait ResponseExt: ::std::marker::Sized {
                /// Gets an empty response without reading the response body.
                fn empty(self) -> ::std::future::Ready<$crate::reqwest::Result<()>> {
                    ::std::future::ready(::std::result::Result::Ok(()))
                }
            }
            impl ResponseExt for $crate::reqwest::Response {}

            $res.$body_type()
        }
    };
    (@query $builder:expr, $query:expr) => {
        $builder.query($query)
    };
    (@query $builder:expr,) => {
        $builder
    };
    (@info_type $info_type:ty, $($_:tt)*) => {
        $info_type
    };
    {
        $(
            (
                $(#[$route_attr:meta])*
                $route_ty:ident {
                    $(
                        $(#[$route_field_attr:meta])*
                        $route_field:ident : $route_field_type:ty
                    ),*
                    $(,)?
                }
                $(, generics = [$($generics:tt)*])?
                $(, body = [$body_type:ident] $body:expr)?
                , method = $method:ident $route:literal
                $(, info = |$info_method:pat_param, $info_route:pat_param| -> $info_type:ty $info:block)?
                $(, query = $query:expr)?
                $(, processor = |$req:pat_param| $processor:expr)?
                , response = [$res_body_type:ident] $response:ty
                $(,)?
            )
        ),*
        $(,)?
    } => {
        $(
            $(#[$route_attr])*
            #[derive(Clone, Debug)]
            pub struct $route_ty $(<$($generics)*>)? {
                $(
                    $(#[$route_field_attr])*
                    pub $route_field: $route_field_type,
                )*
            }

            impl $(<$($generics)*>)? $route_ty $(<$($generics)*>)? {
                /// Executes this route.
                pub async fn execute<C>(
                    client: &C
                    $(, $route_field: $route_field_type)*
                ) -> ::std::result::Result<
                    <Self as $crate::Route>::Response,
                    $crate::RequestError
                >
                where
                    C: $crate::RestClient<Self>,
                {
                    // Build request
                    let route = Self { $($route_field,)* };

                    // Execute request
                    $crate::RestClient::request(client, route).await
                }
            }

            impl $(<$($generics)*>)? ::std::fmt::Display for $route_ty $(<$($generics)*>)? {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    match self {
                        Self {
                            $($route_field,)*
                        } => write!(f, $route),
                    }
                }
            }

            #[$crate::async_trait::async_trait]
            impl $(<$($generics)*>)? $crate::Route for $route_ty $(<$($generics)*>)? {
                type Info = $crate::routes!(@info_type $($info_type,)? (),);
                type Response = $response;

                #[inline]
                fn info(&self) -> <Self as $crate::Route>::Info {
                    let Self {
                        $(ref $route_field,)*
                    } = self;

                    $(
                        let $info_method = $crate::reqwest::Method::$method;
                        let $info_route = $route;
                        $info
                    )?
                }

                fn create_request<F>(
                    &self,
                    request_factory: F
                ) -> $crate::reqwest::RequestBuilder
                where
                    F: for<'a> FnOnce(
                        $crate::reqwest::Method,
                        &'a str
                    ) -> $crate::reqwest::RequestBuilder
                {
                    let Self {
                        $(ref $route_field,)*
                    } = self;

                    // Build request
                    let path = ::std::string::ToString::to_string(self);
                    let request = request_factory(
                        $crate::reqwest::Method::$method,
                        &path
                    );

                    // Body
                    let request = $crate::routes!(
                        @req_body request,
                        $(
                            $body,
                            [$body_type]
                        )?
                    );

                    // Query string
                    let request = $crate::routes!(@query request, $($query)?);

                    // Processor
                    $(
                        let $req = request;
                        let request = $processor;
                    )?

                    request
                }

                async fn map_response(
                    &self,
                    response: $crate::reqwest::Response,
                ) -> Result<Self::Response, $crate::RequestError>
                {
                    // Parse response
                    $crate::routes!(
                        @res_body response,
                        [$res_body_type]
                    )
                    .await
                    .map_err(::std::convert::Into::into)
                }
            }
        )*
    };
}
