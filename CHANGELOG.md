# Changelog

本项目所有值得记录的变更都会写入此文件。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- 添加 MIT 许可证。
- 添加 `rustfmt.toml` 统一代码格式。
- 添加 CI 的 `fmt`、`clippy` 检查任务。
- 本变更日志文件。

### Changed

- 重构模块结构：以 `args`、`analysis`、`file_info`、`output` 替代原
  `struct_define`、`methods`，并将 `tree_shape`、`color` 拆分为独立文件。
- 清理死代码（注释掉的旧实现、未使用的辅助函数与常量）。
- 为公开模块与条目补充文档注释。
- 补充 `Cargo.toml` 元数据（`license`、`keywords`、`categories`、`rust-version`）。
- 将仅用于测试的 `assert_cmd`、`walkdir` 移至 `[dev-dependencies]`。
- 升级 GitHub Actions 至 `actions/checkout@v4` 与 `dtolnay/rust-toolchain`。

### Fixed

- 修复 `clippy::needless_borrow` 警告。

## [0.1.0] - 2023-04-26

### Added

- 首个版本：递归扫描目录并以树形结构、彩色百分比展示磁盘占用。
- 支持最大深度（`-d`）、展示阈值（`-p`）、小数精度（`-n`）、实际分配大小（`-a`）等选项。
- 基于 `rayon` 的并行目录遍历。
- 跨平台支持（Unix / Windows）。
- 使用 `assert_cmd` 的端到端测试与 GitHub Actions 基础 CI。

[Unreleased]: https://github.com/Muxiner/mrdu/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Muxiner/mrdu/releases/tag/v0.1.0
