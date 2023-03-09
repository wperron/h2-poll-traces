use std::time::Duration;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use opentelemetry::{
    sdk::{trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::instrument;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[instrument(skip(_req))]
async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let response = Response::builder().status(200).body(Body::empty()).unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let addr = ([127, 0, 0, 1], 9898).into();
    println!("Listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(serve_req))
    }));

    if let Err(err) = serve_future.await {
        eprintln!("server error: {}", err);
    }
}

fn setup_tracing() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
                .with_timeout(Duration::from_secs(5)),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "h2-poll-traces",
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    // NOTE(wperron): I'm not filtering the spans by level here to demo the issue.
    let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry().with(trace_layer).init();
}
