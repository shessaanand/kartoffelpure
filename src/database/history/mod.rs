//! Browsing history storage.

mod entry;
mod repository;
mod service;

pub use entry::HistoryEntry;
pub use repository::HistoryRepository;
pub use service::HistoryService;
