///Shorthand for `OwnedView<WatchRequestView<'static>>`.
pub type OwnedWatchRequestView = ::buffa::view::OwnedView<
    crate::proto::example::v1::__buffa::view::WatchRequestView<'static>,
>;
///Shorthand for `OwnedView<ItemView<'static>>`.
pub type OwnedItemView = ::buffa::view::OwnedView<
    crate::proto::example::v1::__buffa::view::ItemView<'static>,
>;
///Shorthand for `OwnedView<SummaryView<'static>>`.
pub type OwnedSummaryView = ::buffa::view::OwnedView<
    crate::proto::example::v1::__buffa::view::SummaryView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::example::v1::Item>
for crate::proto::example::v1::__buffa::view::ItemView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::example::v1::Item>
for ::buffa::view::OwnedView<
    crate::proto::example::v1::__buffa::view::ItemView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
impl ::connectrpc::Encodable<crate::proto::example::v1::Summary>
for crate::proto::example::v1::__buffa::view::SummaryView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::example::v1::Summary>
for ::buffa::view::OwnedView<
    crate::proto::example::v1::__buffa::view::SummaryView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
    /// An `OwnedView` still holds the buffer it was decoded from, so
    /// its large fields can be handed to the response body by
    /// reference count instead of copied. The bare view impl above
    /// cannot do this: it has borrows but no buffer to name.
    fn encode_segments(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::connectrpc::EncodedBody, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body_segments(
            self.reborrow(),
            self.bytes(),
            codec,
        )
    }
}
/// Full service name for this service.
pub const FEED_SERVICE_SERVICE_NAME: &str = "example.v1.FeedService";
/// Static [`Spec`](::connectrpc::Spec) for the `Watch` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const FEED_SERVICE_WATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/example.v1.FeedService/Watch",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Collect` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const FEED_SERVICE_COLLECT_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/example.v1.FeedService/Collect",
        ::connectrpc::StreamType::ClientStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the `Echo` RPC, as seen by the server; the generated client passes it with [`origin`](::connectrpc::Spec::origin) `Client` (compare across sides with [`Spec::same_method`](::connectrpc::Spec::same_method)).
pub const FEED_SERVICE_ECHO_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/example.v1.FeedService/Echo",
        ::connectrpc::StreamType::BidiStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Moves items in every direction a method can stream.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// [`InboundStream<Req>`](::connectrpc::InboundStream) — a
/// `ServiceStream` of [`StreamMessage`](::connectrpc::StreamMessage)s.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.9.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity)]
pub trait FeedService: Send + Sync + 'static {
    /// Stream the first `count` items of the feed, one reply per item.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn watch(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::example::v1::WatchRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::example::v1::Item,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
    /// Collect a stream of items into one summary.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// Each `requests` item is a [`StreamMessage`](::connectrpc::StreamMessage):
    /// it owns its buffer, is `Send + 'static`, and exposes zero-copy
    /// accessor methods (`item.name()`), `.view()`, and
    /// `.to_owned_message()`.
    fn collect<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::InboundStream<crate::proto::example::v1::Item>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::example::v1::Summary,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Echo each item back as it arrives.
    ///
    /// Each `requests` item is a [`StreamMessage`](::connectrpc::StreamMessage):
    /// it owns its buffer, is `Send + 'static`, and exposes zero-copy
    /// accessor methods (`item.name()`), `.view()`, and
    /// `.to_owned_message()`.
    /// Items can be yielded back unchanged
    /// (`StreamMessage<M>` implements `Encodable<M>`).
    fn echo(
        &self,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::InboundStream<crate::proto::example::v1::Item>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::example::v1::Item,
                > + Send + use<Self>,
            >,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
/// Prefer [`Router::add_service`](::connectrpc::Router::add_service) for
/// top-down registration; `register` remains available for compatibility
/// and cases where the service-first call shape is more convenient.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait FeedServiceExt: FeedService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: FeedService> FeedServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view_server_stream::<
                _,
                _,
                crate::proto::example::v1::Item,
            >(
                FEED_SERVICE_SERVICE_NAME,
                "Watch",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::example::v1::__buffa::view::WatchRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::example::v1::WatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.watch(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(FEED_SERVICE_WATCH_SPEC)
            .route_view_client_stream(
                FEED_SERVICE_SERVICE_NAME,
                "Collect",
                ::connectrpc::view_client_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |ctx, req, format| {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let req = ::connectrpc::dispatcher::codegen::into_stream_messages::<
                                crate::proto::example::v1::Item,
                            >(req);
                            svc.collect(ctx, req)
                                .await?
                                .encode::<crate::proto::example::v1::Summary>(format)
                        }
                    }
                }),
            )
            .with_spec(FEED_SERVICE_COLLECT_SPEC)
            .route_view_bidi_stream::<
                _,
                _,
                crate::proto::example::v1::Item,
            >(
                FEED_SERVICE_SERVICE_NAME,
                "Echo",
                ::connectrpc::view_bidi_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |ctx, req| {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let req = ::connectrpc::dispatcher::codegen::into_stream_messages::<
                                crate::proto::example::v1::Item,
                            >(req);
                            svc.echo(ctx, req).await
                        }
                    }
                }),
            )
            .with_spec(FEED_SERVICE_ECHO_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct FeedServiceRegisterMarker;
impl<S: FeedService> ::connectrpc::ServiceRegister<FeedServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as FeedServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `FeedService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = FeedServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct FeedServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: FeedService> FeedServiceServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for FeedServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: FeedService> ::connectrpc::Dispatcher for FeedServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("example.v1.FeedService/")?;
        match method {
            "Watch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(FEED_SERVICE_WATCH_SPEC),
                )
            }
            "Collect" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::client_streaming()
                        .with_spec(FEED_SERVICE_COLLECT_SPEC),
                )
            }
            "Echo" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::bidi_streaming()
                        .with_spec(FEED_SERVICE_ECHO_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("example.v1.FeedService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("example.v1.FeedService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Watch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::example::v1::WatchRequest,
                    >(request, format)?;
                    let req: crate::proto::example::v1::__buffa::view::WatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                        ctx.decode_options(),
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::example::v1::WatchRequest,
                    >::from_parts(&req, &body);
                    let resp = svc.watch(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::proto::example::v1::Item,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("example.v1.FeedService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            "Collect" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let req_stream = ::connectrpc::dispatcher::codegen::decode_message_request_stream::<
                        crate::proto::example::v1::Item,
                    >(requests, format, ctx.decode_options().clone());
                    svc.collect(ctx, req_stream)
                        .await?
                        .encode::<crate::proto::example::v1::Summary>(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("example.v1.FeedService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            "Echo" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let req_stream = ::connectrpc::dispatcher::codegen::decode_message_request_stream::<
                        crate::proto::example::v1::Item,
                    >(requests, format, ctx.decode_options().clone());
                    let resp = svc.echo(ctx, req_stream).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::proto::example::v1::Item,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = FeedServiceClient::new(conn, config);
/// let response = client.watch(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = FeedServiceClient::new(http, config);
/// let response = client.watch(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.watch(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.watch(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[cfg(feature = "client")]
#[derive(Clone)]
pub struct FeedServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> FeedServiceClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::connectrpc::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the Watch RPC. Sends a request to /example.v1.FeedService/Watch.
    pub async fn watch(
        &self,
        request: crate::proto::example::v1::WatchRequest,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::example::v1::__buffa::view::ItemView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.watch_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Watch RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn watch_with_options(
        &self,
        request: crate::proto::example::v1::WatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::example::v1::__buffa::view::ItemView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                FEED_SERVICE_WATCH_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                request,
                options,
            )
            .await
    }
    /// Call the Collect RPC. Sends a request to /example.v1.FeedService/Collect.
    ///
    /// `requests` is any `Stream<Item = ...> + Send + 'static` of
    /// request messages (the `ClientRequestStream` bound); messages
    /// are sent as the stream yields them. It backs the request
    /// body, so yield owned messages or feed the call from a
    /// channel-backed stream. For a collection that is already in
    /// hand, wrap it with `::connectrpc::stream_iter(...)`.
    ///
    /// Dropping the returned future cancels the call: the request
    /// body is dropped along with it, so messages the stream had
    /// not yet yielded are never delivered. A caller that needs the
    /// request delivered must drive the call to completion rather
    /// than, say, wrapping it in a `timeout`.
    pub async fn collect(
        &self,
        requests: impl ::connectrpc::client::ClientRequestStream<
            crate::proto::example::v1::Item,
        >,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::example::v1::__buffa::view::SummaryView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.collect_with_options(requests, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Collect RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    ///
    /// `requests` is any `Stream<Item = ...> + Send + 'static` of
    /// request messages (the `ClientRequestStream` bound); messages
    /// are sent as the stream yields them. It backs the request
    /// body, so yield owned messages or feed the call from a
    /// channel-backed stream. For a collection that is already in
    /// hand, wrap it with `::connectrpc::stream_iter(...)`.
    ///
    /// Dropping the returned future cancels the call: the request
    /// body is dropped along with it, so messages the stream had
    /// not yet yielded are never delivered. A caller that needs the
    /// request delivered must drive the call to completion rather
    /// than, say, wrapping it in a `timeout`.
    pub async fn collect_with_options(
        &self,
        requests: impl ::connectrpc::client::ClientRequestStream<
            crate::proto::example::v1::Item,
        >,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::example::v1::__buffa::view::SummaryView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_client_stream(
                &self.transport,
                &self.config,
                FEED_SERVICE_COLLECT_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                requests,
                options,
            )
            .await
    }
    /// Call the Echo RPC. Sends a request to /example.v1.FeedService/Echo.
    pub async fn echo(
        &self,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::proto::example::v1::Item,
            crate::proto::example::v1::__buffa::view::ItemView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.echo_with_options(::connectrpc::client::CallOptions::default()).await
    }
    /// Call the Echo RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn echo_with_options(
        &self,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::BidiStream<
            T::ResponseBody,
            crate::proto::example::v1::Item,
            crate::proto::example::v1::__buffa::view::ItemView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_bidi_stream(
                &self.transport,
                &self.config,
                FEED_SERVICE_ECHO_SPEC.with_origin(::connectrpc::SpecOrigin::Client),
                options,
            )
            .await
    }
}
