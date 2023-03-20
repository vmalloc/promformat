use std::fmt::Write;

#[derive(Default)]
pub struct Metrics {
    buffer: String,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gauge<'a>(&'a mut self, name: &str, help: &str) -> MetricGroup<'a> {
        self.metric_group("gauge", name, help)
    }

    pub fn counter<'a>(&'a mut self, name: &str, help: &str) -> MetricGroup<'a> {
        self.metric_group("counter", name, help)
    }

    fn metric_group<'a>(
        &'a mut self,
        type_: &'static str,
        name: &str,
        help: &str,
    ) -> MetricGroup<'a> {
        MetricGroup::new(self, type_, name, help)
    }

    pub fn render(&self) -> &str {
        &self.buffer
    }
}

pub struct MetricGroup<'a> {
    metrics: &'a mut Metrics,
    name: String,
}

impl<'a> MetricGroup<'a> {
    fn new(metrics: &'a mut Metrics, metric_type: &str, name: &str, help: &str) -> Self {
        writeln!(&mut metrics.buffer, "# HELP {name} {help}").unwrap();
        writeln!(&mut metrics.buffer, "# TYPE {name} {metric_type}").unwrap();
        Self {
            name: name.to_owned(),
            metrics,
        }
    }

    pub fn label(&mut self, label: impl AsRef<str>, value: impl AsRef<str>) -> SingleMetric {
        SingleMetric {
            name: &self.name,
            metrics: self.metrics,
            labels: String::default(),
        }
        .label(label, value)
    }
}

pub struct SingleMetric<'a, 'b> {
    labels: String,
    name: &'a String,
    metrics: &'b mut Metrics,
}

impl<'a, 'b> SingleMetric<'a, 'b> {
    pub fn label(mut self, label: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let label = label.as_ref();
        let value = value.as_ref();
        if self.labels.is_empty() {
            self.labels.push('{')
        } else {
            self.labels.push(',')
        }

        // TODO: escaping
        write!(&mut self.labels, "{label}=\"{value}\"").unwrap();

        self
    }

    pub fn set(mut self, value: impl std::fmt::Display) {
        self.labels.push('}');
        writeln!(self.metrics.buffer, "{}{} {value}", self.name, self.labels).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge() {
        let mut metrics = Metrics::default();

        let mut gauge = metrics.gauge("testme", "help here");
        gauge.label("x", "y").set(2);
        gauge.label("a", "b").label("c", "d").set(20);

        assert_eq!(
            metrics.render(),
            r#"# HELP testme help here
# TYPE testme gauge
testme{x="y"} 2
testme{a="b",c="d"} 20
"#
        );
    }
}
