//! mrdu —— 一个简单的命令行磁盘占用分析工具。
//!
//! 库层按职责划分为：
//! - [`args`]：命令行参数解析；
//! - [`analysis`]：目录递归分析与结果树构建；
//! - [`file_info`]：跨平台的文件/目录元信息读取；
//! - [`output`]：结果树的终端渲染。

pub mod analysis;
pub mod args;
pub mod file_info;
pub mod output;
