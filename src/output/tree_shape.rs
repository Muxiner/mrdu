//! 终端输出的树形结构字符。

/// 连接符与名称之间的间隔。
pub const SPACING: &str = "──";
/// 非末位同级条目下方的竖线。
pub const BRANCH: &str = "│";
/// 非末位子条目的分支连接符。
pub const LEAF: &str = "├──";
/// 末位子条目的分支连接符。
pub const LAST_LEAF: &str = "└──";
