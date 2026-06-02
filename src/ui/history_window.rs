//! Browsing history list dialog.

use crate::database::history::{HistoryEntry, HistoryService};
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Dialog, Entry, GestureClick, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

const DIALOG_WIDTH: i32 = 640;
const DIALOG_HEIGHT: i32 = 480;

/// Modal dialog for viewing and managing browsing history.
pub struct HistoryDialog {
    dialog: Dialog,
    list_box: ListBox,
    search_entry: Entry,
    history: Rc<HistoryService>,
    on_open_url: Rc<dyn Fn(String)>,
}

impl HistoryDialog {
    /// Shows a history dialog owned by `parent`.
    pub fn show(
        parent: Option<&impl IsA<Window>>,
        history: Rc<HistoryService>,
        on_open_url: impl Fn(String) + 'static,
    ) {
        let shell = Self::build(parent, Rc::clone(&history), Rc::new(on_open_url));
        Self::refresh_list(&shell, "");
        shell.borrow().dialog.present();
    }

    fn build(
        parent: Option<&impl IsA<Window>>,
        history: Rc<HistoryService>,
        on_open_url: Rc<dyn Fn(String)>,
    ) -> Rc<RefCell<Self>> {
        let dialog = Dialog::builder()
            .title("History")
            .default_width(DIALOG_WIDTH)
            .default_height(DIALOG_HEIGHT)
            .modal(true)
            .build();

        if let Some(parent) = parent {
            dialog.set_transient_for(Some(parent));
        }

        let search_entry = Entry::builder()
            .placeholder_text("Search history")
            .hexpand(true)
            .build();

        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .build();

        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .min_content_height(200)
            .child(&list_box)
            .build();

        let clear_button = Button::with_mnemonic("Clear _All");
        let close_button = Button::with_mnemonic("_Close");

        let header = GtkBox::new(Orientation::Horizontal, 6);
        header.append(&search_entry);

        let footer = GtkBox::new(Orientation::Horizontal, 6);
        footer.set_halign(gtk4::Align::End);
        footer.append(&clear_button);
        footer.append(&close_button);

        let content = GtkBox::new(Orientation::Vertical, 8);
        content.set_margin_top(8);
        content.set_margin_bottom(8);
        content.set_margin_start(8);
        content.set_margin_end(8);
        content.append(&header);
        content.append(&scroll);
        content.append(&footer);
        dialog.content_area().append(&content);

        let shell = Rc::new(RefCell::new(Self {
            dialog: dialog.clone(),
            list_box: list_box.clone(),
            search_entry: search_entry.clone(),
            history,
            on_open_url,
        }));

        {
            let shell = Rc::clone(&shell);
            search_entry.connect_changed(move |entry| {
                Self::refresh_list(&shell, &entry.text());
            });
        }

        {
            let shell = Rc::clone(&shell);
            clear_button.connect_clicked(move |_| {
                shell.borrow().confirm_clear_all();
            });
        }

        close_button.connect_clicked({
            let dialog = dialog.clone();
            move |_| {
                dialog.close();
            }
        });

        dialog.connect_response(|dialog, _| {
            dialog.close();
        });

        shell
    }

    fn refresh_list(shell: &Rc<RefCell<Self>>, query: &str) {
        let this = shell.borrow();
        while let Some(row) = this.list_box.row_at_index(0) {
            this.list_box.remove(&row);
        }

        let entries = match this.history.search(query) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("history search failed: {err}");
                return;
            }
        };

        if entries.is_empty() {
            this.list_box.append(&empty_row());
            return;
        }

        for entry in entries {
            this.list_box
                .append(&Self::build_row(Rc::clone(shell), entry));
        }
    }

    fn build_row(shell: Rc<RefCell<Self>>, entry: HistoryEntry) -> ListBoxRow {
        let this = shell.borrow();
        let row = ListBoxRow::new();

        let title = Label::builder()
            .label(entry.display_title())
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        let url = Label::builder()
            .label(&entry.url)
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .css_classes(["dim-label"])
            .build();

        let time = Label::builder()
            .label(format_visited_at(entry.visited_at))
            .xalign(0.0)
            .css_classes(["dim-label"])
            .build();

        let text_col = GtkBox::new(Orientation::Vertical, 2);
        text_col.set_hexpand(true);
        text_col.append(&title);
        text_col.append(&url);
        text_col.append(&time);

        let delete = Button::builder().label("Delete").build();

        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.append(&text_col);
        row_box.append(&delete);
        row.set_child(Some(&row_box));

        let open_target = entry.url.clone();
        let on_open = Rc::clone(&this.on_open_url);
        let dialog = this.dialog.clone();
        drop(this);

        let open_gesture = GestureClick::new();
        open_gesture.connect_pressed(move |_, _, _, _| {
            on_open(open_target.clone());
            dialog.close();
        });
        text_col.add_controller(open_gesture);

        let entry_id = entry.id;
        delete.connect_clicked(move |_| {
            if let Err(err) = shell.borrow().history.delete_entry(entry_id) {
                eprintln!("delete history entry failed: {err}");
                return;
            }
            let query = shell.borrow().search_entry.text().to_string();
            Self::refresh_list(&shell, &query);
        });

        row
    }

    fn confirm_clear_all(&self) {
        let dialog = self.dialog.clone();
        let history = Rc::clone(&self.history);
        let list_box = self.list_box.clone();
        let search_entry = self.search_entry.clone();

        let confirm = gtk4::MessageDialog::builder()
            .transient_for(&dialog)
            .modal(true)
            .text("Clear all browsing history?")
            .secondary_text("This cannot be undone.")
            .buttons(gtk4::ButtonsType::OkCancel)
            .build();
        confirm.set_message_type(gtk4::MessageType::Warning);

        confirm.connect_response(move |confirm_dialog, response| {
            if response == gtk4::ResponseType::Ok {
                if let Err(err) = history.clear_all() {
                    eprintln!("clear history failed: {err}");
                } else {
                    while let Some(row) = list_box.row_at_index(0) {
                        list_box.remove(&row);
                    }
                    list_box.append(&empty_row());
                    search_entry.set_text("");
                }
            }
            confirm_dialog.close();
        });

        confirm.present();
    }
}

fn empty_row() -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_child(Some(&Label::new(Some("No history entries"))));
    row
}

fn format_visited_at(secs: i64) -> String {
    match gtk4::glib::DateTime::from_unix_utc(secs).and_then(|dt| dt.format_iso8601()) {
        Ok(value) => value.to_string(),
        Err(_) => secs.to_string(),
    }
}
