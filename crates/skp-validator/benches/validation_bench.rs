use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use skp_validator::Validate;
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Basic Models
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
struct SimpleUser {
    #[validate(required, length(min = 3, max = 50))]
    pub name: String,

    #[validate(required, email)]
    pub email: String,

    #[validate(range(min = 18, max = 120))]
    pub age: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct NestedData {
    #[validate(required)]
    pub id: String,
    
    #[validate(dive)]
    pub items: Vec<SimpleItem>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct SimpleItem {
    #[validate(range(min = 0))]
    pub value: i32,
}

// -----------------------------------------------------------------------------
// Complex / Recursive Models for Advanced Benchmarks
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
struct RecursiveNode {
    #[validate(length(min=1))]
    pub name: String,
    
    #[validate(dive)]
    pub children: Vec<Box<RecursiveNode>>,
}

// -----------------------------------------------------------------------------
// Benchmarks
// -----------------------------------------------------------------------------

fn bench_simple_validation(c: &mut Criterion) {
    let valid_user = SimpleUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(25),
    };
    
    let invalid_user = SimpleUser {
        name: "Al".to_string(),
        email: "invalid".to_string(),
        age: Some(10),
    };

    let mut group = c.benchmark_group("basic_validation");
    group.bench_function("valid_user", |b| b.iter(|| black_box(&valid_user).validate()));
    group.bench_function("invalid_user", |b| b.iter(|| black_box(&invalid_user).validate()));
    group.finish();
}

fn bench_high_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_throughput");
    group.sample_size(10); // Reduce sample size for large datasets
    
    // Benchmark varying collection sizes
    for size in [100, 1_000, 10_000].iter() {
        let items: Vec<SimpleItem> = (0..*size).map(|i| SimpleItem { value: i }).collect();
        let data = NestedData {
            id: "dataset".to_string(),
            items,
        };

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| black_box(&data).validate())
        });
    }
    group.finish();
}

fn bench_deep_recursion(c: &mut Criterion) {
    // Construct a deeply nested tree
    fn build_tree(depth: usize) -> RecursiveNode {
        if depth == 0 {
            RecursiveNode { name: "Leaf".into(), children: vec![] }
        } else {
            RecursiveNode { 
                name: format!("Node {}", depth),
                children: vec![Box::new(build_tree(depth - 1))] 
            }
        }
    }

    let deep_tree = build_tree(50); // Depth of 50
    
    let mut group = c.benchmark_group("advanced_validation");
    group.bench_function("recursive_depth_50", |b| b.iter(|| black_box(&deep_tree).validate()));
    group.finish();
}

criterion_group!(benches, bench_simple_validation, bench_high_throughput, bench_deep_recursion);
criterion_main!(benches);
