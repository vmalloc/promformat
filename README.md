# What is this?

`promformat` is a small utility library to help formatting Prometheus metrics.

# Why?

In most cases exposing Prometheus metrics in Rust can be easily done using [the `prometheus` crate](https://docs.rs/prometheus/latest/prometheus/), using global counters registered as `lazy_static!`s or similar tricks. However this can have potential downsides to some use cases.

For example, you may want to set and unset metrics based on varying conditions, or register/unregister specific label sets. Such a task is tricky and cumbersome to achieve in the `prometheus` crate.

# Usage

Using `promformat` is pretty straightforward:

``` rust
use std::time::SystemTime;
use promformat::Metrics;

let mut metrics = Metrics::new();

let mut gauge1 = metrics.gauge("gauge_1", "Some gauge help text here");
gauge1.label("label1", "value1").label("label2", "value2").set(100);

let mut gauge2 = metrics.gauge("gauge_2", "Some other gaueg here");
gauge2.label("label2", "value2").set_with_timestamp(100, SystemTime::now());


let rendered = metrics.render();
```


# License

`promformat` is licensed under the Apache 2.0 license
