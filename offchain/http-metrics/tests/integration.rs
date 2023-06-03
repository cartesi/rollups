use clap::Parser;
use http_metrics::{
    config::MetricsCLIConfig, Counter, MetricsServer, Registry,
};

#[derive(Default)]
struct TestMetrics {
    pub a: Counter,
    pub b: Counter,
}

impl TestMetrics {
    fn new() -> Self {
        let a = Counter::default();
        let b = Counter::default();

        TestMetrics { a, b }
    }

    fn registry(&self) -> Registry {
        let mut registry = Registry::default();
        registry.register("counter1", "Counter 1", self.a.clone());
        registry.register("counter2", "Counter 2", self.b.clone());
        registry
    }
}

#[tokio::test]
async fn todo() {
    let metrics_server: MetricsServer =
        MetricsCLIConfig::parse().try_into().unwrap();

    let test_metrics = TestMetrics::new();

    let metrics_handle = metrics_server.run(test_metrics.registry()).await;

    let x = metrics_handle.await.unwrap().unwrap();
}
