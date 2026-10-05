//! CLI 端到端测试。
//!
//! 通过 `assert_cmd` 调用真实编译出的 `mrdu` 二进制，覆盖默认分析、
//! 深度 / 占比 / 精度 / 实际分配大小等选项，以及文件、不存在路径等错误分支。

use assert_cmd::Command;
use std::ffi::OsStr;
use std::process::Output;

/// 用于测试的固定目录。
const FIXTURE: &str = "tests/test_file";

/// 运行 `mrdu` 并返回其执行结果。
fn run<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::cargo_bin("mrdu")
        .expect("未找到 mrdu 二进制")
        .args(args)
        .output()
        .expect("执行 mrdu 失败")
}

/// 运行并断言成功、stderr 为空，返回标准输出字符串。
fn run_ok<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = run(args);
    assert!(
        output.status.success(),
        "预期成功，实际退出码 {:?}，stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "",
        "成功的运行不应向 stderr 输出"
    );
    String::from_utf8(output.stdout).expect("标准输出不是合法 UTF-8")
}

/// 运行并断言失败，返回 stderr 字符串。
fn run_err<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = run(args);
    assert!(!output.status.success(), "预期失败，但运行成功了");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn default_analysis_lists_entries_within_depth() {
    let out = run_ok([FIXTURE]);

    assert!(out.contains("Analyzing: tests/test_file"));
    // 根节点与第一层子项
    for name in [
        "test_file",
        "test_dir_",
        "test_dir_d2",
        "test_file_d1",
        "test_dir_hidden_file",
    ] {
        assert!(out.contains(name), "输出缺少 {name}:\n{out}");
    }
    // 第二层子项（默认深度 2）
    for name in ["test_file😄.unicode", "test_dir_d3", ".test_file"] {
        assert!(out.contains(name), "输出缺少 {name}:\n{out}");
    }
    // 超过默认深度的条目不应出现
    for name in ["test_dir_d4", "test_file_d4"] {
        assert!(!out.contains(name), "不应出现超过深度的 {name}:\n{out}");
    }
    // 树形连接符
    for shape in ["└──", "├──", "│", " ── "] {
        assert!(out.contains(shape), "输出缺少树形字符 {shape}:\n{out}");
    }
}

#[test]
fn max_depth_limits_recursion() {
    let out = run_ok(["-d", "1", FIXTURE]);

    for name in [
        "test_dir_",
        "test_dir_d2",
        "test_file_d1",
        "test_dir_hidden_file",
    ] {
        assert!(out.contains(name), "输出缺少第一层 {name}:\n{out}");
    }
    for name in ["test_file😄.unicode", "test_dir_d3", ".test_file"] {
        assert!(!out.contains(name), "深度 1 不应包含第二层 {name}:\n{out}");
    }
}

#[test]
fn min_percent_filters_small_entries() {
    let out = run_ok(["-p", "30", FIXTURE]);

    // test_dir_ 占比 32.20% > 30%，保留
    assert!(out.contains("test_dir_"));
    // 下列条目占比低于 30%，应被过滤
    for name in ["test_dir_d2", "test_file_d1", "test_dir_hidden_file"] {
        assert!(!out.contains(name), "低于阈值的 {name} 不应出现:\n{out}");
    }
}

#[test]
fn precision_controls_decimal_places() {
    let one = run_ok(["-n", "1", FIXTURE]);
    assert!(one.contains("100.0%"), "缺少一位小数根节点:\n{one}");
    assert!(one.contains("32.2%"), "缺少一位小数条目:\n{one}");
    assert!(!one.contains("32.20%"), "不应保留两位小数:\n{one}");

    let zero = run_ok(["-n", "0", FIXTURE]);
    assert!(zero.contains("32%"), "缺少零位小数条目:\n{zero}");
    assert!(!zero.contains("32.2%"), "零位小数不应出现小数点:\n{zero}");
}

#[test]
fn apparent_flag_runs_successfully() {
    let out = run_ok(["-a", FIXTURE]);
    assert!(out.contains("Analyzing: tests/test_file"));
    assert!(out.contains("test_file"));
}

#[test]
fn defaults_to_current_directory() {
    let output = Command::cargo_bin("mrdu")
        .expect("未找到 mrdu 二进制")
        .current_dir(FIXTURE)
        .output()
        .expect("执行 mrdu 失败");

    assert!(output.status.success());
    let out = String::from_utf8(output.stdout).expect("标准输出不是合法 UTF-8");
    assert!(out.contains("Analyzing:"), "缺少 Analyzing 头:\n{out}");
    assert!(out.contains("test_dir_"), "未分析当前目录:\n{out}");
}

#[test]
fn file_target_reports_error() {
    let err = run_err(["Cargo.toml"]);
    assert!(err.contains("is not a directory"), "stderr: {err}");
}

#[test]
fn nonexistent_target_reports_error() {
    let output = run(["tests/__does_not_exist__"]);
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("No such file") || err.contains("NotFound"),
        "stderr: {err}"
    );
}
