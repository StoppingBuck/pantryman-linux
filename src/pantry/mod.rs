/// Pantry tab: ingredient list with in-stock toggle and detail view.
use crate::app::{App, AppMsg};
use crate::i18n;
use crate::ui_constants::*;
use janus_engine::DataManager;
use libadwaita as adw;
use relm4::gtk;
use relm4::{gtk::prelude::*, ComponentSender, RelmWidgetExt};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the full Pantry tab widget.
///
/// Returns `(tab_widget, ingredient_list_box, ingredient_detail_box, in_stock_switch)`.
pub fn build_pantry_tab(
    dm: &Option<Rc<RefCell<DataManager>>>,
    in_stock_only: bool,
    sender: ComponentSender<App>,
) -> (gtk::Widget, gtk::ListBox, gtk::Box, gtk::Switch) {
    let s = i18n::strings();

    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_position(LIST_PANE_WIDTH);

    // ── Left: controls + ingredient list ─────────────────────────────────────
    let left = gtk::Box::new(gtk::Orientation::Vertical, 0);
    left.set_width_request(200);

    // Search bar
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(s.search_pantry));
    search.set_margin_top(DEFAULT_MARGIN);
    search.set_margin_bottom(ROW_SPACING);
    search.set_margin_start(DEFAULT_MARGIN);
    search.set_margin_end(DEFAULT_MARGIN);
    left.append(&search);

    // Filter controls
    let filter_box = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    filter_box.set_margin_start(DEFAULT_MARGIN);
    filter_box.set_margin_end(DEFAULT_MARGIN);
    filter_box.set_margin_bottom(ROW_SPACING);

    let in_stock_label = gtk::Label::new(Some(s.in_stock_only_label));
    in_stock_label.set_hexpand(true);
    in_stock_label.set_halign(gtk::Align::Start);

    let in_stock_switch = gtk::Switch::new();
    in_stock_switch.set_active(in_stock_only);
    in_stock_switch.set_valign(gtk::Align::Center);

    filter_box.append(&in_stock_label);
    filter_box.append(&in_stock_switch);
    left.append(&filter_box);

    left.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Ingredient list
    let list_scroll = gtk::ScrolledWindow::new();
    list_scroll.set_vexpand(true);
    list_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let pantry_list = gtk::ListBox::new();
    pantry_list.set_selection_mode(gtk::SelectionMode::Single);
    pantry_list.add_css_class("navigation-sidebar");
    list_scroll.set_child(Some(&pantry_list));
    left.append(&list_scroll);

    // Add ingredient button
    let add_btn = gtk::Button::with_label(s.add_ingredient_btn);
    add_btn.add_css_class("flat");
    add_btn.set_margin_all(DEFAULT_MARGIN);
    {
        let sender_add = sender.clone();
        add_btn.connect_clicked(move |_| sender_add.input(AppMsg::AddIngredient));
    }
    left.append(&add_btn);

    // ── Right: ingredient detail ──────────────────────────────────────────────
    let detail_scroll = gtk::ScrolledWindow::new();
    detail_scroll.set_hexpand(true);
    detail_scroll.set_vexpand(true);
    detail_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let ingredient_detail = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    ingredient_detail.set_margin_all(DEFAULT_MARGIN);
    detail_scroll.set_child(Some(&ingredient_detail));
    show_ingredient_placeholder(&ingredient_detail);

    paned.set_start_child(Some(&left));
    paned.set_end_child(Some(&detail_scroll));

    // Populate initial list
    populate_pantry_list(&pantry_list, dm, "", &[], in_stock_only, &sender);

    // Search handler — sends message to update model, update_view will rebuild list
    {
        let sender_search = sender.clone();
        search.connect_search_changed(move |entry| {
            sender_search.input(AppMsg::SearchIngredients(entry.text().to_string()));
        });
    }

    // In-stock toggle handler
    {
        let sender_switch = sender.clone();
        in_stock_switch.connect_active_notify(move |sw| {
            sender_switch.input(AppMsg::ToggleInStockOnly(sw.is_active()));
        });
    }

    // Row selection handler
    {
        let sender_select = sender.clone();
        pantry_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let name = row.widget_name().to_string();
                if !name.is_empty() && name != "__empty__" && name != "__header__" {
                    sender_select.input(AppMsg::SelectIngredient(Some(name)));
                }
            }
        });
    }

    (paned.upcast(), pantry_list, ingredient_detail, in_stock_switch)
}

/// Rebuild the pantry ingredient list with current filters.
pub fn populate_pantry_list(
    list: &gtk::ListBox,
    dm: &Option<Rc<RefCell<DataManager>>>,
    search: &str,
    categories: &[String],
    in_stock_only: bool,
    _sender: &ComponentSender<App>,
) {
    let s = i18n::strings();
    crate::utils::clear_list_box(list);

    let Some(dm) = dm else {
        list.append(&empty_state_row(s.no_data_dir));
        return;
    };

    let dm = dm.borrow();
    let mut ingredients = dm.filter_ingredients(search, categories, in_stock_only);
    // Sort by (category, name), empty category sorts last via sentinel
    ingredients.sort_by(|a, b| {
        let ca = if a.category.is_empty() { "\u{FFFF}" } else { a.category.as_str() };
        let cb = if b.category.is_empty() { "\u{FFFF}" } else { b.category.as_str() };
        ca.cmp(cb).then_with(|| a.name.cmp(&b.name))
    });

    if ingredients.is_empty() {
        list.append(&empty_state_row(s.no_ingredients_found));
        return;
    }

    let mut current_category: Option<String> = None;
    for ing in &ingredients {
        let cat = if ing.category.is_empty() { s.uncategorised } else { ing.category.as_str() };
        if current_category.as_deref() != Some(cat) {
            list.append(&build_category_header_row(cat));
            current_category = Some(cat.to_string());
        }
        let in_pantry = dm.is_in_pantry(&ing.name);
        let row = build_ingredient_row(ing, in_pantry);
        list.append(&row);
    }
}

fn build_ingredient_row(
    ing: &janus_engine::Ingredient,
    in_pantry: bool,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&ing.name);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    hbox.set_margin_top(ROW_SPACING);
    hbox.set_margin_bottom(ROW_SPACING);
    hbox.set_margin_start(DEFAULT_MARGIN);
    hbox.set_margin_end(DEFAULT_MARGIN);

    // Pantry status dot
    let dot = gtk::Label::new(Some(if in_pantry { "●" } else { "○" }));
    if in_pantry {
        dot.add_css_class("success");
    } else {
        dot.add_css_class("dim-label");
    }
    hbox.append(&dot);

    // Ingredient name
    let name_label = gtk::Label::new(Some(&ing.name));
    name_label.set_hexpand(true);
    name_label.set_halign(gtk::Align::Start);
    hbox.append(&name_label);

    row.set_child(Some(&hbox));
    row
}

fn build_category_header_row(category: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name("__header__");
    row.set_activatable(false);
    row.set_selectable(false);

    let label = gtk::Label::new(Some(category));
    label.add_css_class("heading");
    label.set_halign(gtk::Align::Start);
    label.set_margin_top(DEFAULT_MARGIN);
    label.set_margin_bottom(ROW_SPACING);
    label.set_margin_start(DEFAULT_MARGIN);
    label.set_margin_end(DEFAULT_MARGIN);
    row.set_child(Some(&label));
    row
}

/// Update the ingredient detail panel for the selected ingredient.
pub fn update_ingredient_detail(
    detail: &gtk::Box,
    dm: &Option<Rc<RefCell<DataManager>>>,
    name: &str,
    sender: &ComponentSender<App>,
) {
    let s = i18n::strings();
    crate::utils::clear_box(detail);

    let Some(dm_rc) = dm else {
        show_ingredient_placeholder(detail);
        return;
    };

    let dm = dm_rc.borrow();
    let Some(ing) = dm.get_ingredient(name) else {
        show_ingredient_placeholder(detail);
        return;
    };

    // ── Header ────────────────────────────────────────────────────────────────
    let title = gtk::Label::new(Some(&ing.name));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    title.set_wrap(true);
    detail.append(&title);

    // Category (omit if empty)
    if !ing.category.is_empty() {
        let cat = gtk::Label::new(Some(&i18n::fmt_category(&ing.category)));
        cat.add_css_class("caption");
        cat.add_css_class("dim-label");
        cat.set_halign(gtk::Align::Start);
        detail.append(&cat);
    }

    // Tags
    if let Some(tags) = &ing.tags {
        if !tags.is_empty() {
            let tags_label = gtk::Label::new(Some(&i18n::fmt_tags(&tags.join(", "))));
            tags_label.add_css_class("caption");
            tags_label.set_halign(gtk::Align::Start);
            detail.append(&tags_label);
        }
    }

    detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // ── Pantry status ─────────────────────────────────────────────────────────
    let pantry_header = gtk::Label::new(Some(s.pantry_heading));
    pantry_header.add_css_class("heading");
    pantry_header.set_halign(gtk::Align::Start);
    detail.append(&pantry_header);

    if let Some(item) = dm.get_pantry_item(name) {
        let status = gtk::Label::new(Some(s.in_stock_status));
        status.add_css_class("success");
        status.set_halign(gtk::Align::Start);
        detail.append(&status);

        if let Some(qty) = item.quantity {
            let qty_label = gtk::Label::new(Some(&i18n::fmt_quantity(qty, &item.quantity_type)));
            qty_label.set_halign(gtk::Align::Start);
            detail.append(&qty_label);
        }

        let updated = gtk::Label::new(Some(&i18n::fmt_last_updated(&item.last_updated)));
        updated.add_css_class("caption");
        updated.add_css_class("dim-label");
        updated.set_halign(gtk::Align::Start);
        detail.append(&updated);
    } else {
        let status = gtk::Label::new(Some(s.not_in_stock_status));
        status.add_css_class("error");
        status.set_halign(gtk::Align::Start);
        detail.append(&status);
    }

    // ── Used in recipes ───────────────────────────────────────────────────────
    let recipes = dm.get_recipes_with_ingredient(name);
    if !recipes.is_empty() {
        detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let recipes_header = gtk::Label::new(Some(s.used_in_recipes));
        recipes_header.add_css_class("heading");
        recipes_header.set_halign(gtk::Align::Start);
        detail.append(&recipes_header);

        for recipe in &recipes {
            let cov = recipe.pantry_coverage(&dm);
            let tooltip = i18n::fmt_required_tooltip(cov.required_in_stock, cov.required_total);
            let recipe_row = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
            let pie = crate::utils::build_coverage_pie(cov.required_ratio(), &tooltip);
            recipe_row.append(&pie);
            let recipe_label = gtk::Label::new(Some(&recipe.title));
            recipe_label.set_halign(gtk::Align::Start);
            recipe_row.append(&recipe_label);
            detail.append(&recipe_row);
        }
    }

    detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // ── Action buttons ────────────────────────────────────────────────────────
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    btn_box.set_halign(gtk::Align::End);

    let edit_btn = gtk::Button::with_label(s.edit);
    edit_btn.add_css_class("flat");
    {
        let sender_edit = sender.clone();
        let name_clone = name.to_string();
        edit_btn.connect_clicked(move |_| {
            sender_edit.input(AppMsg::EditIngredient(name_clone.clone()));
        });
    }

    let delete_btn = gtk::Button::with_label(s.delete);
    delete_btn.add_css_class("flat");
    delete_btn.add_css_class("destructive-action");
    {
        let sender_delete = sender.clone();
        let name_clone = name.to_string();
        delete_btn.connect_clicked(move |btn| {
            let window = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            show_delete_ingredient_confirm(window.as_ref(), &name_clone, &sender_delete);
        });
    }

    btn_box.append(&edit_btn);
    btn_box.append(&delete_btn);
    detail.append(&btn_box);
}

fn show_delete_ingredient_confirm(
    parent: Option<&gtk::Window>,
    name: &str,
    sender: &ComponentSender<App>,
) {
    use adw::prelude::*;
    let s = i18n::strings();

    let dialog = adw::MessageDialog::new(
        parent,
        Some(&i18n::fmt_delete_ingredient_title(name)),
        Some(s.delete_ingredient_body),
    );
    dialog.add_response("cancel", s.cancel);
    dialog.add_response("delete", s.delete);
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let sender_clone = sender.clone();
    let name_owned = name.to_string();
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            sender_clone.input(AppMsg::DeleteIngredient(name_owned.clone()));
        }
    });
    dialog.present();
}

pub fn show_ingredient_placeholder(detail: &gtk::Box) {
    let s = i18n::strings();
    crate::utils::clear_box(detail);
    let status = adw::StatusPage::new();
    status.set_icon_name(Some("view-list-symbolic"));
    status.set_title(s.ingredient_placeholder_title);
    status.set_description(Some(s.ingredient_placeholder_desc));
    status.set_vexpand(true);
    detail.append(&status);
}

fn empty_state_row(text: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name("__empty__");
    row.set_activatable(false);
    row.set_selectable(false);
    let label = gtk::Label::new(Some(text));
    label.add_css_class("dim-label");
    label.set_margin_top(DEFAULT_MARGIN);
    label.set_margin_bottom(DEFAULT_MARGIN);
    row.set_child(Some(&label));
    row
}
