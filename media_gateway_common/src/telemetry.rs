use std::borrow::Cow;
use std::cell::OnceCell;

use http::HeaderMap;
use log::error;
use opentelemetry::global::Error;
use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry::{global, Context};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use parking_lot::Mutex;
use savant_core::message::Message;
use savant_core::otlp::PropagatedContext;
use savant_core::telemetry::{Configurator, TelemetryConfiguration};

static CONFIGURATOR: Mutex<OnceCell<Configurator>> = Mutex::new(OnceCell::new());

pub fn init(config: &TelemetryConfiguration) {
    let configurator = CONFIGURATOR.lock();
    match configurator.get() {
        Some(_) => panic!("Open Telemetry has been configured"),
        None => {
            let result = configurator.set(Configurator::new("media-gateway", config));
            if result.is_err() {
                // should not happen
                panic!("Error while configuring OpenTelemetry");
            }
            global::set_error_handler(|e| match e {
                Error::Propagation(pe)
                    if pe.to_string().contains(
                        "Cannot extract from invalid jaeger header format, JaegerPropagator",
                    ) => {}
                _ => {
                    error!(target: "opentelemetry", "{}", e);
                }
            })
            .expect("Error while configuring OpenTelemetry error handler");
        }
    }
}

pub fn shutdown() {
    let mut configurator = CONFIGURATOR.lock();
    if let Some(mut c) = configurator.take() {
        c.shutdown()
    }
}

pub fn get_context_with_span<T>(span_name: T, parent_ctx: &Context) -> Context
where
    T: Into<Cow<'static, str>>,
{
    if !parent_ctx.span().span_context().is_valid() {
        Context::default()
    } else {
        let tracer = global::tracer("");
        let span = tracer
            .span_builder(span_name)
            .with_kind(SpanKind::Internal)
            .start_with_context(&tracer, parent_ctx);
        Context::default().with_span(span)
    }
}

pub fn get_message_context(message: &Message) -> Context {
    global::get_text_map_propagator(|p| p.extract(&message.meta().span_context))
}

pub fn get_header_context(headers: &HeaderMap) -> Context {
    global::get_text_map_propagator(|p| p.extract(&HeaderExtractor(headers)))
}

pub fn propagate_header_context(headers: &mut HeaderMap, ctx: &Context) {
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(ctx, &mut HeaderInjector(headers))
    })
}

pub fn get_propagated_context(ctx: &Context) -> PropagatedContext {
    let mut propagated_ctx = PropagatedContext::new();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(ctx, &mut propagated_ctx)
    });
    propagated_ctx
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use http::{HeaderMap, HeaderName, HeaderValue};
    use opentelemetry::trace::{TraceContextExt, TraceFlags, Tracer};
    use opentelemetry::{global, Context};
    use savant_core::message::Message;
    use savant_core::telemetry::TelemetryConfiguration;

    use crate::telemetry::{
        get_context_with_span, get_header_context, get_message_context, get_propagated_context,
        init, propagate_header_context,
    };

    const TRACE_ID: &str = "d4532e7a51d51c29c2166a58b7e0916a";
    const SPAN_ID: &str = "12ba53d3c291b8f1";
    const TRACE_FLAGS: u8 = 1;
    const VENDOR_KEY: &str = "vendor1_key";
    const VENDOR_VALUE: &str = "vendor1_value";
    const W3C_TRACE_PARENT_KEY: &str = "traceparent";
    const W3C_TRACE_PARENT_VALUE: &str = "00-d4532e7a51d51c29c2166a58b7e0916a-12ba53d3c291b8f1-01";
    const W3C_TRACE_STATE_KEY: &str = "tracestate";
    const W3C_TRACE_STATE_VALUE: &str = "vendor1_key=vendor1_value";

    static INIT: Once = Once::new();

    fn init_telemetry() {
        INIT.call_once(|| init(&TelemetryConfiguration::no_op()))
    }

    #[test]
    fn test_get_message_context() {
        init_telemetry();

        let mut message = Message::unknown("message".to_string());
        let meta = message.meta_mut();
        meta.span_context.0.insert(
            W3C_TRACE_PARENT_KEY.to_string(),
            W3C_TRACE_PARENT_VALUE.to_string(),
        );
        meta.span_context.0.insert(
            W3C_TRACE_STATE_KEY.to_string(),
            W3C_TRACE_STATE_VALUE.to_string(),
        );

        let result = get_message_context(&message);
        let result_span = result.span();
        let result_span_context = result_span.span_context();

        assert_eq!(result_span_context.trace_id().to_string(), TRACE_ID);
        assert_eq!(result_span_context.span_id().to_string(), SPAN_ID);
        assert_eq!(
            result_span_context.trace_flags(),
            TraceFlags::new(TRACE_FLAGS)
        );
        assert_eq!(
            result_span_context.trace_state().get(VENDOR_KEY),
            Some(VENDOR_VALUE)
        );
    }

    #[test]
    fn test_get_header_context() {
        init_telemetry();

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::try_from(W3C_TRACE_PARENT_KEY).unwrap(),
            HeaderValue::try_from(W3C_TRACE_PARENT_VALUE).unwrap(),
        );
        headers.insert(
            HeaderName::try_from(W3C_TRACE_STATE_KEY).unwrap(),
            HeaderValue::try_from(W3C_TRACE_STATE_VALUE).unwrap(),
        );

        let result = get_header_context(&headers);
        let result_span = result.span();
        let result_span_context = result_span.span_context();

        assert_eq!(result_span_context.trace_id().to_string(), TRACE_ID);
        assert_eq!(result_span_context.span_id().to_string(), SPAN_ID);
        assert_eq!(
            result_span_context.trace_flags(),
            TraceFlags::new(TRACE_FLAGS)
        );
        assert_eq!(
            result_span_context.trace_state().get(VENDOR_KEY),
            Some(VENDOR_VALUE)
        );
    }

    #[test]
    fn test_propagate_header_context() {
        init_telemetry();

        let mut headers = HeaderMap::new();

        let (trace_id, span_id, trace_flags) = global::tracer("").in_span("test", |ctx| {
            let span = ctx.span();
            let span_context = span.span_context();

            propagate_header_context(&mut headers, &ctx);

            (
                span_context.trace_id(),
                span_context.span_id(),
                span_context.trace_flags(),
            )
        });

        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers
                .get(W3C_TRACE_STATE_KEY)
                .map(|e| e.to_str().unwrap()),
            Some("")
        );
        assert_eq!(
            headers
                .get(W3C_TRACE_PARENT_KEY)
                .map(|e| e.to_str().unwrap()),
            Some(format!("00-{}-{}-{:02x}", trace_id, span_id, trace_flags.to_u8()).as_str())
        );
    }

    #[test]
    fn test_get_propagated_context() {
        init_telemetry();

        let (trace_id, span_id, trace_flags, result) = global::tracer("").in_span("test", |ctx| {
            let span = ctx.span();
            let span_context = span.span_context();

            (
                span_context.trace_id(),
                span_context.span_id(),
                span_context.trace_flags(),
                get_propagated_context(&ctx),
            )
        });

        assert_eq!(result.0.len(), 2);
        assert_eq!(
            result.0.get(W3C_TRACE_STATE_KEY).map(|e| e.as_str()),
            Some("")
        );
        assert_eq!(
            result.0.get(W3C_TRACE_PARENT_KEY).map(|e| e.as_str()),
            Some(format!("00-{}-{}-{:02x}", trace_id, span_id, trace_flags.to_u8()).as_str())
        );
    }

    #[test]
    fn test_get_context_with_span_invalid_parent() {
        init_telemetry();

        let result = get_context_with_span("test", &Context::default());

        assert_eq!(result.span().span_context().is_valid(), false);
    }

    #[test]
    fn test_get_context_with_span_valid_parent() {
        init_telemetry();

        let (trace_id, result) = global::tracer("").in_span("parent", |ctx| {
            let child_ctx = get_context_with_span("child", &ctx);

            (ctx.span().span_context().trace_id(), child_ctx)
        });

        let result_span = result.span();
        let result_span_context = result_span.span_context();

        assert_eq!(result_span_context.is_valid(), true);
        assert_eq!(result_span_context.trace_id(), trace_id);
    }
}
