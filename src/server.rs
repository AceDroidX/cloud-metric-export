use axum::{
    extract::State, http::header::CONTENT_TYPE, response::IntoResponse, routing::get, Router,
};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

pub async fn start_metrics_server(metrics_addr: SocketAddr, registry: Arc<Registry>) {
    info!("Starting metrics server on {}", metrics_addr);

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/healthz", get(health_handler))
        .with_state(registry);

    let listener = tokio::net::TcpListener::bind(metrics_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler(State(registry): State<Arc<Registry>>) -> impl IntoResponse {
    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();

    (
        [(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        buffer,
    )
}

async fn health_handler() -> impl IntoResponse {
    "OK"
}

#[cfg(test)]
pub fn create_app(registry: Arc<Registry>) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/healthz", get(health_handler))
        .with_state(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use prometheus_client::metrics::family::Family;
    use prometheus_client::metrics::gauge::Gauge;
    use tower::ServiceExt;

    #[derive(Clone, Debug, Hash, PartialEq, Eq, prometheus_client::encoding::EncodeLabelSet)]
    struct TestLabel {
        name: String,
    }

    #[tokio::test]
    async fn test_metrics_endpoint_empty() {
        let registry = Arc::new(Registry::default());
        let app = create_app(registry);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(
            content_type,
            "application/openmetrics-text; version=1.0.0; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.ends_with("# EOF\n"));
    }

    #[tokio::test]
    async fn test_metrics_endpoint_with_data() {
        let mut registry = Registry::default();
        let gauge: Family<TestLabel, Gauge> = Family::default();
        registry.register("test_metric", "Test metric", gauge.clone());

        gauge
            .get_or_create(&TestLabel {
                name: "test".to_string(),
            })
            .set(42);

        let registry = Arc::new(registry);
        let app = create_app(registry);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("# HELP test_metric Test metric."));
        assert!(body_str.contains("# TYPE test_metric gauge"));
        assert!(body_str.contains(r#"test_metric{name="test"} 42"#));
        assert!(body_str.ends_with("# EOF\n"));
    }

    #[tokio::test]
    async fn test_metrics_endpoint_not_found() {
        let registry = Arc::new(Registry::default());
        let app = create_app(registry);

        let request = Request::builder()
            .uri("/not-found")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_endpoint_with_multiple_metrics() {
        let mut registry = Registry::default();

        let gauge1: Family<TestLabel, Gauge> = Family::default();
        let gauge2: Family<TestLabel, Gauge> = Family::default();

        registry.register("metric_a", "Metric A", gauge1.clone());
        registry.register("metric_b", "Metric B", gauge2.clone());

        gauge1
            .get_or_create(&TestLabel {
                name: "test".to_string(),
            })
            .set(100);
        gauge2
            .get_or_create(&TestLabel {
                name: "test".to_string(),
            })
            .set(200);

        let registry = Arc::new(registry);
        let app = create_app(registry);

        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("metric_a"));
        assert!(body_str.contains("metric_b"));
        assert!(body_str.contains(r#"metric_a{name="test"} 100"#));
        assert!(body_str.contains(r#"metric_b{name="test"} 200"#));
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let registry = Arc::new(Registry::default());
        let app = create_app(registry);

        let request = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "OK");
    }
}
