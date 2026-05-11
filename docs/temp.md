PS D:\项目\agent\wf-agent\packages\common-utils\src\template> translator detect .
2026-05-11T03:08:26.018774Z INFO Starting language script detection, path: ., language: None, verbose: false
at src\commands\detect.rs:55

2026-05-11T03:08:26.019476Z INFO Checking path type, path: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template, is_file: false, is_dir: true
at src\scanner\walker.rs:352

2026-05-11T03:08:26.020007Z INFO Starting directory scan: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template
at src\scanner\walker.rs:123

2026-05-11T03:08:26.021330Z WARN Low confidence encoding detection, source: <bytes>, encoding: Shift_JIS, confidence: 0.46695367450885283, threshold: 0.7
at src\encoding\detector.rs:75

Error: Parse error: Low confidence encoding detection: encoding=Shift_JIS confidence=0.47 threshold=0.70
PS D:\项目\agent\wf-agent\packages\common-utils\src\template> translator detect .
2026-05-11T03:08:38.351563Z INFO Starting language script detection, path: ., language: None, verbose: false
at src\commands\detect.rs:55

2026-05-11T03:08:38.352135Z INFO Checking path type, path: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template, is_file: false, is_dir: true
at src\scanner\walker.rs:352

2026-05-11T03:08:38.352533Z INFO Starting directory scan: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template
at src\scanner\walker.rs:123

================================================================================
Language Detection Report
================================================================================

Summary:
Detection Time: 2026-05-11 03:08:38
Target Path: .
Total Files: 3
Total Lines: 1026
Matching Lines: 33
Matching Segments: 22
Target Script: CJK

---

NOTE: Detection is based on Unicode script/language family.
Specific language identification is not guaranteed.

---

## Detection Results:

File: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template\template-renderer.test.ts
Segment 1 (Lines 453-453):
expect(() => renderTemplate("{{@index}}", {})).toThrow(/只能在.\*循环内部使用/);

Segment 2 (Lines 458-458):
expect(() => renderTemplate("{{@first}}", {})).toThrow(/只能在.\*循环内部使用/);

Segment 3 (Lines 463-463):
expect(() => renderTemplate("{{@last}}", {})).toThrow(/只能在.\*循环内部使用/);

Segment 4 (Lines 472-472):
).toThrow(/不支持的循环特殊变量/);

Segment 5 (Lines 485-485):
).toThrow(/不支持的循环特殊变量/);

Segment 6 (Lines 492-492):
expect(() => renderTemplate("{{this}}", {})).toThrow(/只能在.\*循环内部使用/);

Segment 7 (Lines 497-497):
expect(() => renderTemplate("{{this.name}}", {})).toThrow(/只能在.\*循环内部使用/)...

Segment 8 (Lines 505-505):
/只能在.\*循环内部使用/,

## Total: 8 matching line(s)

File: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template\template-renderer.ts
Segment 1 (Lines 2-3):
_ TemplateRenderer - 模板渲染器
_ 提供模板变量替换功能，支持嵌套路径解析

Segment 2 (Lines 5-11):
_ 功能：
_ - 支持 {{variable}} 占位符替换 \* - 支持嵌套路径解析（如 user.name）
... (4 more lines)

Segment 3 (Lines 158-158):
`不支持的循环特殊变量 '${trimmedName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECIA...

Segment 4 (Lines 244-244):
`不支持的循环特殊变量 '${varName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECIA...

Segment 5 (Lines 303-303):
`不支持的循环特殊变量 '${trimmedName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECIA...

Segment 6 (Lines 360-360):
`不支持的循环特殊变量 '${variableName}'。支持的变量: ${Array.from(SUPPORTED_LOOP_SPECI...

Segment 7 (Lines 378-379):
_ 渲染模板
_ 替换模板中的 {{variable}} 占位符，支持条件和循环

Segment 8 (Lines 381-384):
_ @param template 模板字符串，包含 {{variable}} 占位符
_ @param variables 变量对象 \* @returns 渲染后的字符串
... (1 more lines)

Segment 9 (Lines 390-390): \* // 结果: 'Hello, Alice! Today is 2024-01-01.'

Segment 10 (Lines 397-397): \* // 结果: 'User: Bob, Age: 30'

Segment 11 (Lines 400-400): \* @example 条件渲染

Segment 12 (Lines 404-404): \* // 结果: 'Name: Alice'

Segment 13 (Lines 407-407): \* @example 循环渲染

Segment 14 (Lines 411-411): \* // 结果: 'Items: - A - B - C'

## Total: 25 matching line(s)

================================================================================

2026-05-11T03:08:38.353606Z INFO Language script detection completed, files_scanned: 3, lines_scanned: 1026, matching_lines: 33
at src\commands\detect.rs:102

PS D:\项目\agent\wf-agent\packages\common-utils\src\template> translator verify
2026-05-11T03:08:51.799160Z INFO Starting verification of extraction rules, path: .
at src\commands\verify\args.rs:55

2026-05-11T03:08:51.799814Z INFO Checking path type, path: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template, is_file: false, is_dir: true
at src\scanner\walker.rs:352

2026-05-11T03:08:51.800281Z INFO Starting directory scan: \\?\D:\项目\agent\wf-agent\packages\common-utils\src\template
at src\scanner\walker.rs:123

2026-05-11T03:08:51.800539Z INFO Scanned files, files_found: 3
at src\commands\verify\args.rs:86

2026-05-11T03:08:51.803913Z INFO Custom pattern matcher created successfully, name: todo_pattern
at src\parser\regex\custom_pattern_matcher.rs:58

2026-05-11T03:08:51.804124Z INFO Custom pattern matcher created successfully, name: custom_error_message
at src\parser\regex\custom_pattern_matcher.rs:58

2026-05-11T03:08:51.804370Z INFO Custom pattern matcher created successfully, name: custom_log_message
at src\parser\regex\custom_pattern_matcher.rs:58

2026-05-11T03:08:51.804586Z INFO Custom pattern matcher created successfully, name: config_message
at src\parser\regex\custom_pattern_matcher.rs:58

2026-05-11T03:08:51.804922Z INFO Custom pattern matcher created successfully, name: note_pattern
at src\parser\regex\custom_pattern_matcher.rs:58

2026-05-11T03:08:51.806558Z INFO Extracted matches, total_matches: 0
at src\commands\verify\args.rs:126

2026-05-11T03:08:51.806704Z INFO Filtered matches, filtered_matches: 0
at src\commands\verify\args.rs:137

┌─────────┬──────┬──────────┬──────┬──────┬────────────────┬───────────┐
│ Pattern ┆ Type ┆ Category ┆ File ┆ Line ┆ Extracted Text ┆ Raw Match │
╞═════════╪══════╪══════════╪══════╪══════╪════════════════╪═══════════╡
└─────────┴──────┴──────────┴──────┴──────┴────────────────┴───────────┘

=== Summary ===
Total files: 3
Total matches: 0

2026-05-11T03:08:51.807022Z INFO Verification completed successfully
at src\commands\verify\args.rs:171
