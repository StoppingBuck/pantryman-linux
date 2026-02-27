/// Knowledge Base tab: lists KB articles and shows their content.
use crate::app::{App, AppMsg};
use crate::ui_constants::*;
use janus_engine::DataManager;
use libadwaita as adw;
use relm4::gtk;
use relm4::{gtk::prelude::*, ComponentSender, RelmWidgetExt};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the full Knowledge Base tab widget.
///
/// Returns `(tab_widget, kb_list_box, kb_detail_box)`.
pub fn build_kb_tab(
    dm: &Option<Rc<RefCell<DataManager>>>,
    sender: ComponentSender<App>,
) -> (gtk::Widget, gtk::ListBox, gtk::Box) {
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_position(LIST_PANE_WIDTH);

    // ── Left: article list ────────────────────────────────────────────────────
    let left = gtk::Box::new(gtk::Orientation::Vertical, 0);
    left.set_width_request(200);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search articles…"));
    search.set_margin_top(DEFAULT_MARGIN);
    search.set_margin_bottom(DEFAULT_MARGIN);
    search.set_margin_start(DEFAULT_MARGIN);
    search.set_margin_end(DEFAULT_MARGIN);
    left.append(&search);

    let list_scroll = gtk::ScrolledWindow::new();
    list_scroll.set_vexpand(true);
    list_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let kb_list = gtk::ListBox::new();
    kb_list.set_selection_mode(gtk::SelectionMode::Single);
    kb_list.add_css_class("navigation-sidebar");
    list_scroll.set_child(Some(&kb_list));
    left.append(&list_scroll);

    // ── Right: article detail ─────────────────────────────────────────────────
    let detail_scroll = gtk::ScrolledWindow::new();
    detail_scroll.set_hexpand(true);
    detail_scroll.set_vexpand(true);
    detail_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let kb_detail = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    kb_detail.set_margin_all(DEFAULT_MARGIN);
    detail_scroll.set_child(Some(&kb_detail));

    show_kb_placeholder(&kb_detail);

    paned.set_start_child(Some(&left));
    paned.set_end_child(Some(&detail_scroll));

    // Populate initial list
    populate_kb_list(&kb_list, dm, &sender);

    // Search filter
    {
        let kb_list_clone = kb_list.clone();
        let dm_clone = dm.clone();
        let sender_search = sender.clone();
        search.connect_search_changed(move |entry| {
            let query = entry.text().to_string().to_lowercase();
            crate::utils::clear_list_box(&kb_list_clone);
            populate_kb_list_filtered(&kb_list_clone, &dm_clone, &query, &sender_search);
        });
    }

    // Row selection
    {
        let sender_select = sender.clone();
        kb_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let slug = row.widget_name().to_string();
                if !slug.is_empty() {
                    sender_select.input(AppMsg::SelectKb(Some(slug)));
                }
            }
        });
    }

    (paned.upcast(), kb_list, kb_detail)
}

/// Populate the KB list box with all articles (no filter).
pub fn populate_kb_list(
    list: &gtk::ListBox,
    dm: &Option<Rc<RefCell<DataManager>>>,
    sender: &ComponentSender<App>,
) {
    populate_kb_list_filtered(list, dm, "", sender);
}

fn populate_kb_list_filtered(
    list: &gtk::ListBox,
    dm: &Option<Rc<RefCell<DataManager>>>,
    query: &str,
    _sender: &ComponentSender<App>,
) {
    crate::utils::clear_list_box(list);

    let Some(dm) = dm else {
        let row = empty_state_row("No data loaded");
        list.append(&row);
        return;
    };

    let dm = dm.borrow();
    let mut entries: Vec<_> = dm.get_all_kb_entries().into_iter().collect();
    entries.sort_by(|a, b| a.title.cmp(&b.title));

    let filtered: Vec<_> = entries
        .iter()
        .filter(|e| {
            query.is_empty()
                || e.title.to_lowercase().contains(query)
                || e.slug.to_lowercase().contains(query)
        })
        .collect();

    if filtered.is_empty() {
        let row = empty_state_row("No articles found");
        list.append(&row);
        return;
    }

    for entry in filtered {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&entry.slug);
        let label = gtk::Label::new(Some(&entry.title));
        label.set_halign(gtk::Align::Start);
        label.set_margin_top(ROW_SPACING);
        label.set_margin_bottom(ROW_SPACING);
        label.set_margin_start(DEFAULT_MARGIN);
        label.set_margin_end(DEFAULT_MARGIN);
        row.set_child(Some(&label));
        list.append(&row);
    }
}

/// Update the KB detail panel for the selected article slug.
pub fn update_kb_detail(
    detail: &gtk::Box,
    dm: &Option<Rc<RefCell<DataManager>>>,
    slug: &str,
) {
    crate::utils::clear_box(detail);

    let Some(dm) = dm else {
        show_kb_placeholder(detail);
        return;
    };

    let dm = dm.borrow();
    let Some(entry) = dm.get_kb_entry(slug) else {
        show_kb_placeholder(detail);
        return;
    };

    // Title
    let title = gtk::Label::new(Some(&entry.title));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    title.set_wrap(true);
    detail.append(&title);

    detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Content as plain text
    let content_label = gtk::Label::new(Some(&entry.content));
    content_label.set_halign(gtk::Align::Start);
    content_label.set_valign(gtk::Align::Start);
    content_label.set_wrap(true);
    content_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    content_label.set_selectable(true);
    content_label.set_xalign(0.0);
    detail.append(&content_label);

    // Linked ingredients
    let linked = dm.get_ingredients_with_kb_reference(slug);
    if !linked.is_empty() {
        detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let header = gtk::Label::new(Some("Related ingredients"));
        header.add_css_class("heading");
        header.set_halign(gtk::Align::Start);
        detail.append(&header);

        for ing in linked {
            let label = gtk::Label::new(Some(&ing.name));
            label.set_halign(gtk::Align::Start);
            label.add_css_class("caption");
            detail.append(&label);
        }
    }
}

pub fn show_kb_placeholder(detail: &gtk::Box) {
    crate::utils::clear_box(detail);
    let status = adw::StatusPage::new();
    status.set_icon_name(Some("system-help-symbolic"));
    status.set_title("Knowledge Base");
    status.set_description(Some("Select an article from the list to read it."));
    status.set_vexpand(true);
    detail.append(&status);
}

fn empty_state_row(text: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_selectable(false);
    let label = gtk::Label::new(Some(text));
    label.add_css_class("dim-label");
    label.set_margin_top(DEFAULT_MARGIN);
    label.set_margin_bottom(DEFAULT_MARGIN);
    row.set_child(Some(&label));
    row
}
