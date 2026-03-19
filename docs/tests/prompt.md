为main模块编写集成测试文件，参考tests\parser_integration目录的结构，使用fixtures目录存放待测试的目标文件，使用output存放输出结果，包括缓存文件、备份文件、翻译后文件、日志。修改 `e:\project\codebase-translator\tests\main_e2e.rs` 作为主导出模块。
要求实际调用 `e:\project\codebase-translator\Cargo.toml` `e:\project\codebase-translator\.env` `e:\project\codebase-translator\translator.toml` 的配置，完成端到端测试。测试用例的设计可以简单，重点在于查看日志、缓存文件、备份文件、翻译结果是否符合预期，因此重点在于提供所有输出文件，以供我检查。
在fixtures目录编写项目级配置文件，参考 `e:\project\codebase-translator\.translator` 。也可以直接复制

由于部分文件被gitignore屏蔽(例如配置文件可能包含api key等敏感信息)，不确定文件是否存在时使用test-path命令。可以使用type命令查看被屏蔽的文件的内容