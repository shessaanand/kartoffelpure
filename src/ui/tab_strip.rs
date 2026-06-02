//! Tab strip presentation (horizontal or vertical). Tab data lives in `TabManager`.

use crate::browser::TabId;
use crate::ui::tab_layout::{TabLayoutMode, TabStripConfig};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, CssProvider, Label, Orientation, PolicyType, ScrolledWindow};
use gtk4::{gdk::Display, style_context_add_provider_for_display};
use std::collections::HashMap;

/// Widgets for one tab entry in the strip.
pub struct TabStripItem {
    row: GtkBox,
    #[allow(dead_code)]
    select: Button,
    #[allow(dead_code)]
    close: Button,
    label: Label,
}

/// Tab strip UI with layout-mode-specific containers.
pub struct TabStrip {
    config: TabStripConfig,
    mode: TabLayoutMode,
    h_strip: GtkBox,
    h_tabs_box: GtkBox,
    h_new_tab: Button,
    v_sidebar: GtkBox,
    v_tabs_box: GtkBox,
    v_new_tab: Button,
    order: Vec<TabId>,
    entries: HashMap<TabId, TabStripItem>,
}

impl TabStrip {
    /// Creates an empty tab strip for `config.layout_mode`.
    pub fn new(config: TabStripConfig) -> Self {
        Self::install_tab_css(&config);

        let mode = config.layout_mode;

        let h_tabs_box = GtkBox::new(Orientation::Horizontal, 0);
        h_tabs_box.add_css_class("linked");

        let h_scroll = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(false)
            .has_frame(false)
            .propagate_natural_width(false)
            .build();
        h_scroll.set_policy(PolicyType::Automatic, PolicyType::Never);
        h_scroll.set_child(Some(&h_tabs_box));

        let h_new_tab = Button::with_mnemonic("New _Tab");
        let h_strip = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(6)
            .margin_end(6)
            .build();
        h_strip.append(&h_scroll);
        h_strip.append(&h_new_tab);

        let v_tabs_box = GtkBox::new(Orientation::Vertical, 4);
        v_tabs_box.add_css_class("linked");

        let v_scroll = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .has_frame(false)
            .propagate_natural_height(false)
            .build();
        v_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        v_scroll.set_child(Some(&v_tabs_box));

        let v_new_tab = Button::with_mnemonic("New _Tab");
        let v_sidebar = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .width_request(config.vertical_sidebar_width)
            .build();
        v_sidebar.append(&v_scroll);
        v_sidebar.append(&v_new_tab);

        Self {
            config,
            mode,
            h_strip,
            h_tabs_box,
            h_new_tab,
            v_sidebar,
            v_tabs_box,
            v_new_tab,
            order: Vec::new(),
            entries: HashMap::new(),
        }
    }

    /// Current layout mode.
    pub fn layout_mode(&self) -> TabLayoutMode {
        self.mode
    }

    /// Updates layout mode and reparents existing tab widgets into the new container.
    pub fn set_layout_mode(&mut self, mode: TabLayoutMode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.config.layout_mode = mode;
        self.reparent_all_tabs();
    }

    /// Invokes `f` for each New Tab button (horizontal and vertical).
    pub fn for_each_new_tab_button<F: Fn(&Button)>(&self, f: F) {
        f(&self.h_new_tab);
        f(&self.v_new_tab);
    }

    /// Top strip widget used in horizontal window layout.
    pub fn horizontal_strip(&self) -> &GtkBox {
        &self.h_strip
    }

    /// Left sidebar widget used in vertical window layout.
    pub fn vertical_sidebar(&self) -> &GtkBox {
        &self.v_sidebar
    }

    /// Adds a tab control.
    pub fn add_tab(
        &mut self,
        id: TabId,
        title: &str,
        on_select: impl Fn(TabId) + 'static,
        on_close: impl Fn(TabId) + 'static,
    ) {
        let item = Self::build_tab_item(self.mode, &self.config, id, title, on_select, on_close);
        self.tabs_box_for(self.mode).append(&item.row);
        self.order.push(id);
        self.entries.insert(id, item);
    }

    /// Removes a tab's widgets from the strip.
    pub fn remove_tab(&mut self, id: TabId) {
        if let Some(item) = self.entries.remove(&id)
            && let Some(parent) = item.row.parent()
            && let Ok(box_parent) = parent.downcast::<GtkBox>()
        {
            box_parent.remove(&item.row);
        }
        self.order.retain(|&tid| tid != id);
    }

    /// Highlights the active tab.
    pub fn set_active_tab(&self, active_id: TabId) {
        for (&id, item) in &self.entries {
            if id == active_id {
                item.row.add_css_class("active");
            } else {
                item.row.remove_css_class("active");
            }
        }
    }

    /// Updates the visible title for a tab button.
    pub fn set_tab_title(&self, id: TabId, title: &str) {
        if let Some(item) = self.entries.get(&id) {
            let display = if title.is_empty() { "New Tab" } else { title };
            item.label.set_text(display);
        }
    }

    fn tabs_box_for(&self, mode: TabLayoutMode) -> &GtkBox {
        match mode {
            TabLayoutMode::Horizontal => &self.h_tabs_box,
            TabLayoutMode::Vertical => &self.v_tabs_box,
        }
    }

    fn reparent_all_tabs(&mut self) {
        let order = self.order.clone();
        let mode = self.mode;
        let config = self.config;
        for id in order {
            let Some(row) = self.entries.get(&id).map(|i| i.row.clone()) else {
                continue;
            };
            if let Some(parent) = row.parent()
                && let Ok(box_parent) = parent.downcast::<GtkBox>()
            {
                box_parent.remove(&row);
            }
            Self::apply_row_layout(mode, &config, &row);
            self.tabs_box_for(mode).append(&row);
        }
    }

    fn build_tab_item(
        mode: TabLayoutMode,
        config: &TabStripConfig,
        id: TabId,
        title: &str,
        on_select: impl Fn(TabId) + 'static,
        on_close: impl Fn(TabId) + 'static,
    ) -> TabStripItem {
        let label = Label::builder()
            .label(if title.is_empty() { "New Tab" } else { title })
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .xalign(0.0)
            .build();

        let select = Button::builder().child(&label).hexpand(true).build();
        let close = Button::builder()
            .label("×")
            .tooltip_text("Close tab")
            .build();

        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .build();
        row.add_css_class("kp-tab-row");
        row.append(&select);
        row.append(&close);

        Self::apply_row_layout(mode, config, &row);

        let tab_id = id;
        select.connect_clicked(move |_| on_select(tab_id));

        let tab_id = id;
        close.connect_clicked(move |_| on_close(tab_id));

        TabStripItem {
            row,
            select,
            close,
            label,
        }
    }

    fn apply_row_layout(mode: TabLayoutMode, config: &TabStripConfig, row: &GtkBox) {
        match mode {
            TabLayoutMode::Horizontal => {
                row.remove_css_class("vertical-tab");
                row.set_hexpand(false);
                row.set_vexpand(false);
                row.set_size_request(config.min_tab_width, -1);
            }
            TabLayoutMode::Vertical => {
                row.add_css_class("vertical-tab");
                row.set_hexpand(true);
                row.set_size_request(-1, 36);
            }
        }
    }

    fn install_tab_css(config: &TabStripConfig) {
        let css = format!(
            "
.kp-tab-row {{
  min-width: {min_w}px;
  max-width: {max_w}px;
}}
.kp-tab-row.vertical-tab {{
  min-width: unset;
  max-width: unset;
  min-height: 32px;
}}
",
            min_w = config.min_tab_width,
            max_w = config.max_tab_width,
        );
        let provider = CssProvider::new();
        provider.load_from_data(&css);
        if let Some(display) = Display::default() {
            style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}
