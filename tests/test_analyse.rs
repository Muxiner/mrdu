use assert_cmd::Command;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::str;
use walkdir::WalkDir;

// 构建一个命令行指令，并执行该指令，并返回标准输出的字符串
pub fn build_command<T: AsRef<OsStr>>(command_args: Vec<T>) -> String {
    let mut cmd = &mut Command::cargo_bin("mrdu").unwrap();
    for p in command_args {
        cmd = cmd.arg(p);
    }
    let finished = &cmd.unwrap();
    let stderr = str::from_utf8(&finished.stderr).unwrap();
    assert_eq!(stderr, "");

    str::from_utf8(&finished.stdout).unwrap().into()
}

// 递归获取给定路径和深度的所有的文件名和文件夹名
pub fn get_all_filename_dirname(dir_path: &str, depth: u8) -> Result<Vec<String>, Box<dyn Error>> {
    let mut vec_string: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        match entry.path().is_dir() {
            true => {
                vec_string.push(entry.file_name().to_str().unwrap().to_string());
                if depth > 1 {
                    vec_string.extend(
                        get_all_filename_dirname(entry.path().to_str().unwrap(), depth - 1)
                            .unwrap_or_else(|_| Vec::new()),
                    );
                }
            }
            false => vec_string.push(entry.file_name().to_str().unwrap().to_string()),
        }
    }
    Ok(vec_string)
}

// 获取 path 这一路径中能够递归的深度
pub fn get_max_depth(path: &str) -> Option<usize> {
    let mut max_depth = None;
    for entry in WalkDir::new(path).max_depth(10) {
        match entry {
            Ok(entry) => {
                let depth = entry.depth();
                max_depth = max_depth.map(|max: usize| max.max(depth)).or(Some(depth));
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                continue;
            }
        }
    }
    max_depth
}

#[cfg(test)]
mod test_analyse {
    use crate::build_command;
    // use crate::get_all_filename_dirname;
    use crate::get_max_depth;
    use std::env::current_dir;
    use std::error::Error;

    #[test]
    // 测试没有任何参数的分析结果
    // 主要测试结果是否含有应该有的 tree——shape 字符和含有的文件名和文件夹名
    /// # 结果
    /// ```txt
    ///    Analyzing: tests/test_file
    ///    Elapsed time: 1.3224ms
    ///    └── 100.00% [22.53 KB] ── test_file
    ///        ├── 24.49% [5.52 KB] ── test_dir_
    ///        │  └── 97.54% [5.38 KB] ── test_file😄.unicode
    ///        ├── 24.49% [5.52 KB] ── test_dir_d2
    ///        │  └── 97.54% [5.38 KB] ── test_file_d2
    ///        ├── 23.89% [5.38 KB] ── test_dir_hidden_file
    ///        │  └── 100.00% [5.38 KB] ── .test_file
    ///        └── 23.89% [5.38 KB] ── test_file_d1
    /// ```
    fn test_no_args_analyse() -> Result<(), Box<dyn Error>> {
        let output = build_command(vec!["tests/test_file"]);
        // let target_dir = current_dir()?
            // .join("tests/test_file")
            // .to_str()
            // .unwrap()
            // .replace(r#"\"#, "/");
        // let mut file_names =
            // get_all_filename_dirname(&target_dir, 2).unwrap_or_else(|_| Vec::new());
        // file_names.push("test_file".to_string());
        // println!("{}", output);
        // for file_name in file_names {
            // println!("{}", file_name);
            // assert!(output.contains(&file_name));
        // }
        assert!(output.contains("Analyzing: tests/test_file"));
        assert!(output.contains("└──"));
        assert!(output.contains("    ├──"));
        assert!(output.contains("    │  └──"));
        assert!(output.contains("    └──"));
        assert!(output.contains(" ── "));
        Ok(())
    }

    #[test]
    /// ### 结果
    /// ```txt
    ///     Analyzing: tests/test_file
    ///     Elapsed time: 1.451ms
    ///     └── 100.00% [22.53 KB] ── test_file
    ///         ├── 24.49% [5.52 KB] ── test_dir_
    ///         ├── 24.49% [5.52 KB] ── test_dir_d2
    ///         ├── 23.89% [5.38 KB] ── test_dir_hidden_file
    ///         └── 23.89% [5.38 KB] ── test_file_d1
    /// ```
    fn test_depth_analyse() -> Result<(), Box<dyn Error>> {
        let output = build_command(vec!["-d", "1", "tests/test_file"]);
        // println!("{}", output);
        // let target_dir = current_dir()?
        //     .join("tests/test_file")
        //     .to_str()
        //     .unwrap()
        //     .replace(r#"\"#, "/");
        // let mut file_names =
        //     get_all_filename_dirname(&target_dir, 1).unwrap_or_else(|_| Vec::new());
        // file_names.push("test_file".to_string());
        // println!("{}", output);
        // for file_name in file_names {
            // println!("{}", file_name);
            // assert!(output.contains(&file_name));
        // }
        assert!(output.contains("Analyzing: tests/test_file"));
        assert!(output.contains("└──"));
        assert!(output.contains("    ├──"));
        assert!(output.contains("    └──"));
        assert!(output.contains(" ── "));
        Ok(())
    }

    #[test]
    /// # 结果
    /// ```txt
    ///    Analyzing: tests/test_file
    ///    Elapsed time: 1.3224ms
    ///    └── 100.00% [22.53 KB] ── test_file
    ///        ├── 24.49% [5.52 KB] ── test_dir_
    ///        │  └── 97.54% [5.38 KB] ── test_file😄.unicode
    ///        ├── 24.49% [5.52 KB] ── test_dir_d2
    ///        │  └── 97.54% [5.38 KB] ── test_file_d2
    ///        ├── 23.89% [5.38 KB] ── test_dir_hidden_file
    ///        │  └── 100.00% [5.38 KB] ── .test_file
    ///        └── 23.89% [5.38 KB] ── test_file_d1
    /// ```
    fn test_max_depth_analyse() -> Result<(), Box<dyn Error>> {
        let target_dir = current_dir()?
            .join("tests/test_file")
            .to_str()
            .unwrap()
            .replace(r#"\"#, "/");
        let depth = get_max_depth(&target_dir).unwrap();
        let output = build_command(vec!["-d", depth.to_string().as_str(), "tests/test_file"]);
        // let mut file_names =
        //     get_all_filename_dirname(&target_dir, depth as u8).unwrap_or_else(|_| Vec::new());
        // file_names.push("test_file".to_string());
        // // println!("{}", output);
        // for file_name in file_names {
        //     // println!("{}", file_name);
        //     assert!(output.contains(&file_name));
        // }
        assert!(output.contains("Analyzing: tests/test_file"));
        assert!(output.contains("└──"));
        assert!(output.contains("    ├──"));
        assert!(output.contains("    │  └──"));
        assert!(output.contains("    └──"));
        assert!(output.contains(" ── "));
        Ok(())
    }
}
