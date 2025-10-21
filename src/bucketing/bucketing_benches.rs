use chrono::Utc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use devcycle_bucketing_rs::bucketing::generate_bucketed_config;
use devcycle_bucketing_rs::bucketing::variable_for_user;
use devcycle_bucketing_rs::config::platform_data::{
    get_platform_data, set_platform_data, PlatformData,
};
use devcycle_bucketing_rs::events::EventQueueOptions;
use devcycle_bucketing_rs::init_event_queue;
use devcycle_bucketing_rs::user::PopulatedUser;
use serde_json::Value;
use std::collections::HashMap;

const BENCH_SDK_KEY: &str = "benchmark-sdk-key";

// Setup functions to initialize test data
fn setup_platform_data() {
    let platform_data = PlatformData {
        sdk_type: "server".to_string(),
        sdk_version: "1.0.0".to_string(),
        platform_version: "1.0.0".to_string(),
        device_model: "benchmark-device".to_string(),
        platform: "benchmark".to_string(),
        hostname: "localhost".to_string(),
    };
    set_platform_data(BENCH_SDK_KEY.to_string(), platform_data);
}

fn create_test_user(user_id: &str) -> PopulatedUser {
    PopulatedUser {
        user_id: user_id.to_string(),
        email: format!("{}@benchmark.com", user_id),
        name: format!("Benchmark User {}", user_id),
        language: "en".to_string(),
        country: "US".to_string(),
        app_version: "1.0.0".to_string(),
        app_build: "100".to_string(),
        custom_data: HashMap::new(),
        private_custom_data: HashMap::new(),
        device_model: "benchmark-device".to_string(),
        last_seen_date: Utc::now(),
        platform_data: get_platform_data(BENCH_SDK_KEY).unwrap(),
        created_date: Utc::now(),
    }
}

fn create_test_user_with_custom_data(user_id: &str) -> PopulatedUser {
    let mut custom_data: HashMap<String, Value> = HashMap::new();
    custom_data.insert(
        "age".to_string(),
        Value::Number(serde_json::Number::from(25)),
    );
    custom_data.insert("tier".to_string(), Value::String("premium".to_string()));
    custom_data.insert("beta_user".to_string(), Value::Bool(true));

    PopulatedUser {
        user_id: user_id.to_string(),
        email: format!("{}@benchmark.com", user_id),
        name: format!("Benchmark User {}", user_id),
        language: "en".to_string(),
        country: "US".to_string(),
        app_version: "1.0.0".to_string(),
        app_build: "100".to_string(),
        custom_data,
        private_custom_data: HashMap::new(),
        device_model: "benchmark-device".to_string(),
        last_seen_date: Utc::now(),
        platform_data: get_platform_data(BENCH_SDK_KEY).unwrap(),
        created_date: Utc::now(),
    }
}

// Benchmark for generate_bucketed_config with PopulatedUser
fn bench_generate_bucketed_config(c: &mut Criterion) {
    setup_platform_data();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("generate_bucketed_config");

    // Benchmark with basic user (no custom data)
    group.bench_function("basic_user", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let user = create_test_user("bench_user");
                let client_custom_data = HashMap::new();

                unsafe {
                    generate_bucketed_config(BENCH_SDK_KEY.to_string(), user, client_custom_data)
                        .await
                        .ok()
                }
            })
        });
    });

    // Benchmark with user that has custom data
    group.bench_function("user_with_custom_data", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let user = create_test_user_with_custom_data("bench_user_custom");
                let client_custom_data = HashMap::new();

                unsafe {
                    generate_bucketed_config(BENCH_SDK_KEY.to_string(), user, client_custom_data)
                        .await
                        .ok()
                }
            })
        });
    });

    // Benchmark with varying numbers of users to see scaling
    for user_count in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiple_users", user_count),
            user_count,
            |b, &count| {
                b.iter(|| {
                    runtime.block_on(async move {
                        for i in 0..count {
                            let user = create_test_user(&format!("bench_user_{}", i));
                            let client_custom_data = HashMap::new();

                            unsafe {
                                generate_bucketed_config(
                                    BENCH_SDK_KEY.to_string(),
                                    user,
                                    client_custom_data,
                                )
                                .await
                                .ok()
                            };
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

// Benchmark comparing different user scenarios
fn bench_user_scenarios(c: &mut Criterion) {
    setup_platform_data();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("user_scenarios");

    // Different countries
    for country in ["US", "CA", "GB", "DE", "FR"].iter() {
        group.bench_with_input(
            BenchmarkId::new("country", country),
            country,
            |b, &country_code| {
                b.iter(|| {
                    runtime.block_on(async move {
                        let mut user = create_test_user("country_bench");
                        user.country = country_code.to_string();
                        let client_custom_data = HashMap::new();

                        unsafe {
                            generate_bucketed_config(
                                BENCH_SDK_KEY.to_string(),
                                user,
                                client_custom_data,
                            )
                            .await
                            .ok()
                        }
                    })
                });
            },
        );
    }

    // Different amounts of custom data
    for data_count in [0, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("custom_data_fields", data_count),
            data_count,
            |b, &count| {
                b.iter(|| {
                    runtime.block_on(async move {
                        let mut user = create_test_user("custom_data_bench");
                        for i in 0..count {
                            user.custom_data.insert(
                                format!("field_{}", i),
                                Value::String(format!("value_{}", i)),
                            );
                        }
                        let client_custom_data = HashMap::new();

                        unsafe {
                            generate_bucketed_config(
                                BENCH_SDK_KEY.to_string(),
                                user,
                                client_custom_data,
                            )
                            .await
                            .ok()
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

// Throughput benchmark - how many requests per second
fn bench_throughput(c: &mut Criterion) {
    setup_platform_data();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("throughput");
    group.sample_size(50); // Reduce sample size for throughput tests

    group.bench_function("sequential_100_requests", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for i in 0..100 {
                    let user = create_test_user(&format!("throughput_user_{}", i));
                    let client_custom_data = HashMap::new();

                    unsafe {
                        generate_bucketed_config(
                            BENCH_SDK_KEY.to_string(),
                            user,
                            client_custom_data,
                        )
                        .await
                        .ok()
                    };
                }
            })
        });
    });

    group.bench_function("sequential_1000_requests", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for i in 0..1000 {
                    let user = create_test_user(&format!("throughput_user_{}", i));
                    let client_custom_data = HashMap::new();

                    unsafe {
                        generate_bucketed_config(
                            BENCH_SDK_KEY.to_string(),
                            user,
                            client_custom_data,
                        )
                        .await
                        .ok()
                    };
                }
            })
        });
    });

    group.finish();
}

// Benchmark for variable_for_user
fn bench_variable_for_user(c: &mut Criterion) {
    setup_platform_data();
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("variable_for_user");

    // Benchmark with basic user
    group.bench_function("basic_user", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let user = create_test_user("bench_var_user");
                let client_custom_data = HashMap::new();

                unsafe {
                    variable_for_user(
                        BENCH_SDK_KEY,
                        user,
                        "test_variable",
                        "String",
                        client_custom_data,
                    )
                    .await
                    .ok()
                }
            })
        });
    });

    // Benchmark with user that has custom data
    group.bench_function("user_with_custom_data", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let user = create_test_user_with_custom_data("bench_var_user_custom");
                let client_custom_data = HashMap::new();

                unsafe {
                    variable_for_user(
                        BENCH_SDK_KEY,
                        user,
                        "test_variable",
                        "String",
                        client_custom_data,
                    )
                    .await
                    .ok()
                }
            })
        });
    });

    // Benchmark with varying numbers of variable lookups
    for var_count in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiple_variables", var_count),
            var_count,
            |b, &count| {
                b.iter(|| {
                    runtime.block_on(async move {
                        // Initialize event queue via manager
                        init_event_queue(BENCH_SDK_KEY.clone(), EventQueueOptions::default())
                            .await
                            .unwrap();

                        for i in 0..count {
                            let user = create_test_user(&format!("bench_var_user_{}", i));
                            let client_custom_data = HashMap::new();

                            unsafe {
                                variable_for_user(
                                    BENCH_SDK_KEY,
                                    user,
                                    "test_variable",
                                    "String",
                                    client_custom_data,
                                )
                                .await
                                .ok()
                            };
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_generate_bucketed_config,
    bench_user_scenarios,
    bench_throughput,
    bench_variable_for_user
);
criterion_main!(benches);
