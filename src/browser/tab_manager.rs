//! In-memory tab collection and active-tab tracking for one window.

use super::tab::{Tab, TabId};

/// Owns all tabs in a window and which tab is currently active.
pub struct TabManager {
    tabs: Vec<Tab>,
    active_index: usize,
    next_id: u64,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
            next_id: 1,
        }
    }
}

impl TabManager {
    /// Returns the number of open tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns whether there are no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the id of the active tab, if any.
    pub fn active_id(&self) -> Option<TabId> {
        self.tabs.get(self.active_index).map(Tab::id)
    }

    /// Returns the active tab.
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_index)
    }

    /// Returns the active tab mutably.
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let index = self.active_index;
        self.tabs.get_mut(index)
    }

    /// Returns a tab by id.
    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id() == id)
    }

    /// Returns a tab by id mutably.
    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id() == id)
    }

    /// Iterates over all tabs in creation order.
    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    /// Appends a new tab, makes it active, and returns its id.
    pub fn create_tab(&mut self) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(Tab::new(id));
        self.active_index = self.tabs.len() - 1;
        id
    }

    /// Activates the tab with `id`. Returns false if the id is unknown.
    pub fn set_active(&mut self, id: TabId) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id() == id) {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Removes a tab unless it is the last one.
    ///
    /// Returns `Some(new_active_id)` on success, or `None` if the tab was not
    /// found or is the final tab.
    pub fn close_tab(&mut self, id: TabId) -> Option<TabId> {
        if self.tabs.len() <= 1 {
            return None;
        }
        let index = self.tabs.iter().position(|t| t.id() == id)?;
        let was_active = index == self.active_index;
        self.tabs.remove(index);

        if was_active {
            self.active_index = index.min(self.tabs.len() - 1);
        } else if index < self.active_index {
            self.active_index -= 1;
        }

        self.active_id()
    }
}
