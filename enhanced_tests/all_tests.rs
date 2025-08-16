// ========== integration_tests.rs ==========
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::task;
    use insta::assert_json_snapshot;

    #[tokio::test]
    async fn test_basic_flow() {
        let result = perform_operation("valid_input").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "expected_output");
    }

    #[tokio::test]
    async fn test_empty_input() {
        let result = perform_operation("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_malformed_input() {
        let result = perform_operation("{bad json}").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_snapshot_output() {
        let result = perform_operation("structured_input").await.unwrap();
        assert_json_snapshot!(result);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let handles: Vec<_> = (0..10)
            .map(|_| task::spawn(perform_operation("valid_input")))
            .collect();

        for handle in handles {
            let res = handle.await.unwrap();
            assert!(res.is_ok());
        }
    }
}

// ========== performance_tests.rs ==========
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    fn bench_small_input(c: &mut Criterion) {
        c.bench_function("small_input", |b| b.iter(|| perform_sync("tiny")));
    }

    fn bench_large_input(c: &mut Criterion) {
        let input = "a".repeat(10000);
        c.bench_function("large_input", |b| b.iter(|| perform_sync(&input)));
    }

    criterion_group!(benches, bench_small_input, bench_large_input);
    criterion_main!(benches);
}

// ========== property_tests.rs ==========
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_round_trip(s in ".*") {
            let encoded = encode(&s);
            let decoded = decode(&encoded);
            prop_assert_eq!(decoded, s);
        }

        #[test]
        fn test_idempotent(s in ".*") {
            let once = normalize(&s);
            let twice = normalize(&once);
            prop_assert_eq!(once, twice);
        }

        #[test]
        fn test_combined_property(s in "[a-zA-Z0-9]{1,100}") {
            let norm = normalize(&s);
            let encoded = encode(&norm);
            let decoded = decode(&encoded);
            prop_assert_eq!(normalize(&decoded), norm);
        }

        #[test]
        fn test_non_panicking(s in proptest::collection::vec(".*", 1..100)) {
            for item in &s {
                let _ = perform_fallible(item);
            }
        }
    }
}