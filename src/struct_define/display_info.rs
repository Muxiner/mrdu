use crate::struct_define::tree_shape;
use termcolor::Color;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DisplayItemInfo {
    pub(crate) occupied_size: f64,
    pub(crate) dir_level: usize,
    is_last: bool,
    pub(crate) prefix: String,
}

#[allow(dead_code)]
impl DisplayItemInfo {
    pub fn new() -> Self {
        Self {
            occupied_size: 100.0,
            dir_level: 0,
            is_last: true,
            prefix: String::new(),
        }
    }

    pub fn add_item(&self, occupied_size: f64, is_last: bool) -> Self {
        Self {
            occupied_size,
            dir_level: self.dir_level + 1,
            is_last,
            prefix: self.prefix.clone()
                + self.display_prefix(false)
                + &String::from("  "),
        }
    }

    pub fn display_prefix(&self, is_fork: bool) -> &'static str {
        match self.is_last {
            true => match is_fork {
                true => tree_shape::LAST_LEAF,  // "└──"
                false => "  ",
            },
            false => match is_fork {
                true => tree_shape::LEAF,       // "├──"
                false => tree_shape::BRANCH,    // "│"
            },
        }
    }

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
            // Analyzed root directory, Purple
            0 => Some(get_color(250, 250, 250)),
            // Directories or files that occupied >= 50%, Red
            _ if self.occupied_size >= 50.0 => Some(get_color(255, 100, 100)),
            // Directories or files that occupied < 50.0% && >= 10.0%, Yellow
            _ if self.occupied_size >= 10.0 && self.occupied_size < 50.0 => {
                Some(get_color(255, 222, 72))
            }
            // Directories or files that occupied < 10.0%, Green
            _ => Some(get_color(100, 255, 90)),
        }
    }
}

impl Default for DisplayItemInfo {
    fn default() -> Self {
        Self::new()
    }
}
