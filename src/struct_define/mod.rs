pub mod config;
pub mod display_info;
pub mod analysis_item;
pub mod file_info;

// 模块，终端输出树形结构视觉效果
pub mod tree_shape {
    pub const SPACING: &str = "──";
    pub const BRANCH: &str = "│";
    pub const LEAF: &str = "├──";
    pub const _LEAF_WITH_BRANCH: &str = "├─┬";
    pub const LAST_LEAF: &str = "└──";
    pub const _LAST_LEAF_WITH_BRANCH: &str = "└─┬";
}

// 模块，终端输出内容的颜色
pub mod display_color {
    use termcolor::Color;
    pub const COLOR_GRAY: Option<Color> = Some(Color::Rgb(147, 147, 147));
}
