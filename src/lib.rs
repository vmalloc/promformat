use std::fmt::Write;

pub trait TimestampLike {
    fn as_prometheus_timestamp(&self) -> i64;
}

impl TimestampLike for std::time::SystemTime {
    fn as_prometheus_timestamp(&self) -> i64 {
        self.duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
}

#[cfg(feature = "chrono")]
impl<H: chrono::TimeZone> TimestampLike for chrono::DateTime<H> {
    fn as_prometheus_timestamp(&self) -> i64 {
        self.timestamp_millis()
    }
}

#[cfg(feature = "time")]
const NANOS_PER_MILLI: i128 = 1_000_000;

#[cfg(feature = "time")]
impl TimestampLike for time::OffsetDateTime {
    fn as_prometheus_timestamp(&self) -> i64 {
        (self.unix_timestamp_nanos() / NANOS_PER_MILLI) as i64
    }
}

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

    pub fn into_rendered(self) -> String {
        self.buffer
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

    pub fn label(
        &mut self,
        label: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> SingleMetric<'_, '_> {
        SingleMetric {
            name: &self.name,
            metrics: self.metrics,
            labels: String::default(),
        }
        .label(label, value)
    }

    pub fn set(self, value: impl std::fmt::Display) {
        SingleMetric {
            name: &self.name,
            metrics: self.metrics,
            labels: String::from("{"),
        }
        .set(value)
    }

    pub fn set_with_timestamp(self, value: impl std::fmt::Display, timestamp: impl TimestampLike) {
        SingleMetric {
            name: &self.name,
            metrics: self.metrics,
            labels: String::from("{"),
        }
        .set_with_timestamp(value, timestamp)
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

    pub fn set_with_timestamp(
        mut self,
        value: impl std::fmt::Display,
        timestamp: impl TimestampLike,
    ) {
        self.labels.push('}');
        let unix_millis = timestamp.as_prometheus_timestamp();
        writeln!(
            self.metrics.buffer,
            "{}{} {value} {unix_millis}",
            self.name, self.labels
        )
        .unwrap();
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

    #[test]
    fn test_gauge_no_labels() {
        let mut metrics = Metrics::default();

        metrics.gauge("testme", "help here").set(20);

        assert_eq!(
            metrics.render(),
            r#"# HELP testme help here
# TYPE testme gauge
testme{} 20
"#
        );
    }

    #[test]
    fn test_gauge_timestamp_systemtime() {
        let mut metrics = Metrics::default();
        let now = std::time::UNIX_EPOCH + std::time::Duration::from_secs(259200);

        metrics
            .gauge("testme", "help here")
            .set_with_timestamp(20, now);

        assert_eq!(
            metrics.render(),
            r#"# HELP testme help here
# TYPE testme gauge
testme{} 20 259200000
"#
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_gauge_timestamp_chrono() {
        use chrono::{TimeZone, Utc};

        let mut metrics = Metrics::default();
        let now = Utc.with_ymd_and_hms(2025, 8, 19, 21, 31, 5).unwrap();

        metrics
            .gauge("testme", "help here")
            .set_with_timestamp(20, now);

        assert_eq!(
            metrics.render(),
            r#"# HELP testme help here
# TYPE testme gauge
testme{} 20 1755639065000
"#
        );
    }

    #[cfg(feature = "time")]
    #[test]
    fn test_gauge_timestamp_time() {
        use time::{Date, Month, OffsetDateTime, Time};

        let mut metrics = Metrics::default();
        let now = OffsetDateTime::new_utc(
            Date::from_calendar_date(2025, Month::August, 19).unwrap(),
            Time::from_hms_nano(21, 31, 5, 0).unwrap(),
        );

        metrics
            .gauge("testme", "help here")
            .set_with_timestamp(20, now);

        assert_eq!(
            metrics.render(),
            r#"# HELP testme help here
# TYPE testme gauge
testme{} 20 1755639065000
"#
        );
    }
}

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
