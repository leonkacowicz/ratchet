//! Minimal Rust fixture: functions with branching so the structural collector
//! has cyclomatic/cognitive metrics to measure.

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn classify(n: i32) -> &'static str {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}
