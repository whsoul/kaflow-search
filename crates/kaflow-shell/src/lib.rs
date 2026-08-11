//! `kaflow-shell` — Tauri desktop shell (public).
//!
//! ## The rule this crate lives by
//!
//! **It depends on no particular engine.** Everything goes through the `KafkaToolEngine`
//! trait. Break that and a build with only the mock engine stops working — which is the
//! same thing as saying this crate can no longer stand on its own.
//!
//! ## `all_handlers!`
//! Tauri accepts exactly one handler, and the macro that builds one cannot be composed.
//! So the list is exported as a macro here and a binary adds its own to it. **This file is
//! the only place the command list lives.**

pub mod app_paths;
pub mod commands;
pub mod tauri_emitter;

/// Combines the commands this crate owns with any the caller adds, into one handler.
///
/// ```ignore
/// .invoke_handler(kaflow_shell::all_handlers![
///     debug_commands::rocksdb_put,
///     bench::bench_write,
/// ])
/// ```
/// With nothing to add, call it with no arguments.
///
/// An added command may carry `#[cfg(...)]`, so it can be compiled in conditionally.
#[macro_export]
macro_rules! all_handlers {
    ($($(#[$attr:meta])* $extra:path),* $(,)?) => {
        ::tauri::generate_handler![
            $crate::commands::clear_workspace_index,
            $crate::commands::clear_workspace_meta,
            $crate::commands::clear_topic_index,
            $crate::commands::clear_topic_meta,
            $crate::commands::clear_all,
            $crate::commands::resync_meta,
            $crate::commands::list_topic_fields,
            $crate::commands::get_topic_index_sizes,
            $crate::commands::verify_topic_consistency,
            $crate::commands::get_global_ilm_config,
            $crate::commands::get_cluster_id,
            $crate::commands::get_disk_space,
            $crate::commands::get_kafka_version_info,
            $crate::commands::list_kafka_topics,
            $crate::commands::get_topic_message_count,
            $crate::commands::list_topic_message_counts,
            $crate::commands::get_topic_size_profile,
            $crate::commands::suggest_tokenize_fields,
            $crate::commands::fetch_kafka_messages,
            $crate::commands::open_kafka_topic,
            $crate::commands::cancel_indexing,
            $crate::commands::record_placeholder,
            $crate::commands::fetch_compact_key_history,
            $crate::commands::fetch_compact_deleted_keys_page,
            $crate::commands::get_topic_offset_status,
            $crate::commands::list_watched_lag,
            $crate::commands::cancel_search,
            $crate::commands::export_search_results,
            $crate::commands::cancel_export,
            $crate::commands::prefetch_search_locs,
            $crate::commands::multi_search,
            $crate::commands::multi_search_time_buckets,
            $crate::commands::multi_search_combined_buckets,
            $crate::commands::multi_search_locs_in_range,
            $crate::commands::multi_search_page,
            $crate::commands::multi_search_offset_buckets,
            $crate::commands::search_offset_buckets,
            $crate::commands::search_combined_buckets,
            $crate::commands::browse_offset_buckets,
            $crate::commands::browse_combined_buckets,
            $crate::commands::prefetch_browse_locs,
            $crate::commands::fetch_browse_locs_in_bucket,
            $crate::commands::fetch_browse_locs_in_ts_range,
            $crate::commands::fetch_browse_locs_page,
            $crate::commands::fetch_time_buckets,
            $crate::commands::fetch_search_time_buckets,
            $crate::commands::fetch_search_locs_in_range,
            $crate::commands::fetch_search_locs_page,
            $crate::commands::fetch_messages,
            $crate::commands::fetch_meta_rows,
            $crate::commands::test_deserializers,
            $crate::commands::trigger_background_ilm,
            $crate::commands::trigger_size_based_cleanup,
            $crate::commands::get_topic_config_info,
            $crate::commands::set_topic_cleanup_policy,
            $crate::commands::set_topic_meta_config,
            $crate::commands::set_global_ilm_config,
            $crate::commands::remove_topic_from_index,
            $crate::commands::set_indexed_fields,
            $crate::commands::reindex_fields_from_meta,
            $crate::commands::drop_fields_from_index,
            $crate::commands::list_cluster_profiles,
            $crate::commands::save_cluster_profile,
            $crate::commands::delete_cluster_profile,
            $crate::commands::list_registry_resources,
            $crate::commands::save_registry_resource,
            $crate::commands::delete_registry_resource,
            $crate::commands::test_registry_resource,
            $crate::commands::list_registry_subjects,
            $crate::commands::fetch_registry_subject_latest,
            $crate::commands::list_registry_schema_index,
            $crate::commands::save_workspace_cluster_id,
            $crate::commands::get_workspace_cluster_id,
            $crate::commands::force_topic_cleanup,
            $crate::commands::trigger_topic_retention_cleanup,
            $crate::commands::detect_topic_drift,
            $crate::commands::ensure_topic_watched,
            $crate::commands::set_topic_deserializers,
            $crate::commands::set_tokenize_fields,
            $crate::commands::read_schema_file,
            $crate::commands::read_proto_closure,
            $crate::commands::unwatch_topic,
            $crate::commands::fetch_registry_subject_schema,
            $crate::commands::mark_topic_auto_cleaned,
            $crate::commands::reset_workspace,
            $crate::commands::inspect_workspace_recovery,
            $crate::commands::check_workspace_storage,
            $crate::commands::apply_profile_limits,
            $crate::commands::get_effective_limits,
            $crate::commands::get_system_limits,
            $crate::commands::register_kafka_auth,
            $crate::commands::clear_kafka_auth,
            $crate::commands::verify_kafka_auth,
            $crate::commands::list_sasl_mechanisms,
            $crate::commands::list_aws_profiles,
            $crate::commands::resolve_aws_paths,
            $crate::commands::load_aws_profile,
            $crate::commands::is_debug_build_enabled,
            $crate::commands::build_mode_label,
            $crate::commands::get_engine_info,
            $crate::commands::get_host_info,
            $crate::commands::show_find_context_menu,
            $crate::commands::export_diagnostic_report,
            $crate::commands::fetch_cluster_topology,
            $($(#[$attr])* $extra),*
        ]
    };
}
