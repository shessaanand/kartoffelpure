//! Tab strip layout mode and sizing configuration.

/// How tab buttons are presented in the browser window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabLayoutMode {
    /// Tabs in a top row; overflow scrolls horizontally.
    #[default]
    Horizontal,
    /// Tabs in a left sidebar; overflow scrolls vertically.
    Vertical,
}

/// Presentation settings for the tab strip (not tab data).
#[derive(Debug, Clone, Copy)]
pub struct TabStripConfig {
    pub layout_mode: TabLayoutMode,
    pub min_tab_width: i32,
    pub max_tab_width: i32,
    pub vertical_sidebar_width: i32,
}

impl Default for TabStripConfig {
    fn default() -> Self {
        Self {
            layout_mode: TabLayoutMode::Horizontal,
            min_tab_width: 80,
            max_tab_width: 220,
            vertical_sidebar_width: 220,
        }
    }
}

impl TabStripConfig {
    /// Builds config for the given layout mode with default sizes.
    pub fn with_mode(mode: TabLayoutMode) -> Self {
        Self {
            layout_mode: mode,
            ..Self::default()
        }
    }
}
