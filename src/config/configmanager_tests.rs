#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::configmanager::*;
    use crate::filters::NoIdAudience;
    use std::collections::HashMap;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    // Helper function to create a test config
    fn create_test_config(key: &str) -> ConfigBody {
        let project = Project {
            _id: format!("proj_{}", key),
            key: key.to_string(),
            a0_organization: "test_org".to_string(),
            settings: ProjectSettings {
                edgedb: EdgeDBSettings { enabled: false },
                optin: OptInSettings {
                    enabled: false,
                    title: String::new(),
                    description: String::new(),
                    image_url: String::new(),
                    colors: OptInColors {
                        primary: String::new(),
                        secondary: String::new(),
                    },
                },
                disable_passthrough_rollouts: false,
                obfuscation: None,
            },
        };

        let environment = Environment {
            _id: format!("env_{}", key),
            key: "test".to_string(),
        };

        let sse = SSE {
            hostname: "test.example.com".to_string(),
            path: "/test".to_string(),
        };

        // Audiences now owned, no Box::leak required
        let audiences: HashMap<String, NoIdAudience> = HashMap::new();

        ConfigBody {
            project,
            audiences,
            environment,
            features: vec![],
            variables: vec![],
            sse,
            variable_id_map: HashMap::new(),
            variable_key_map: HashMap::new(),
            variable_id_to_feature_map: HashMap::new(),
            etag: "test_etag".to_string(),
            ray_id: "test_ray".to_string(),
            last_modified: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_set_and_get_config() {
        let sdk_key = "test_key_1";
        let config = create_test_config(sdk_key);

        set_config(sdk_key, config);

        let retrieved = get_config(sdk_key);
        assert!(retrieved.is_some());

        let retrieved_config = retrieved.unwrap();
        assert_eq!(retrieved_config.project.key, sdk_key);
        assert_eq!(retrieved_config.project._id, format!("proj_{}", sdk_key));
    }

    #[test]
    fn test_has_config() {
        let sdk_key = "test_key_2";

        // Initially should not have the config
        assert!(!has_config(sdk_key));

        // Set the config
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        // Now should have the config
        assert!(has_config(sdk_key));
    }

    #[test]
    fn test_get_nonexistent_config() {
        let sdk_key = "nonexistent_key";
        let retrieved = get_config(sdk_key);
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_update_config() {
        let sdk_key = "test_key_3";

        // Set initial config
        let config1 = create_test_config("initial");
        set_config(sdk_key, config1);

        let retrieved1 = get_config(sdk_key).unwrap();
        assert_eq!(retrieved1.project.key, "initial");

        // Update with new config
        let config2 = create_test_config("updated");
        set_config(sdk_key, config2);

        let retrieved2 = get_config(sdk_key).unwrap();
        assert_eq!(retrieved2.project.key, "updated");
    }

    #[test]
    fn test_multiple_configs() {
        let keys = vec!["key_a", "key_b", "key_c"];

        // Set multiple configs
        for key in &keys {
            let config = create_test_config(key);
            set_config(key, config);
        }

        // Verify all configs exist and are correct
        for key in &keys {
            assert!(has_config(key));
            let config = get_config(key).unwrap();
            assert_eq!(config.project.key, *key);
        }
    }

    #[test]
    fn test_concurrent_reads() {
        let sdk_key = "concurrent_read_key";
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        let num_threads = 10;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];

        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Perform multiple reads
                for _ in 0..100 {
                    let config = get_config(&key);
                    assert!(config.is_some(), "Thread {} failed to get config", i);
                    let cfg = config.unwrap();
                    assert_eq!(cfg.project.key, key);
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_concurrent_reads_and_writes() {
        let sdk_key = "concurrent_rw_key";
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        let num_readers = 8;
        let num_writers = 2;
        let barrier = Arc::new(Barrier::new(num_readers + num_writers));
        let mut handles = vec![];

        // Spawn reader threads
        for i in 0..num_readers {
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for _ in 0..50 {
                    let config = get_config(&key);
                    assert!(config.is_some(), "Reader thread {} failed to get config", i);
                    thread::sleep(Duration::from_micros(10));
                }
            });

            handles.push(handle);
        }

        // Spawn writer threads
        for i in 0..num_writers {
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for j in 0..10 {
                    let config = create_test_config(&format!("writer_{}_{}", i, j));
                    set_config(&key, config);
                    thread::sleep(Duration::from_micros(50));
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Verify config still exists
        assert!(has_config(sdk_key));
    }

    #[test]
    fn test_arc_cloning_efficiency() {
        let sdk_key = "arc_test_key";
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        // Get multiple references - should clone Arc, not ConfigBody
        let ref1 = get_config(sdk_key).unwrap();
        let ref2 = get_config(sdk_key).unwrap();
        let ref3 = get_config(sdk_key).unwrap();

        // All should point to the same underlying data
        assert_eq!(Arc::strong_count(&ref1), 4); // 1 in HashMap + 3 clones
        assert_eq!(ref1.project.key, sdk_key);
        assert_eq!(ref2.project.key, sdk_key);
        assert_eq!(ref3.project.key, sdk_key);
    }

    #[test]
    fn test_concurrent_has_config_checks() {
        let sdk_key = "has_config_key";
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        let num_threads = 20;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];

        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for _ in 0..100 {
                    let exists = has_config(&key);
                    assert!(exists, "Thread {} failed to find config", i);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_concurrent_different_keys() {
        let num_keys = 10;
        let reads_per_key = 50;
        let barrier = Arc::new(Barrier::new(num_keys));
        let mut handles = vec![];

        // Set up configs for all keys
        for i in 0..num_keys {
            let key = format!("key_{}", i);
            let config = create_test_config(&key);
            set_config(&key, config);
        }

        // Each thread reads its own key
        for i in 0..num_keys {
            let barrier_clone = Arc::clone(&barrier);
            let key = format!("key_{}", i);

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for _ in 0..reads_per_key {
                    let config = get_config(&key);
                    assert!(config.is_some());
                    assert_eq!(config.unwrap().project.key, key);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_stress_concurrent_operations() {
        let base_key = "stress_test";
        let num_keys = 5;
        let num_threads_per_key = 4;
        let operations_per_thread = 100;

        // Initialize configs
        for i in 0..num_keys {
            let key = format!("{}_{}", base_key, i);
            let config = create_test_config(&key);
            set_config(&key, config);
        }

        let barrier = Arc::new(Barrier::new(num_keys * num_threads_per_key));
        let mut handles = vec![];

        for key_idx in 0..num_keys {
            for thread_idx in 0..num_threads_per_key {
                let barrier_clone = Arc::clone(&barrier);
                let key = format!("{}_{}", base_key, key_idx);

                let handle = thread::spawn(move || {
                    barrier_clone.wait();

                    for op in 0..operations_per_thread {
                        match op % 3 {
                            0 => {
                                // Read operation
                                let config = get_config(&key);
                                assert!(config.is_some());
                            }
                            1 => {
                                // Check operation
                                let exists = has_config(&key);
                                assert!(exists);
                            }
                            _ => {
                                // Write operation (every 3rd operation)
                                if thread_idx == 0 {
                                    let new_config = create_test_config(&key);
                                    set_config(&key, new_config);
                                }
                            }
                        }
                    }
                });

                handles.push(handle);
            }
        }

        for handle in handles {
            handle.join().expect("Thread panicked during stress test");
        }

        // Verify all configs still exist
        for i in 0..num_keys {
            let key = format!("{}_{}", base_key, i);
            assert!(has_config(&key));
        }
    }

    #[test]
    fn test_config_immutability_across_threads() {
        let sdk_key = "immutable_key";
        let config = create_test_config(sdk_key);
        let original_project_id = config.project._id.clone();
        set_config(sdk_key, config);

        let num_threads = 10;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];

        for _ in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();
            let expected_id = original_project_id.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for _ in 0..50 {
                    let config = get_config(&key).unwrap();
                    // Verify the data hasn't changed
                    assert_eq!(config.project._id, expected_id);
                    assert_eq!(config.project.key, key);
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_sequential_updates_with_concurrent_reads() {
        let sdk_key = "seq_update_key";
        let initial_config = create_test_config("version_0");
        set_config(sdk_key, initial_config);

        let num_readers = 10;
        let num_updates = 5;

        for update_num in 1..=num_updates {
            let barrier = Arc::new(Barrier::new(num_readers + 1));
            let mut handles = vec![];

            // Spawn reader threads
            for _ in 0..num_readers {
                let barrier_clone = Arc::clone(&barrier);
                let key = sdk_key.to_string();

                let handle = thread::spawn(move || {
                    barrier_clone.wait();

                    for _ in 0..20 {
                        let config = get_config(&key);
                        assert!(config.is_some());
                        thread::sleep(Duration::from_micros(5));
                    }
                });

                handles.push(handle);
            }

            // Update config while readers are active
            let barrier_clone = Arc::clone(&barrier);
            let key = sdk_key.to_string();
            let update_handle = thread::spawn(move || {
                barrier_clone.wait();
                thread::sleep(Duration::from_micros(50));

                let new_config = create_test_config(&format!("version_{}", update_num));
                set_config(&key, new_config);
            });

            handles.push(update_handle);

            // Wait for all threads
            for handle in handles {
                handle.join().expect("Thread panicked");
            }
        }

        // Verify final config
        let final_config = get_config(sdk_key).unwrap();
        assert_eq!(final_config.project.key, format!("version_{}", num_updates));
    }

    #[test]
    fn test_no_deadlock_scenario() {
        let keys = vec!["deadlock_key_1", "deadlock_key_2", "deadlock_key_3"];

        // Initialize configs
        for key in &keys {
            let config = create_test_config(key);
            set_config(key, config);
        }

        let num_threads = 15;
        let barrier = Arc::new(Barrier::new(num_threads));
        let mut handles = vec![];

        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let keys_clone = keys.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                // Each thread accesses keys in different orders
                let access_order = match i % 3 {
                    0 => vec![0, 1, 2],
                    1 => vec![2, 0, 1],
                    _ => vec![1, 2, 0],
                };

                for _ in 0..50 {
                    for &idx in &access_order {
                        let config = get_config(keys_clone[idx]);
                        assert!(config.is_some());
                    }
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked - possible deadlock");
        }
    }

    #[test]
    fn test_memory_safety_with_arc() {
        let sdk_key = "memory_safety_key";
        let config = create_test_config(sdk_key);
        set_config(sdk_key, config);

        // Get reference and hold it
        let config_ref1 = get_config(sdk_key).unwrap();

        // Update config (should not affect existing reference)
        let new_config = create_test_config("new_version");
        set_config(sdk_key, new_config);

        // Get new reference
        let config_ref2 = get_config(sdk_key).unwrap();

        // Old reference should still have old data
        assert_eq!(config_ref1.project.key, sdk_key);

        // New reference should have new data
        assert_eq!(config_ref2.project.key, "new_version");

        // Both are valid and independent
        assert_ne!(config_ref1.project.key, config_ref2.project.key);
    }
}
