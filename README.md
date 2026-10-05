# mrdu

> A simple command line disk analysis tool. — 一个简单的命令行磁盘占用分析工具。

`mrdu` 递归扫描指定目录，统计每个子项占用的磁盘空间，并以树形结构、彩色百分比的形式直观展示（类似 `du` / `dust`）。

```text
Analyzing: tests/test_file
└── 100.00% [25.62 KB] ── test_file
    ├── 32.20% [8.25 KB] ── test_dir_
    │  ├── 62.29% [5.14 KB] ── test_file😄.unicode
    │  ├── 19.77% [1.63 KB] ── test_file.c
    │  └── 17.94% [1.48 KB] ── long_dir_name_what_a_very_long_dir_name_....txt
    ├── 24.97% [6.4 KB] ── test_dir_d2
    │  ├── 80.32% [5.14 KB] ── test_file_d2
    │  └── 19.68% [1.26 KB] ── test_dir_d3
    ├── 20.06% [5.14 KB] ── test_file_d1
    └── 20.06% [5.14 KB] ── test_dir_hidden_file
        └── 100.00% [5.14 KB] ── .test_file

Elapsed time: 586.625µs
```

## 特性

- **树形展示**：以 `├──` / `└──` 等字符绘制目录树，层级一目了然。
- **占用百分比**：每个条目显示其占父目录的比例，并按占比着色（红 / 黄 / 绿）。
- **并行扫描**：基于 [`rayon`](https://crates.io/crates/rayon) 并行遍历子目录，充分利用多核。
- **深度与阈值控制**：可限制递归深度、过滤占比过低的小条目。
- **跨平台**：支持 Unix / Windows，正确处理 Unicode 文件名与隐藏文件。

## 安装

需要 Rust 工具链（stable，edition 2021）。

```bash
# 从源码编译
cargo build --release

# 可执行文件位于
./target/release/mrdu

# 安装到 PATH（可选）
cargo install --path .
```

## 使用

```bash
mrdu [FLAGS] [OPTIONS] [target-dir]
```

`target-dir` 为需要分析的目录，默认为当前路径。

### 选项

| 选项 | 缩写 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `--max-depth <max-depth>` | `-d` | `2` | 最大递归深度。 |
| `--min-percent <min-percent>` | `-p` | `5` | 展示阈值（0–100%），占比低于该值的条目不显示。 |
| `--precision <decimal-num>` | `-n` | `2` | 百分比数值的小数位数。 |
| `--apparent` | `-a` | | 按文件在磁盘上**实际分配**的大小统计（默认统计文件的逻辑长度）。 |
| `--help` | `-h` | | 打印帮助信息。 |
| `--version` | `-V` | | 打印版本信息。 |

### 示例

```bash
# 分析当前目录，默认深度 2
mrdu

# 分析指定目录，最大深度 3
mrdu -d 3 /path/to/dir

# 只显示占比大于 10% 的条目
mrdu -p 10 /path/to/dir

# 按磁盘实际分配大小统计，百分比保留 1 位小数
mrdu -a -n 1 /path/to/dir
```

## 输出说明

每一行格式为：

```text
<树形前缀> <占比>% [<大小>] ── <名称>
```

- **占比**：该条目占用父目录大小的百分比。
- **大小**：人类可读的字节数（B / KB / MB / GB …，以 1000 为进制）。
- **颜色**：根目录为白色；占比 ≥ 50% 为红色，10%–50% 为黄色，< 10% 为绿色。当输出被重定向到非终端时自动关闭着色。

### 大小统计口径

- 默认：文件的逻辑大小（Unix `st_size`）。
- `-a` / `--apparent`：文件在磁盘上的实际分配大小（Unix `st_blocks * 512`，Windows 压缩文件大小）。

## 开发与测试

```bash
cargo check          # 编译检查
cargo test           # 运行测试
cargo build --release
```

测试位于 `tests/`，使用 [`assert_cmd`](https://crates.io/crates/assert_cmd) 以真实二进制配合 `tests/test_file` 目录进行端到端校验。持续集成配置见 `.github/workflows/rust_actions.yml`（每次 push 执行 `cargo check` 与 `cargo test`）。

## 项目结构

```text
src/
├── main.rs                     # 入口：解析参数、选择配色、调用分析并输出
├── lib.rs                      # 库入口，导出各模块
├── args.rs                     # 命令行参数定义 (structopt)
├── analysis.rs                 # 目录递归分析与结果树（rayon 并行）
├── file_info.rs                # 跨平台文件/目录信息读取
└── output/
    ├── mod.rs                  # 结果树渲染、大小格式化
    ├── display_info.rs         # 缩进前缀与颜色
    ├── tree_shape.rs           # 树形连接符常量
    └── color.rs                # 颜色常量
```

## 许可

本项目基于 [MIT](./LICENSE) 许可证开源。作者：[Muxiner](https://github.com/Muxiner)。
