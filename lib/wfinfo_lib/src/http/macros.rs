#[macro_export]
macro_rules! routes {
    (@req_body $builder:expr, [$body_type:ident] $body:expr) => {
        $builder.$body_type($body)
    };
    (@req_body $builder:expr, $body:expr) => {
        $crate::routes!(@req_body $builder, [json] $body)
    };
    (@req_body $builder:expr,) => {
        $builder
    };
    (@res_body $res:expr, [$body_type:ident]) => {
        $res.$body_type()
    };
    (@res_body $res:expr,) => {
        $crate::routes!(@res_body $res, [json])
    };
    (@query $builder:expr, $query:expr) => {
        $builder.query($query)
    };
    (@query $builder:expr,) => {
        $builder
    };
    (@info_type $info_type:ty) => {
        $info_type
    };
    (@info_type) => {
        ()
    };
    (
        $(
            (
                $name:ident {
                    $($url_param:ident : $url_param_type:ty),*
                    $(,)?
                }
                $(, body = $([$body_type:ident])? $body_param:ident : $body_param_type:ty)?
                $(, extra = {
                    $($extra_field:ident : $extra_field_type:ty),*
                    $(,)?
                })?
                , method = $method:ident $route:tt
                $(, info = |$info_method:pat_param, $info_route:pat_param| -> $info_type:ty $info:block)?
                $(, query = $query:expr)?
                $(, processor = |$req:pat_param| $processor:expr)?
                , helper = $([$res_body_type:ident])? $response:ty
                $(,)?
            )
        ),*
        $(,)?
    ) => {
        $(
            #[derive(Clone, Debug)]
            pub struct $name {
                $(pub $url_param: $url_param_type,)*
                $(pub $body_param: $body_param_type,)?
                $($(pub $extra_field: $extra_field_type,)*)?
            }

            impl ::std::fmt::Display for $name {
                fn fmt(
                    &self,
                    f: &mut ::std::fmt::Formatter<'_>
                ) -> ::std::fmt::Result {
                    let Self { $(ref $url_param,)* .. } = self;

                    write!(f, $route, $($url_param = $url_param),*)
                }
            }

            impl $crate::http::Route for $name {
                type Info = $crate::routes!(@info_type $($info_type)?);

                #[inline]
                fn info(&self) -> <Self as $crate::http::Route>::Info {
                    let Self {
                        $(ref $url_param,)*
                        $(ref $body_param,)?
                        $($(ref $extra_field,)*)?
                    } = self;

                    $(
                        let $info_method = $crate::reqwest::Method::$method;
                        let $info_route = $route;
                        $info
                    )?
                }

                #[inline]
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
                        $(ref $url_param,)*
                        $(ref $body_param,)?
                        $($(ref $extra_field,)*)?
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
                        $($([$body_type])? &$body_param)?
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
            }

            impl $name {
                pub async fn execute<C>(
                    client: &C
                    $(, $url_param: $url_param_type)*
                    $(, $body_param: $body_param_type)?
                    $($(, $extra_field: $extra_field_type)*)?
                ) -> ::std::result::Result<
                    $response,
                    $crate::http::RequestError
                >
                where
                    C: $crate::http::RestClient<
                        <Self as $crate::http::Route>::Info
                    >,
                {
                    // Build request
                    let route = Self {
                        $($url_param,)*
                        $($body_param,)?
                        $($($extra_field,)*)?
                    };

                    // Execute request
                    let response = $crate::http::RestClient::request(
                        client,
                        route
                    ).await?;

                    // Parse response
                    let response =
                        $crate::routes!(
                            @res_body response,
                            $([$res_body_type])?
                        )
                        .await
                        .map_err(
                            $crate::http::RequestError::ResponseParseError
                        )?;

                    ::std::result::Result::Ok(response)
                }
            }
        )*
    };
}
