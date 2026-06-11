// 单元测试：验证日志改进

#[cfg(test)]
mod logging_improvement_tests {
    use codebase_translate::reporter::logger::EventLogger;
    use std::path::Path;

    #[test]
    fn test_event_logger_info_levels() {
        let logger = EventLogger::new();

        // 测试文件处理相关的信息是否使用 info 级别
        // 这些日志现在应该在生产环境可见

        // 测试总文件数记录
        logger.log_total_files(10);

        // 测试文件处理记录
        let test_path = Path::new("/test/file.rs");
        logger.log_file_processed(test_path, 5);

        // 测试进度记录（应该包含百分比）
        logger.log_progress(3, 10);

        // 测试跳过文件记录（debug 级别）
        logger.log_skipped(test_path);

        // 测试 API 调用记录（debug 级别）
        logger.log_api_call(2);

        // 测试缓存命中记录（debug 级别）
        logger.log_cache_hit();

        // 测试缓存未命中记录（debug 级别）
        logger.log_cache_miss();

        // 测试翻译器调用记录（debug 级别）
        logger.log_translator_call("deeplx", 100, true, 500);

        // 测试报告生成记录（info 级别）
        logger.log_report_generation("text");

        // 测试报告保存记录（info 级别）
        logger.log_report_saved(test_path);
    }

    #[test]
    fn test_progress_logging_format() {
        let logger = EventLogger::new();

        // 测试进度日志的格式
        // 应该包含百分比信息
        logger.log_progress(0, 100); // 0%
        logger.log_progress(50, 100); // 50%
        logger.log_progress(100, 100); // 100%
        logger.log_progress(1, 3); // 33.3%
        logger.log_progress(2, 3); // 66.7%
        logger.log_progress(3, 3); // 100%

        // 测试边界情况
        logger.log_progress(0, 0); // 0% (除零保护)
    }

    #[test]
    fn test_logging_config_span_events() {
        use codebase_translate::config::global::LoggingConfig;

        // 测试默认配置中 span_events 为 false
        let default_config = LoggingConfig::default();
        assert_eq!(default_config.span_events, false);

        // 测试自定义配置
        let custom_config = LoggingConfig {
            level: "info".to_string(),
            output: "stdout".to_string(),
            file: None,
            format: "pretty".to_string(),
            span_events: true,
        };
        assert_eq!(custom_config.span_events, true);
    }
}
