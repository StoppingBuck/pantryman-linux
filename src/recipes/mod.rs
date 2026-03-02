/// Recipes tab: recipe list with availability indicators and detail view.
use crate::app::{App, AppMsg};
use crate::i18n;
use crate::ui_constants::*;
use janus_engine::DataManager;
use libadwaita as adw;
use relm4::gtk;
use relm4::{gtk::prelude::*, ComponentSender, RelmWidgetExt};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the full Recipes tab widget.
///
/// Returns `(tab_widget, recipe_list_box, recipe_detail_box)`.
pub fn build_recipes_tab(
    dm: &Option<Rc<RefCell<DataManager>>>,
    sender: ComponentSender<App>,
) -> (gtk::Widget, gtk::ListBox, gtk::Box) {
    let s = i18n::strings();

    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_hexpand(true);
    paned.set_vexpand(true);
    paned.set_position(LIST_PANE_WIDTH);

    // ── Left: search + recipe list ────────────────────────────────────────────
    let left = gtk::Box::new(gtk::Orientation::Vertical, 0);
    left.set_width_request(200);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(s.search_recipes));
    search.set_margin_top(DEFAULT_MARGIN);
    search.set_margin_bottom(DEFAULT_MARGIN);
    search.set_margin_start(DEFAULT_MARGIN);
    search.set_margin_end(DEFAULT_MARGIN);
    left.append(&search);

    left.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let list_scroll = gtk::ScrolledWindow::new();
    list_scroll.set_vexpand(true);
    list_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let recipe_list = gtk::ListBox::new();
    recipe_list.set_selection_mode(gtk::SelectionMode::Single);
    recipe_list.add_css_class("navigation-sidebar");
    list_scroll.set_child(Some(&recipe_list));
    left.append(&list_scroll);

    // Add recipe button
    let add_btn = gtk::Button::with_label(s.add_recipe);
    add_btn.add_css_class("flat");
    add_btn.set_margin_all(DEFAULT_MARGIN);
    {
        let sender_add = sender.clone();
        add_btn.connect_clicked(move |_| sender_add.input(AppMsg::AddRecipe));
    }
    left.append(&add_btn);

    // ── Right: recipe detail ──────────────────────────────────────────────────
    let detail_scroll = gtk::ScrolledWindow::new();
    detail_scroll.set_hexpand(true);
    detail_scroll.set_vexpand(true);
    detail_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let recipe_detail = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
    recipe_detail.set_margin_all(DEFAULT_MARGIN);
    detail_scroll.set_child(Some(&recipe_detail));
    show_recipe_placeholder(&recipe_detail);

    paned.set_start_child(Some(&left));
    paned.set_end_child(Some(&detail_scroll));

    // Populate initial list
    populate_recipe_list(&recipe_list, dm, "", &sender);

    // Search handler
    {
        let sender_search = sender.clone();
        search.connect_search_changed(move |entry| {
            sender_search.input(AppMsg::SearchRecipes(entry.text().to_string()));
        });
    }

    // Row selection handler
    {
        let sender_select = sender.clone();
        recipe_list.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let title = row.widget_name().to_string();
                if !title.is_empty() && title != "__empty__" {
                    sender_select.input(AppMsg::SelectRecipe(Some(title)));
                }
            }
        });
    }

    (paned.upcast(), recipe_list, recipe_detail)
}

/// Rebuild the recipe list with an optional search query.
pub fn populate_recipe_list(
    list: &gtk::ListBox,
    dm: &Option<Rc<RefCell<DataManager>>>,
    search: &str,
    _sender: &ComponentSender<App>,
) {
    let s = i18n::strings();
    crate::utils::clear_list_box(list);

    let Some(dm) = dm else {
        list.append(&empty_state_row(s.no_data_dir));
        return;
    };

    let dm = dm.borrow();
    let recipes = if search.is_empty() {
        let mut all: Vec<_> = dm.get_all_recipes().iter().collect();
        all.sort_by(|a, b| a.title.cmp(&b.title));
        all
    } else {
        dm.search_recipes(search)
    };

    if recipes.is_empty() {
        list.append(&empty_state_row(s.no_recipes_found));
        return;
    }

    for recipe in recipes {
        let cov = recipe.pantry_coverage(&dm);
        let row = build_recipe_row(recipe, cov);
        list.append(&row);
    }
}

fn build_recipe_row(
    recipe: &janus_engine::Recipe,
    cov: janus_engine::PantryCoverage,
) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_widget_name(&recipe.title);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    hbox.set_margin_top(ROW_SPACING);
    hbox.set_margin_bottom(ROW_SPACING);
    hbox.set_margin_start(DEFAULT_MARGIN);
    hbox.set_margin_end(DEFAULT_MARGIN);

    // Pie-chart availability indicator
    let tooltip = i18n::fmt_required_tooltip(cov.required_in_stock, cov.required_total);
    let pie = crate::utils::build_coverage_pie(cov.required_ratio(), &tooltip);
    hbox.append(&pie);

    let title_label = gtk::Label::new(Some(&recipe.title));
    title_label.set_hexpand(true);
    title_label.set_halign(gtk::Align::Start);
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    hbox.append(&title_label);

    row.set_child(Some(&hbox));
    row
}

/// Update the recipe detail panel for the selected recipe title.
pub fn update_recipe_detail(
    detail: &gtk::Box,
    dm: &Option<Rc<RefCell<DataManager>>>,
    title: &str,
    sender: &ComponentSender<App>,
) {
    let s = i18n::strings();
    crate::utils::clear_box(detail);

    let Some(dm_rc) = dm else {
        show_recipe_placeholder(detail);
        return;
    };

    let dm = dm_rc.borrow();
    let Some(recipe) = dm.get_recipe(title) else {
        show_recipe_placeholder(detail);
        return;
    };

    // ── Header ────────────────────────────────────────────────────────────────
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    header_box.set_halign(gtk::Align::Fill);

    let title_label = gtk::Label::new(Some(&recipe.title));
    title_label.add_css_class("title-1");
    title_label.set_halign(gtk::Align::Start);
    title_label.set_hexpand(true);
    title_label.set_wrap(true);
    header_box.append(&title_label);

    // Edit/Delete buttons
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
    btn_box.set_valign(gtk::Align::Start);

    let edit_btn = gtk::Button::with_label(s.edit);
    edit_btn.add_css_class("flat");
    {
        let sender_edit = sender.clone();
        let title_clone = title.to_string();
        edit_btn.connect_clicked(move |_| {
            sender_edit.input(AppMsg::EditRecipe(title_clone.clone()));
        });
    }

    let delete_btn = gtk::Button::with_label(s.delete);
    delete_btn.add_css_class("flat");
    delete_btn.add_css_class("destructive-action");
    {
        let sender_delete = sender.clone();
        let title_clone = title.to_string();
        delete_btn.connect_clicked(move |btn| {
            let window = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            show_delete_recipe_confirm(window.as_ref(), &title_clone, &sender_delete);
        });
    }

    btn_box.append(&edit_btn);
    btn_box.append(&delete_btn);
    header_box.append(&btn_box);
    detail.append(&header_box);

    // ── Meta info ─────────────────────────────────────────────────────────────
    let meta_box = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
    meta_box.set_halign(gtk::Align::Start);

    let mut has_meta = false;
    if let Some(prep) = recipe.prep_time {
        let label = gtk::Label::new(Some(&i18n::fmt_prep_time(prep)));
        label.add_css_class("caption");
        meta_box.append(&label);
        has_meta = true;
    }
    if let Some(down) = recipe.downtime {
        let label = gtk::Label::new(Some(&i18n::fmt_cook_time(down)));
        label.add_css_class("caption");
        meta_box.append(&label);
        has_meta = true;
    }
    if let Some(servings) = recipe.servings {
        let label = gtk::Label::new(Some(&i18n::fmt_servings(servings)));
        label.add_css_class("caption");
        meta_box.append(&label);
        has_meta = true;
    }
    if has_meta {
        detail.append(&meta_box);
    }

    // Tags
    if let Some(tags) = &recipe.tags {
        if !tags.is_empty() {
            let tags_label = gtk::Label::new(Some(&i18n::fmt_tags(&tags.join(", "))));
            tags_label.add_css_class("caption");
            tags_label.add_css_class("dim-label");
            tags_label.set_halign(gtk::Align::Start);
            detail.append(&tags_label);
        }
    }

    detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // ── Ingredients ───────────────────────────────────────────────────────────
    let ing_header = gtk::Label::new(Some(s.ingredients_heading));
    ing_header.add_css_class("heading");
    ing_header.set_halign(gtk::Align::Start);
    detail.append(&ing_header);

    let cov = recipe.pantry_coverage(&dm);
    let status_label = if cov.is_cookable() {
        let lbl = gtk::Label::new(Some(s.all_required_available));
        lbl.add_css_class("success");
        lbl
    } else {
        let missing = cov.required_total - cov.required_in_stock;
        let lbl = gtk::Label::new(Some(&i18n::fmt_missing_required(missing)));
        lbl.add_css_class("dim-label");
        lbl
    };
    status_label.set_halign(gtk::Align::Start);
    detail.append(&status_label);

    for ing in &recipe.ingredients {
        let in_pantry = dm.is_in_pantry(&ing.ingredient);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, ROW_SPACING);
        row.set_margin_start(DEFAULT_MARGIN);

        let dot = if in_pantry {
            let d = gtk::Label::new(Some("●"));
            d.add_css_class("success");
            d
        } else if ing.optional {
            let d = gtk::Label::new(Some("○"));
            d.add_css_class("dim-label");
            d
        } else {
            let d = gtk::Label::new(Some("○"));
            d.add_css_class("error");
            d
        };
        row.append(&dot);

        let display_name = dm.recipe_ingredient_display_name(ing);
        let mut qty_str = match (&ing.quantity, &ing.quantity_type) {
            (Some(q), Some(u)) if !u.is_empty() => format!("{} {} {}", q, u, display_name),
            (Some(q), _) => format!("{} {}", q, display_name),
            _ => display_name,
        };
        if let Some(note) = &ing.note {
            qty_str = format!("{} ({})", qty_str, note);
        }
        if ing.optional {
            qty_str = format!("{}{}", qty_str, s.optional_suffix);
        }
        let label = gtk::Label::new(Some(&qty_str));
        label.set_halign(gtk::Align::Start);
        if !in_pantry {
            label.add_css_class("dim-label");
        }
        row.append(&label);

        detail.append(&row);
    }

    // ── Instructions ──────────────────────────────────────────────────────────
    if !recipe.instructions.is_empty() {
        detail.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        let instr_header = gtk::Label::new(Some(s.instructions_heading));
        instr_header.add_css_class("heading");
        instr_header.set_halign(gtk::Align::Start);
        detail.append(&instr_header);

        let instr_label = gtk::Label::new(Some(&recipe.instructions));
        instr_label.set_halign(gtk::Align::Start);
        instr_label.set_wrap(true);
        instr_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        instr_label.set_selectable(true);
        instr_label.set_xalign(0.0);
        detail.append(&instr_label);
    }
}

fn show_delete_recipe_confirm(
    parent: Option<&gtk::Window>,
    title: &str,
    sender: &ComponentSender<App>,
) {
    use adw::prelude::*;
    let s = i18n::strings();

    let dialog = adw::MessageDialog::new(
        parent,
        Some(&i18n::fmt_delete_recipe_title(title)),
        Some(s.delete_recipe_body),
    );
    dialog.add_response("cancel", s.cancel);
    dialog.add_response("delete", s.delete);
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");

    let sender_clone = sender.clone();
    let title_owned = title.to_string();
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            sender_clone.input(AppMsg::DeleteRecipe(title_owned.clone()));
        }
    });
    dialog.present();
}

pub fn show_recipe_placeholder(detail: &gtk::Box) {
    let s = i18n::strings();
    crate::utils::clear_box(detail);
    let status = adw::StatusPage::new();
    status.set_icon_name(Some("emblem-documents-symbolic"));
    status.set_title(s.recipe_placeholder_title);
    status.set_description(Some(s.recipe_placeholder_desc));
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
