#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "assertions in tests"
)]

use piramid_hardware::compute::strategies::for_mode;
use piramid_hardware::compute::{
    cosine_similarity, dot_product, euclidean_distance, euclidean_distance_squared,
    DistanceKernels, ExecutionMode, Metric,
};

fn auto() -> &'static dyn DistanceKernels {
    for_mode(ExecutionMode::Auto).expect("Auto always resolves to an available CPU strategy")
}

#[test]
fn euclidean_distance_basic_cases() {
    let v = vec![1.0, 2.0, 3.0];
    assert_eq!(euclidean_distance(&v, &v, auto()), 0.0);

    let v1 = vec![0.0, 0.0];
    let v2 = vec![3.0, 4.0];
    let dist = euclidean_distance(&v1, &v2, auto());
    assert!((dist - 5.0).abs() < 1e-6);

    let sq = euclidean_distance_squared(&v1, &v2, auto());
    assert!((sq - 25.0).abs() < 1e-6);
}

#[test]
#[should_panic(expected = "Vectors must have same length")]
fn euclidean_rejects_mismatched_lengths() {
    euclidean_distance(&[1.0, 2.0], &[1.0], auto());
}

#[test]
fn dot_product_basic_cases() {
    let v1 = vec![1.0, 2.0, 3.0];
    let v2 = vec![4.0, 5.0, 6.0];
    let result = dot_product(&v1, &v2, auto());
    assert!((result - 32.0).abs() < 1e-6);

    let ortho = dot_product(&[1.0, 0.0], &[0.0, 1.0], auto());
    assert!(ortho.abs() < 1e-6);
}

#[test]
#[should_panic(expected = "Vectors must have same length")]
fn dot_rejects_mismatched_lengths() {
    dot_product(&[1.0, 2.0], &[1.0], auto());
}

#[test]
fn metric_calculate_cosine_and_euclidean() {
    let v1 = vec![1.0, 0.0];
    let v2 = vec![0.0, 1.0];
    assert!(Metric::Cosine.calculate(&v1, &v2, auto()).abs() < 1e-6);

    let euclid_sim = Metric::Euclidean.calculate(&v1, &v1, auto());
    assert!((euclid_sim - 1.0).abs() < 1e-6);
}

#[test]
fn cosine_similarity_cases() {
    let v = vec![1.0, 2.0, 3.0];
    let sim_same = cosine_similarity(&v, &v, auto());
    assert!((sim_same - 1.0).abs() < 1e-6);

    let sim_orth = cosine_similarity(&[1.0, 0.0], &[0.0, 1.0], auto());
    assert!(sim_orth.abs() < 1e-6);
}

// One spelling per metric. serde, `as_str` and `FromStr` previously disagreed: serde rendered
// DotProduct as "dotproduct" while the HTTP layer accepted only "dot", so the same value was
// spelled differently in the config file and on the wire.
#[test]
fn a_metric_spells_the_same_everywhere() {
    use piramid_hardware::compute::Metric;

    for metric in [Metric::Cosine, Metric::Euclidean, Metric::DotProduct] {
        let name = metric.as_str();
        assert_eq!(
            serde_json::to_string(&metric).unwrap(),
            format!("\"{name}\""),
            "serde and as_str disagree for {metric:?}"
        );
        assert_eq!(name.parse::<Metric>().unwrap(), metric, "FromStr disagrees");
    }
}

#[test]
fn an_unknown_metric_names_the_alternatives() {
    use piramid_hardware::compute::Metric;

    let error = "dot_product".parse::<Metric>().unwrap_err().to_string();
    assert!(error.contains("cosine"), "{error}");
    assert!(error.contains("dot"), "{error}");
}
