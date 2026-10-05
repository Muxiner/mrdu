//! 单个输出条目的展示信息。
//!
//! 记录条目占父级的比例、所处深度、缩进前缀与是否为末位子项，
//! 并据此决定树形连接符与显示颜色。

use crate::output::tree_shape;
use termcolor::Color;

/// 渲染一个 [`super::AnalysisItem`] 节点所需的展示状态。
#[derive(Debug, Clone)]
pub struct DisplayItemInfo {
    /// 该条目占父级的百分比（根节点为 100）。
    pub(crate) occupied_size: f64,
    /// 当前深度，根节点为 0。
    pub(crate) dir_level: usize,
    /// 是否为同级中的最后一个条目，影响连接符形状。
    is_last: bool,
    /// 累积的缩进前缀。
    pub(crate) prefix: String,
}

impl DisplayItemInfo {
    /// 创建根节点对应的展示信息。
    pub fn new() -> Self {
        Self {
            occupied_size: 100.0,
            dir_level: 0,
            is_last: true,
            prefix: String::new(),
        }
    }

    /// 基于当前节点派生出子节点的展示信息。
    pub fn add_item(&self, occupied_size: f64, is_last: bool) -> Self {
        Self {
            occupied_size,
            dir_level: self.dir_level + 1,
            is_last,
            prefix: self.prefix.clone() + self.display_prefix(false) + &String::from("  "),
        }
    }

    /// 返回树形连接符：`is_fork` 为 `true` 时返回分支符号，否则返回缩进。
    pub fn display_prefix(&self, is_fork: bool) -> &'static str {
        match self.is_last {
            true => match is_fork {
                true => tree_shape::LAST_LEAF, // "└──"
                false => "  ",
            },
            false => match is_fork {
                true => tree_shape::LEAF,    // "├──"
                false => tree_shape::BRANCH, // "│"
            },
        }
    }

    /// 返回显示颜色：根节点为白色，其余按占比分为红 / 黄 / 绿。
    ///
    /// `is_disk_size` 为 `true` 时返回大小文字所用的加深色。
    pub fn display_color(&self, is_disk_size: bool) -> Option<Color> {
        let darken = |x: u8| (x as f32 * 0.5).round() as u8;
        let get_color = |r: u8, g: u8, b: u8| {
            if is_disk_size {
                Color::Rgb(darken(r), darken(g), b)
            } else {
                Color::Rgb(r, g, b)
            }
        };
        match self.dir_level {
            // 根目录，白色
            0 => Some(get_color(250, 250, 250)),
            // 占比 >= 50%，红色
            _ if self.occupied_size >= 50.0 => Some(get_color(255, 100, 100)),
            // 占比 [10, 50)，黄色
            _ if self.occupied_size >= 10.0 && self.occupied_size < 50.0 => {
                Some(get_color(255, 222, 72))
            }
            // 占比 < 10%，绿色
            _ => Some(get_color(100, 255, 90)),
        }
    }
}

impl Default for DisplayItemInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_reflects_position() {
        let last = DisplayItemInfo::new().add_item(50.0, true);
        assert_eq!(last.display_prefix(true), tree_shape::LAST_LEAF);

        let middle = DisplayItemInfo::new().add_item(50.0, false);
        assert_eq!(middle.display_prefix(true), tree_shape::LEAF);
        assert_eq!(middle.display_prefix(false), tree_shape::BRANCH);
    }

    #[test]
    fn color_follows_occupancy_thresholds() {
        assert_eq!(
            DisplayItemInfo::new().display_color(false),
            Some(Color::Rgb(250, 250, 250))
        );
        assert_eq!(
            DisplayItemInfo::new()
                .add_item(60.0, false)
                .display_color(false),
            Some(Color::Rgb(255, 100, 100))
        );
        assert_eq!(
            DisplayItemInfo::new()
                .add_item(20.0, false)
                .display_color(false),
            Some(Color::Rgb(255, 222, 72))
        );
        assert_eq!(
            DisplayItemInfo::new()
                .add_item(5.0, false)
                .display_color(false),
            Some(Color::Rgb(100, 255, 90))
        );
    }
}
