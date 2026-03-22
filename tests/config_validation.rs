//! 配置验证文件
//!
//! 本文件用于验证配置加载功能，将解析结果写入 tests/config_test 目录
//! 配置来源：
//!   - e:\project\codebase-translator\.translator.toml (项目配置)
//!   - e:\project\codebase-translator\translator.toml (全局配置)
//!   - e:\project\codebase-translator\.env (环境变量)

use std::fs;
use std::path::{Path, PathBuf};

use codebase_translate::config::{ConfigLoader, EnvLoader, GlobalConfig, ProjectConfig};

/// Provider 可用性状态
#[derive(Debug, Clone)]
struct ProviderAvailability {
    provider_type: String,  // "deeplx", "llm", "tencent"
    enabled: bool,          // 是否在 enabled_providers 列表中
    available: bool,        // 是否可用（配置完整）
    reason: Option<String>, // 不可用的原因
}

/// 配置验证结果
#[derive(Debug)]
struct ConfigValidationResult {
    global_config_loaded: bool,
    project_config_loaded: bool,
    env_file_loaded: bool,
    enabled_providers: Vec<String>,
    provider_availabilities: Vec<ProviderAvailability>,
    deeplx_configured: bool,
    llm_configured: bool,
    tencent_configured: bool,
    deeplx_api_key_configured: bool,
    llm_api_keys_configured: Vec<(String, bool)>, // (provider_id, is_configured)
    tencent_credentials_configured: (bool, bool), // (secret_id, secret_key)
    global_config_details: Option<GlobalConfigSummary>,
    project_config_details: Option<ProjectConfigSummary>,
    errors: Vec<String>,
}

/// 全局配置摘要（不包含敏感信息）
#[derive(Debug)]
struct GlobalConfigSummary {
    provider: String,
    enabled_providers: Vec<String>,
    deeplx: DeepLXSummary,
    llm: LLMSummary,
    tencent: TencentSummary,
    logging: LoggingSummary,
}

#[derive(Debug)]
struct DeepLXSummary {
    api_url: String,
    api_key_configured: bool,
    proxy_configured: bool,
    rate_limit: u32,
    max_retries: u32,
    enabled: bool,
    available: bool,
}

#[derive(Debug)]
struct LLMSummary {
    health_check_enabled: bool,
    provider_count: usize,
    providers: Vec<LLMProviderSummary>,
    enabled: bool,
    available: bool,
    available_providers_count: usize,
}

#[derive(Debug)]
struct LLMProviderSummary {
    id: String,
    name: String,
    model: String,
    model_list: Vec<String>,
    is_multi_model: bool,
    max_tokens: u32,
    temperature: f32,
    base_url: String,
    api_keys_configured: bool,
    api_key_count: usize,
    proxy_configured: bool,
    timeout: u64,
    rate_limit: u32,
    available: bool,
    unavailable_reason: Option<String>,
}

#[derive(Debug)]
struct TencentSummary {
    secret_id_configured: bool,
    secret_key_configured: bool,
    region: String,
    proxy_configured: bool,
    timeout: u64,
    rate_limit: u32,
    max_retries: u32,
    enabled: bool,
    available: bool,
}

#[derive(Debug)]
struct LoggingSummary {
    level: String,
    format: String,
    output: String,
    file: Option<String>,
}

/// 项目配置摘要
#[derive(Debug)]
struct ProjectConfigSummary {
    source_langs: Vec<String>,
    target_lang: String,
    provider: String,
    include_patterns_count: usize,
    exclude_patterns_count: usize,
    cache_enabled: bool,
    cache_directory: String,
    writer_backup: bool,
    writer_dry_run: bool,
    extraction_comments: bool,
    extraction_doc_strings: bool,
    extraction_error_messages: bool,
    extraction_format_strings: bool,
}

fn main() {
    println!("========================================");
    println!("      配置验证工具");
    println!("========================================\n");

    let result = validate_configs();

    let output_dir = PathBuf::from("tests/config_test");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    write_validation_report(&output_dir, &result);
    write_detailed_config(&output_dir, &result);

    println!("\n验证完成！");
    println!("报告已写入: {}", output_dir.display());
}

fn validate_configs() -> ConfigValidationResult {
    let mut result = ConfigValidationResult {
        global_config_loaded: false,
        project_config_loaded: false,
        env_file_loaded: false,
        enabled_providers: Vec::new(),
        provider_availabilities: Vec::new(),
        deeplx_configured: false,
        llm_configured: false,
        tencent_configured: false,
        deeplx_api_key_configured: false,
        llm_api_keys_configured: Vec::new(),
        tencent_credentials_configured: (false, false),
        global_config_details: None,
        project_config_details: None,
        errors: Vec::new(),
    };

    // 1. 加载环境变量文件
    let env_path = PathBuf::from("e:\\project\\codebase-translator\\.env");
    if env_path.exists() {
        let env_loader = EnvLoader::new(vec![env_path.to_string_lossy().to_string()]);
        match env_loader.load() {
            Ok(_) => {
                println!("[✓] 环境变量文件加载成功: {}", env_path.display());
                result.env_file_loaded = true;
            }
            Err(e) => {
                println!("[✗] 环境变量文件加载失败: {}", e);
                result.errors.push(format!("Env load error: {}", e));
            }
        }
    } else {
        println!("[!] 环境变量文件不存在: {}", env_path.display());
        result.errors.push("Env file not found".to_string());
    }

    // 2. 加载全局配置
    let global_config_path = PathBuf::from("e:\\project\\codebase-translator\\translator.toml");
    let global_config = if global_config_path.exists() {
        let loader = ConfigLoader::new().with_global_config(&global_config_path);
        match loader.load_global() {
            Ok(config) => {
                println!("[✓] 全局配置加载成功: {}", global_config_path.display());
                result.global_config_loaded = true;
                Some(config)
            }
            Err(e) => {
                println!("[✗] 全局配置加载失败: {}", e);
                result.errors.push(format!("Global config error: {}", e));
                None
            }
        }
    } else {
        println!("[!] 全局配置文件不存在: {}", global_config_path.display());
        result
            .errors
            .push("Global config file not found".to_string());
        None
    };

    // 3. 加载项目配置
    let project_config_path = PathBuf::from("e:\\project\\codebase-translator\\.translator.toml");
    let project_config = if project_config_path.exists() {
        let loader = ConfigLoader::new().with_project_config(&project_config_path);
        match loader.load_project() {
            Ok(config) => {
                println!("[✓] 项目配置加载成功: {}", project_config_path.display());
                result.project_config_loaded = true;
                Some(config)
            }
            Err(e) => {
                println!("[✗] 项目配置加载失败: {}", e);
                result.errors.push(format!("Project config error: {}", e));
                None
            }
        }
    } else {
        println!("[!] 项目配置文件不存在: {}", project_config_path.display());
        result
            .errors
            .push("Project config file not found".to_string());
        None
    };

    // 4. 分析全局配置
    if let Some(ref config) = global_config {
        result.enabled_providers = config.get_enabled_providers();
        println!("\n--- 全局配置分析 ---");
        println!("启用的翻译器: {:?}", result.enabled_providers);

        // DeepLX 配置检查
        result.deeplx_configured = result.enabled_providers.contains(&"deeplx".to_string());
        result.deeplx_api_key_configured = config
            .deeplx
            .api_key
            .as_ref()
            .map(|k| !k.is_empty() && !k.starts_with("${"))
            .unwrap_or(false);

        let deeplx_available = result.deeplx_configured;
        result.provider_availabilities.push(ProviderAvailability {
            provider_type: "deeplx".to_string(),
            enabled: result.deeplx_configured,
            available: deeplx_available,
            reason: if deeplx_available {
                None
            } else {
                Some("未在 enabled_providers 中启用".to_string())
            },
        });

        println!("\n[DeepLX]");
        println!(
            "  启用状态: {}",
            if result.deeplx_configured {
                "✓ 已启用"
            } else {
                "✗ 未启用"
            }
        );
        println!(
            "  可用状态: {}",
            if deeplx_available {
                "✓ 可用"
            } else {
                "✗ 不可用"
            }
        );
        println!(
            "  API Key: {}",
            if result.deeplx_api_key_configured {
                "已配置"
            } else {
                "未配置"
            }
        );

        // LLM 配置检查
        result.llm_configured = result.enabled_providers.contains(&"llm".to_string());
        println!("\n[LLM]");
        println!(
            "  启用状态: {}",
            if result.llm_configured {
                "✓ 已启用"
            } else {
                "✗ 未启用"
            }
        );
        println!("  供应商总数: {}", config.llm.providers.len());

        let mut available_providers = 0;
        for provider in &config.llm.providers {
            let has_valid_key = !provider.api_keys.is_empty()
                && provider.api_keys.iter().any(|k| {
                    !k.is_empty() && !k.starts_with("${") && k != "xxx" && k != "your-api-key"
                });

            let is_available = has_valid_key
                && !provider.base_url.is_empty()
                && (!provider.model.is_empty() || !provider.model_list.is_empty());

            if is_available {
                available_providers += 1;
            }

            result
                .llm_api_keys_configured
                .push((provider.id.clone(), has_valid_key));

            let model_count = if provider.model_list.is_empty() {
                1
            } else {
                provider.model_list.len()
            };
            let is_multi_model = model_count > 1;

            println!("\n  供应商 '{}' ({}):", provider.name, provider.id);
            println!(
                "    - 可用状态: {}",
                if is_available {
                    "✓ 可用"
                } else {
                    "✗ 不可用"
                }
            );
            println!(
                "    - API Key: {}",
                if has_valid_key {
                    "已配置"
                } else {
                    "未配置"
                }
            );
            println!(
                "    - 模型数量: {} {}",
                model_count,
                if is_multi_model {
                    "(多模型轮询)"
                } else {
                    ""
                }
            );
            if is_multi_model {
                println!("    - 模型列表:");
                for (i, model) in provider.model_list.iter().enumerate() {
                    println!("      {}. {}", i + 1, model);
                }
            } else if !provider.model.is_empty() {
                println!("    - 模型: {}", provider.model);
            }
        }

        let llm_available = result.llm_configured && available_providers > 0;
        result.provider_availabilities.push(ProviderAvailability {
            provider_type: "llm".to_string(),
            enabled: result.llm_configured,
            available: llm_available,
            reason: if !result.llm_configured {
                Some("未在 enabled_providers 中启用".to_string())
            } else if available_providers == 0 {
                Some("没有可用的供应商（API Key 或配置不完整）".to_string())
            } else {
                None
            },
        });
        println!(
            "\n  LLM 整体可用状态: {} (可用供应商: {}/{})",
            if llm_available {
                "✓ 可用"
            } else {
                "✗ 不可用"
            },
            available_providers,
            config.llm.providers.len()
        );

        // Tencent 配置检查
        result.tencent_configured = result.enabled_providers.contains(&"tencent".to_string());
        let secret_id_ok = config
            .tencent
            .secret_id
            .as_ref()
            .map(|s| !s.is_empty() && !s.starts_with("${"))
            .unwrap_or(false);
        let secret_key_ok = config
            .tencent
            .secret_key
            .as_ref()
            .map(|s| !s.is_empty() && !s.starts_with("${"))
            .unwrap_or(false);
        result.tencent_credentials_configured = (secret_id_ok, secret_key_ok);

        let tencent_available = result.tencent_configured && secret_id_ok && secret_key_ok;
        result.provider_availabilities.push(ProviderAvailability {
            provider_type: "tencent".to_string(),
            enabled: result.tencent_configured,
            available: tencent_available,
            reason: if !result.tencent_configured {
                Some("未在 enabled_providers 中启用".to_string())
            } else if !secret_id_ok {
                Some("Secret ID 未配置".to_string())
            } else if !secret_key_ok {
                Some("Secret Key 未配置".to_string())
            } else {
                None
            },
        });

        println!("\n[Tencent Cloud]");
        println!(
            "  启用状态: {}",
            if result.tencent_configured {
                "✓ 已启用"
            } else {
                "✗ 未启用"
            }
        );
        println!(
            "  可用状态: {}",
            if tencent_available {
                "✓ 可用"
            } else {
                "✗ 不可用"
            }
        );
        println!(
            "  Secret ID: {}",
            if secret_id_ok {
                "已配置"
            } else {
                "未配置"
            }
        );
        println!(
            "  Secret Key: {}",
            if secret_key_ok {
                "已配置"
            } else {
                "未配置"
            }
        );

        // 构建全局配置摘要
        result.global_config_details = Some(build_global_config_summary(config, &result));
    }

    // 5. 分析项目配置
    if let Some(ref config) = project_config {
        println!("\n--- 项目配置分析 ---");
        println!("源语言: {:?}", config.get_source_langs());
        println!("目标语言: {}", config.translate.target_lang);
        println!("包含模式数量: {}", config.get_include_patterns().len());
        println!("排除模式数量: {}", config.get_exclude_patterns().len());
        println!(
            "缓存启用: {}",
            if config.cache.enabled { "是" } else { "否" }
        );
        println!("缓存目录: {}", config.cache.directory);
        println!(
            "Writer 备份: {}",
            if config.writer.backup { "是" } else { "否" }
        );
        println!(
            "Writer 预览模式: {}",
            if config.writer.dry_run { "是" } else { "否" }
        );

        result.project_config_details = Some(build_project_config_summary(config));
    }

    result
}

fn build_global_config_summary(
    config: &GlobalConfig,
    result: &ConfigValidationResult,
) -> GlobalConfigSummary {
    // 计算各 provider 的可用性
    let deeplx_available = result
        .provider_availabilities
        .iter()
        .find(|p| p.provider_type == "deeplx")
        .map(|p| p.available)
        .unwrap_or(false);

    let llm_available = result
        .provider_availabilities
        .iter()
        .find(|p| p.provider_type == "llm")
        .map(|p| p.available)
        .unwrap_or(false);

    let tencent_available = result
        .provider_availabilities
        .iter()
        .find(|p| p.provider_type == "tencent")
        .map(|p| p.available)
        .unwrap_or(false);

    let available_llm_providers: Vec<LLMProviderSummary> = config
        .llm
        .providers
        .iter()
        .map(|p| {
            let has_valid_key = !p.api_keys.is_empty()
                && p.api_keys.iter().any(|k| {
                    !k.is_empty() && !k.starts_with("${") && k != "xxx" && k != "your-api-key"
                });
            let is_available = has_valid_key
                && !p.base_url.is_empty()
                && (!p.model.is_empty() || !p.model_list.is_empty());

            let unavailable_reason = if !has_valid_key {
                Some("API Key 未配置或无效".to_string())
            } else if p.base_url.is_empty() {
                Some("Base URL 为空".to_string())
            } else if p.model.is_empty() && p.model_list.is_empty() {
                Some("模型未配置".to_string())
            } else {
                None
            };

            LLMProviderSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                model: p.model.clone(),
                model_list: p.model_list.clone(),
                is_multi_model: p.model_list.len() > 1,
                max_tokens: p.max_tokens,
                temperature: p.temperature,
                base_url: p.base_url.clone(),
                api_keys_configured: has_valid_key,
                api_key_count: p.api_keys.len(),
                proxy_configured: p.proxy_url.is_some(),
                timeout: p.timeout,
                rate_limit: p.rate_limit,
                available: is_available,
                unavailable_reason,
            }
        })
        .collect();

    let available_llm_count = available_llm_providers
        .iter()
        .filter(|p| p.available)
        .count();

    GlobalConfigSummary {
        provider: config.provider.to_string(),
        enabled_providers: config.get_enabled_providers(),
        deeplx: DeepLXSummary {
            api_url: config.deeplx.api_url.clone(),
            api_key_configured: config
                .deeplx
                .api_key
                .as_ref()
                .map(|k| !k.is_empty() && !k.starts_with("${"))
                .unwrap_or(false),
            proxy_configured: config.deeplx.proxy_url.is_some(),
            rate_limit: config.deeplx.rate_limit,
            max_retries: config.deeplx.max_retries,
            enabled: result.deeplx_configured,
            available: deeplx_available,
        },
        llm: LLMSummary {
            health_check_enabled: config.llm.health_check.enabled,
            provider_count: config.llm.providers.len(),
            providers: available_llm_providers,
            enabled: result.llm_configured,
            available: llm_available,
            available_providers_count: available_llm_count,
        },
        tencent: TencentSummary {
            secret_id_configured: config
                .tencent
                .secret_id
                .as_ref()
                .map(|s| !s.is_empty() && !s.starts_with("${"))
                .unwrap_or(false),
            secret_key_configured: config
                .tencent
                .secret_key
                .as_ref()
                .map(|s| !s.is_empty() && !s.starts_with("${"))
                .unwrap_or(false),
            region: config.tencent.region.clone(),
            proxy_configured: config.tencent.proxy_url.is_some(),
            timeout: config.tencent.timeout,
            rate_limit: config.tencent.rate_limit,
            max_retries: config.tencent.max_retries,
            enabled: result.tencent_configured,
            available: tencent_available,
        },
        logging: LoggingSummary {
            level: config.logging.level.clone(),
            format: config.logging.format.clone(),
            output: config.logging.output.clone(),
            file: config.logging.file.clone(),
        },
    }
}

fn build_project_config_summary(config: &ProjectConfig) -> ProjectConfigSummary {
    ProjectConfigSummary {
        source_langs: config.get_source_langs(),
        target_lang: config.translate.target_lang.clone(),
        provider: config.translate.provider.to_string(),
        include_patterns_count: config.get_include_patterns().len(),
        exclude_patterns_count: config.get_exclude_patterns().len(),
        cache_enabled: config.cache.enabled,
        cache_directory: config.cache.directory.clone(),
        writer_backup: config.writer.backup,
        writer_dry_run: config.writer.dry_run,
        extraction_comments: config.extraction.comments,
        extraction_doc_strings: config.extraction.doc_strings,
        extraction_error_messages: config.extraction.error_messages,
        extraction_format_strings: config.extraction.format_strings,
    }
}

fn write_validation_report(output_dir: &Path, result: &ConfigValidationResult) {
    let report_path = output_dir.join("validation_report.txt");

    let mut report = String::new();
    report.push_str("========================================\n");
    report.push_str("      配置验证报告\n");
    report.push_str("========================================\n\n");

    report.push_str("配置文件来源:\n");
    report.push_str("  - 全局配置: e:\\project\\codebase-translator\\translator.toml\n");
    report.push_str("  - 项目配置: e:\\project\\codebase-translator\\.translator.toml\n");
    report.push_str("  - 环境变量: e:\\project\\codebase-translator\\.env\n\n");

    report.push_str("----------------------------------------\n");
    report.push_str("加载状态\n");
    report.push_str("----------------------------------------\n");
    report.push_str(&format!(
        "环境变量文件: {}\n",
        if result.env_file_loaded {
            "✓ 已加载"
        } else {
            "✗ 未加载"
        }
    ));
    report.push_str(&format!(
        "全局配置: {}\n",
        if result.global_config_loaded {
            "✓ 已加载"
        } else {
            "✗ 未加载"
        }
    ));
    report.push_str(&format!(
        "项目配置: {}\n",
        if result.project_config_loaded {
            "✓ 已加载"
        } else {
            "✗ 未加载"
        }
    ));
    report.push('\n');

    // Provider 可用性汇总
    report.push_str("----------------------------------------\n");
    report.push_str("Provider 可用性汇总\n");
    report.push_str("----------------------------------------\n");
    for provider in &result.provider_availabilities {
        report.push_str(&format!("[{}]\n", provider.provider_type.to_uppercase()));
        report.push_str(&format!(
            "  启用状态: {}\n",
            if provider.enabled {
                "✓ 已启用"
            } else {
                "✗ 未启用"
            }
        ));
        report.push_str(&format!(
            "  可用状态: {}\n",
            if provider.available {
                "✓ 可用"
            } else {
                "✗ 不可用"
            }
        ));
        if let Some(ref reason) = provider.reason {
            report.push_str(&format!("  原因: {}\n", reason));
        }
        report.push('\n');
    }

    if result.global_config_loaded {
        report.push_str("----------------------------------------\n");
        report.push_str("翻译器配置详情\n");
        report.push_str("----------------------------------------\n");
        report.push_str(&format!("启用的翻译器: {:?}\n\n", result.enabled_providers));

        // DeepLX
        report.push_str("[DeepLX]\n");
        report.push_str(&format!(
            "  已启用: {}\n",
            if result.deeplx_configured {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  API Key 已配置: {}\n\n",
            if result.deeplx_api_key_configured {
                "是"
            } else {
                "否"
            }
        ));

        // LLM
        report.push_str("[LLM]\n");
        report.push_str(&format!(
            "  已启用: {}\n",
            if result.llm_configured { "是" } else { "否" }
        ));
        report.push_str("  供应商 API Key 状态:\n");
        for (provider_id, is_configured) in &result.llm_api_keys_configured {
            report.push_str(&format!(
                "    - {}: {}\n",
                provider_id,
                if *is_configured {
                    "已配置"
                } else {
                    "未配置"
                }
            ));
        }
        report.push('\n');

        // Tencent
        report.push_str("[Tencent Cloud]\n");
        report.push_str(&format!(
            "  已启用: {}\n",
            if result.tencent_configured {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  Secret ID 已配置: {}\n",
            if result.tencent_credentials_configured.0 {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  Secret Key 已配置: {}\n\n",
            if result.tencent_credentials_configured.1 {
                "是"
            } else {
                "否"
            }
        ));
    }

    if let Some(ref details) = result.global_config_details {
        report.push_str("----------------------------------------\n");
        report.push_str("全局配置详情\n");
        report.push_str("----------------------------------------\n");
        report.push_str(&format!("默认翻译器: {}\n", details.provider));
        report.push_str(&format!(
            "启用的翻译器: {:?}\n\n",
            details.enabled_providers
        ));

        report.push_str("[DeepLX 配置]\n");
        report.push_str(&format!(
            "  启用状态: {}\n",
            if details.deeplx.enabled {
                "已启用"
            } else {
                "未启用"
            }
        ));
        report.push_str(&format!(
            "  可用状态: {}\n",
            if details.deeplx.available {
                "可用"
            } else {
                "不可用"
            }
        ));
        report.push_str(&format!("  API URL: {}\n", details.deeplx.api_url));
        report.push_str(&format!(
            "  API Key: {}\n",
            if details.deeplx.api_key_configured {
                "已配置"
            } else {
                "未配置"
            }
        ));
        report.push_str(&format!(
            "  Proxy: {}\n",
            if details.deeplx.proxy_configured {
                "已配置"
            } else {
                "未配置"
            }
        ));
        report.push_str(&format!(
            "  速率限制: {} 请求/秒\n",
            details.deeplx.rate_limit
        ));
        report.push_str(&format!(
            "  最大重试: {} 次\n\n",
            details.deeplx.max_retries
        ));

        report.push_str("[LLM 配置]\n");
        report.push_str(&format!(
            "  启用状态: {}\n",
            if details.llm.enabled {
                "已启用"
            } else {
                "未启用"
            }
        ));
        report.push_str(&format!(
            "  可用状态: {}\n",
            if details.llm.available {
                "可用"
            } else {
                "不可用"
            }
        ));
        report.push_str(&format!(
            "  健康检查: {}\n",
            if details.llm.health_check_enabled {
                "启用"
            } else {
                "禁用"
            }
        ));
        report.push_str(&format!("  供应商总数: {}\n", details.llm.provider_count));
        report.push_str(&format!(
            "  可用供应商: {}/{}\n",
            details.llm.available_providers_count, details.llm.provider_count
        ));

        for provider in &details.llm.providers {
            report.push_str(&format!(
                "\n  供应商: {} ({}):\n",
                provider.name, provider.id
            ));
            report.push_str(&format!(
                "    - 可用状态: {}\n",
                if provider.available {
                    "✓ 可用"
                } else {
                    "✗ 不可用"
                }
            ));
            if let Some(ref reason) = provider.unavailable_reason {
                report.push_str(&format!("    - 不可用原因: {}\n", reason));
            }

            // 多模型轮询展示
            if provider.is_multi_model {
                report.push_str(&format!(
                    "    - 模型模式: 多模型轮询 (共 {} 个模型)\n",
                    provider.model_list.len()
                ));
                report.push_str("    - 模型列表:\n");
                for (i, model) in provider.model_list.iter().enumerate() {
                    let is_primary =
                        i == 0 && provider.model.is_empty() || model == &provider.model;
                    report.push_str(&format!(
                        "      {}. {}{}\n",
                        i + 1,
                        model,
                        if is_primary { " (主模型)" } else { "" }
                    ));
                }
            } else {
                report.push_str("    - 模型模式: 单模型\n");
                report.push_str(&format!("    - 模型: {}\n", provider.model));
            }

            report.push_str(&format!("    - Max Tokens: {}\n", provider.max_tokens));
            report.push_str(&format!("    - Temperature: {}\n", provider.temperature));
            report.push_str(&format!("    - 速率限制: {}\n", provider.rate_limit));
            report.push_str(&format!("    - Base URL: {}\n", provider.base_url));
            report.push_str(&format!(
                "    - API Keys: {} (共 {} 个)\n",
                if provider.api_keys_configured {
                    "已配置"
                } else {
                    "未配置"
                },
                provider.api_key_count
            ));
            report.push_str(&format!(
                "    - Proxy: {}\n",
                if provider.proxy_configured {
                    "已配置"
                } else {
                    "未配置"
                }
            ));
            report.push_str(&format!("    - 超时: {} 秒\n", provider.timeout));
            report.push_str(&format!(
                "    - 速率限制: {} 请求/秒\n",
                provider.rate_limit
            ));
        }
        report.push('\n');

        report.push_str("[Tencent Cloud 配置]\n");
        report.push_str(&format!(
            "  启用状态: {}\n",
            if details.tencent.enabled {
                "已启用"
            } else {
                "未启用"
            }
        ));
        report.push_str(&format!(
            "  可用状态: {}\n",
            if details.tencent.available {
                "可用"
            } else {
                "不可用"
            }
        ));
        report.push_str(&format!(
            "  Secret ID: {}\n",
            if details.tencent.secret_id_configured {
                "已配置"
            } else {
                "未配置"
            }
        ));
        report.push_str(&format!(
            "  Secret Key: {}\n",
            if details.tencent.secret_key_configured {
                "已配置"
            } else {
                "未配置"
            }
        ));
        report.push_str(&format!("  区域: {}\n", details.tencent.region));
        report.push_str(&format!(
            "  Proxy: {}\n",
            if details.tencent.proxy_configured {
                "已配置"
            } else {
                "未配置"
            }
        ));
        report.push_str(&format!("  超时: {} 秒\n", details.tencent.timeout));
        report.push_str(&format!(
            "  速率限制: {} 请求/秒\n",
            details.tencent.rate_limit
        ));
        report.push_str(&format!(
            "  最大重试: {} 次\n\n",
            details.tencent.max_retries
        ));

        report.push_str("[日志配置]\n");
        report.push_str(&format!("  级别: {}\n", details.logging.level));
        report.push_str(&format!("  格式: {}\n", details.logging.format));
        report.push_str(&format!("  输出: {}\n", details.logging.output));
        if let Some(ref file) = details.logging.file {
            report.push_str(&format!("  日志文件: {}\n", file));
        }
        report.push('\n');
    }

    if let Some(ref details) = result.project_config_details {
        report.push_str("----------------------------------------\n");
        report.push_str("项目配置详情\n");
        report.push_str("----------------------------------------\n");
        report.push_str(&format!("源语言: {:?}\n", details.source_langs));
        report.push_str(&format!("目标语言: {}\n", details.target_lang));
        report.push_str(&format!("翻译器: {}\n\n", details.provider));

        report.push_str(&format!(
            "包含模式数量: {}\n",
            details.include_patterns_count
        ));
        report.push_str(&format!(
            "排除模式数量: {}\n\n",
            details.exclude_patterns_count
        ));

        report.push_str("[缓存配置]\n");
        report.push_str(&format!(
            "  启用: {}\n",
            if details.cache_enabled { "是" } else { "否" }
        ));
        report.push_str(&format!("  目录: {}\n\n", details.cache_directory));

        report.push_str("[Writer 配置]\n");
        report.push_str(&format!(
            "  备份: {}\n",
            if details.writer_backup { "是" } else { "否" }
        ));
        report.push_str(&format!(
            "  预览模式: {}\n\n",
            if details.writer_dry_run { "是" } else { "否" }
        ));

        report.push_str("[提取配置]\n");
        report.push_str(&format!(
            "  注释: {}\n",
            if details.extraction_comments {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  文档字符串: {}\n",
            if details.extraction_doc_strings {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  错误消息: {}\n",
            if details.extraction_error_messages {
                "是"
            } else {
                "否"
            }
        ));
        report.push_str(&format!(
            "  格式化字符串: {}\n\n",
            if details.extraction_format_strings {
                "是"
            } else {
                "否"
            }
        ));
    }

    if !result.errors.is_empty() {
        report.push_str("----------------------------------------\n");
        report.push_str("错误信息\n");
        report.push_str("----------------------------------------\n");
        for error in &result.errors {
            report.push_str(&format!("- {}\n", error));
        }
        report.push('\n');
    }

    report.push_str("========================================\n");
    report.push_str("验证完成\n");
    report.push_str("========================================\n");

    fs::write(&report_path, report).expect("Failed to write validation report");
    println!("\n验证报告已写入: {}", report_path.display());
}

fn write_detailed_config(output_dir: &Path, result: &ConfigValidationResult) {
    // 写入 JSON 格式的详细配置（不包含敏感信息）
    let json_path = output_dir.join("config_summary.json");

    let mut json_content = String::new();
    json_content.push_str("{\n");
    json_content.push_str("  \"validation_status\": {\n");
    json_content.push_str(&format!(
        "    \"env_file_loaded\": {},\n",
        result.env_file_loaded
    ));
    json_content.push_str(&format!(
        "    \"global_config_loaded\": {},\n",
        result.global_config_loaded
    ));
    json_content.push_str(&format!(
        "    \"project_config_loaded\": {}\n",
        result.project_config_loaded
    ));
    json_content.push_str("  },\n");

    // Provider 可用性
    json_content.push_str("  \"provider_availability\": [\n");
    for (i, provider) in result.provider_availabilities.iter().enumerate() {
        json_content.push_str("    {\n");
        json_content.push_str(&format!(
            "      \"provider_type\": \"{}\",\n",
            provider.provider_type
        ));
        json_content.push_str(&format!("      \"enabled\": {},\n", provider.enabled));
        json_content.push_str(&format!("      \"available\": {}", provider.available));
        if let Some(ref reason) = provider.reason {
            json_content.push_str(&format!(",\n      \"reason\": \"{}\"\n", reason));
        } else {
            json_content.push('\n');
        }
        json_content.push_str("    }");
        if i < result.provider_availabilities.len() - 1 {
            json_content.push(',');
        }
        json_content.push('\n');
    }
    json_content.push_str("  ],\n");

    if let Some(ref details) = result.global_config_details {
        json_content.push_str("  \"global_config\": {\n");
        json_content.push_str(&format!("    \"provider\": \"{}\",\n", details.provider));
        json_content.push_str(&format!(
            "    \"enabled_providers\": {:?},\n",
            details.enabled_providers
        ));

        // DeepLX
        json_content.push_str("    \"deeplx\": {\n");
        json_content.push_str(&format!("      \"enabled\": {},\n", details.deeplx.enabled));
        json_content.push_str(&format!(
            "      \"available\": {},\n",
            details.deeplx.available
        ));
        json_content.push_str(&format!(
            "      \"api_url\": \"{}\",\n",
            details.deeplx.api_url
        ));
        json_content.push_str(&format!(
            "      \"api_key_configured\": {},\n",
            details.deeplx.api_key_configured
        ));
        json_content.push_str(&format!(
            "      \"proxy_configured\": {},\n",
            details.deeplx.proxy_configured
        ));
        json_content.push_str(&format!(
            "      \"rate_limit\": {},\n",
            details.deeplx.rate_limit
        ));
        json_content.push_str(&format!(
            "      \"max_retries\": {}\n",
            details.deeplx.max_retries
        ));
        json_content.push_str("    },\n");

        // LLM
        json_content.push_str("    \"llm\": {\n");
        json_content.push_str(&format!("      \"enabled\": {},\n", details.llm.enabled));
        json_content.push_str(&format!(
            "      \"available\": {},\n",
            details.llm.available
        ));
        json_content.push_str(&format!(
            "      \"health_check_enabled\": {},\n",
            details.llm.health_check_enabled
        ));
        json_content.push_str(&format!(
            "      \"provider_count\": {},\n",
            details.llm.provider_count
        ));
        json_content.push_str(&format!(
            "      \"available_providers_count\": {},\n",
            details.llm.available_providers_count
        ));
        json_content.push_str("      \"providers\": [\n");
        for (i, provider) in details.llm.providers.iter().enumerate() {
            json_content.push_str("        {\n");
            json_content.push_str(&format!("          \"id\": \"{}\",\n", provider.id));
            json_content.push_str(&format!("          \"name\": \"{}\",\n", provider.name));
            json_content.push_str(&format!(
                "          \"available\": {},\n",
                provider.available
            ));
            if let Some(ref reason) = provider.unavailable_reason {
                json_content.push_str(&format!(
                    "          \"unavailable_reason\": \"{}\",\n",
                    reason
                ));
            }
            json_content.push_str(&format!(
                "          \"is_multi_model\": {},\n",
                provider.is_multi_model
            ));
            json_content.push_str(&format!("          \"model\": \"{}\",\n", provider.model));
            json_content.push_str("          \"model_list\": [");
            for (j, model) in provider.model_list.iter().enumerate() {
                json_content.push_str(&format!("\"{}\"", model));
                if j < provider.model_list.len() - 1 {
                    json_content.push_str(", ");
                }
            }
            json_content.push_str("],\n");
            json_content.push_str(&format!(
                "          \"max_tokens\": {},\n",
                provider.max_tokens
            ));
            json_content.push_str(&format!(
                "          \"temperature\": {},\n",
                provider.temperature
            ));
            json_content.push_str(&format!(
                "          \"rate_limit\": {},\n",
                provider.rate_limit
            ));
            json_content.push_str(&format!(
                "          \"base_url\": \"{}\",\n",
                provider.base_url
            ));
            json_content.push_str(&format!(
                "          \"api_keys_configured\": {},\n",
                provider.api_keys_configured
            ));
            json_content.push_str(&format!(
                "          \"proxy_configured\": {},\n",
                provider.proxy_configured
            ));
            json_content.push_str(&format!("          \"timeout\": {},\n", provider.timeout));
            json_content.push_str(&format!(
                "          \"rate_limit\": {}\n",
                provider.rate_limit
            ));
            json_content.push_str("        }");
            if i < details.llm.providers.len() - 1 {
                json_content.push(',');
            }
            json_content.push('\n');
        }
        json_content.push_str("      ]\n");
        json_content.push_str("    },\n");

        // Tencent
        json_content.push_str("    \"tencent\": {\n");
        json_content.push_str(&format!(
            "      \"enabled\": {},\n",
            details.tencent.enabled
        ));
        json_content.push_str(&format!(
            "      \"available\": {},\n",
            details.tencent.available
        ));
        json_content.push_str(&format!(
            "      \"secret_id_configured\": {},\n",
            details.tencent.secret_id_configured
        ));
        json_content.push_str(&format!(
            "      \"secret_key_configured\": {},\n",
            details.tencent.secret_key_configured
        ));
        json_content.push_str(&format!(
            "      \"region\": \"{}\",\n",
            details.tencent.region
        ));
        json_content.push_str(&format!(
            "      \"proxy_configured\": {},\n",
            details.tencent.proxy_configured
        ));
        json_content.push_str(&format!(
            "      \"timeout\": {},\n",
            details.tencent.timeout
        ));
        json_content.push_str(&format!(
            "      \"rate_limit\": {},\n",
            details.tencent.rate_limit
        ));
        json_content.push_str(&format!(
            "      \"max_retries\": {}\n",
            details.tencent.max_retries
        ));
        json_content.push_str("    },\n");

        // Logging
        json_content.push_str("    \"logging\": {\n");
        json_content.push_str(&format!(
            "      \"level\": \"{}\",\n",
            details.logging.level
        ));
        json_content.push_str(&format!(
            "      \"format\": \"{}\",\n",
            details.logging.format
        ));
        json_content.push_str(&format!(
            "      \"output\": \"{}\"\n",
            details.logging.output
        ));
        json_content.push_str("    }\n");
        json_content.push_str("  },\n");
    }

    if let Some(ref details) = result.project_config_details {
        json_content.push_str("  \"project_config\": {\n");
        json_content.push_str(&format!(
            "    \"source_langs\": {:?},\n",
            details.source_langs
        ));
        json_content.push_str(&format!(
            "    \"target_lang\": \"{}\",\n",
            details.target_lang
        ));
        json_content.push_str(&format!("    \"provider\": \"{}\",\n", details.provider));
        json_content.push_str(&format!(
            "    \"include_patterns_count\": {},\n",
            details.include_patterns_count
        ));
        json_content.push_str(&format!(
            "    \"exclude_patterns_count\": {},\n",
            details.exclude_patterns_count
        ));
        json_content.push_str(&format!(
            "    \"cache_enabled\": {},\n",
            details.cache_enabled
        ));
        json_content.push_str(&format!(
            "    \"cache_directory\": \"{}\",\n",
            details.cache_directory
        ));
        json_content.push_str(&format!(
            "    \"writer_backup\": {},\n",
            details.writer_backup
        ));
        json_content.push_str(&format!(
            "    \"writer_dry_run\": {},\n",
            details.writer_dry_run
        ));
        json_content.push_str(&format!(
            "    \"extraction_comments\": {},\n",
            details.extraction_comments
        ));
        json_content.push_str(&format!(
            "    \"extraction_doc_strings\": {},\n",
            details.extraction_doc_strings
        ));
        json_content.push_str(&format!(
            "    \"extraction_error_messages\": {},\n",
            details.extraction_error_messages
        ));
        json_content.push_str(&format!(
            "    \"extraction_format_strings\": {}\n",
            details.extraction_format_strings
        ));
        json_content.push_str("  }\n");
    }

    json_content.push_str("}\n");

    fs::write(&json_path, json_content).expect("Failed to write JSON config summary");
    println!("JSON 配置摘要已写入: {}", json_path.display());
}

#[test]
fn test_config_validation() {
    main();
}
