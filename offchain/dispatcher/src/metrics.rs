use http_metrics::{Counter, Metrics, Registry};

#[derive(Debug, Default)]
pub struct DispatcherMetrics {
    pub claims_sent_total: Counter,
    pub advance_inputs_sent_total: Counter,
    pub finish_epochs_sent_total: Counter,
}

impl Metrics for DispatcherMetrics {
    fn registry(&self) -> Registry {
        let mut registry = Registry::default();
        registry.register(
            "claims_sent",
            "Counts the number of claims sent",
            self.claims_sent_total.clone(),
        );
        registry.register(
            "advance_inputs_sent",
            "Counts the number of advance inputs sent",
            self.advance_inputs_sent_total.clone(),
        );
        registry.register(
            "finish_epochs_sent",
            "Counts the number of finish epochs sent",
            self.finish_epochs_sent_total.clone(),
        );
        registry
    }
}
