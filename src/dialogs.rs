/// Modal dialogs for creating and editing recipes and ingredients.
///
/// Each dialog is a `gtk::Window` shown as a transient child of the main window.
/// When the user confirms, the dialog sends a message via the component sender.
use crate::i18n;
use janus_engine::{fold_for_matching, slugify, Ingredient, PantryItem, Recipe, RecipeIngredient};
use libadwaita as adw;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::ComponentSender;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{App, AppMsg};

/// Info about one ingredient from the database, used for live lookup in the recipe editor.
pub struct IngredientInfo {
    /// Canonical name (HashMap key, used when saving).
    pub name: String,
    /// All lowercase forms that should match (name, slug, translations).
    pub forms: Vec<String>,
    /// Whether this ingredient is currently in the pantry.
    pub in_pantry: bool,
}

// ─── Recipe dialog ────────────────────────────────────────────────────────────

/// Add a single ingredient row to the list and register it in `row_entries`.
///
/// `qty`, `ing`, `note`, and `optional` are pre-filled values.
fn add_ingredient_row(
    list: &gtk::ListBox,
    row_entries: &Rc<RefCell<Vec<(gtk::Entry, gtk::Entry, gtk::Entry, gtk::CheckButton)>>>,
    ingredients: &Rc<Vec<IngredientInfo>>,
    qty: &str,
    ing: &str,
    note: &str,
    optional: bool,
) {
    let s = i18n::strings();

    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row_box.set_margin_top(4);
    row_box.set_margin_bottom(4);
    row_box.set_margin_start(4);
    row_box.set_margin_end(4);

    let qty_entry = gtk::Entry::new();
    qty_entry.set_placeholder_text(Some(s.qty_placeholder));
    qty_entry.set_width_chars(8);
    qty_entry.set_max_width_chars(12);
    qty_entry.set_text(qty);

    let status_label = gtk::Label::new(Some("·"));
    status_label.set_width_chars(1);
    status_label.set_xalign(0.5);

    let ing_entry = gtk::Entry::new();
    ing_entry.set_placeholder_text(Some(s.ingredient_placeholder));
    ing_entry.set_hexpand(true);
    ing_entry.set_text(ing);

    let note_entry = gtk::Entry::new();
    note_entry.set_placeholder_text(Some(s.note_placeholder));
    note_entry.set_hexpand(true);
    note_entry.set_text(note);

    let optional_check = gtk::CheckButton::new();
    optional_check.set_active(optional);
    optional_check.set_tooltip_text(Some(s.optional_check_tooltip));
    optional_check.set_valign(gtk::Align::Center);

    let optional_label = gtk::Label::new(Some(s.optional_check_label));
    optional_label.add_css_class("dim-label");
    optional_label.set_valign(gtk::Align::Center);

    let remove_btn = gtk::Button::new();
    remove_btn.set_icon_name("list-remove-symbolic");
    remove_btn.add_css_class("circular");
    remove_btn.add_css_class("flat");

    row_box.append(&qty_entry);
    row_box.append(&status_label);
    row_box.append(&ing_entry);
    row_box.append(&note_entry);
    row_box.append(&optional_check);
    row_box.append(&optional_label);
    row_box.append(&remove_btn);

    let list_row = gtk::ListBoxRow::new();
    list_row.set_activatable(false);
    list_row.set_selectable(false);
    list_row.set_child(Some(&row_box));

    list.append(&list_row);
    row_entries
        .borrow_mut()
        .push((qty_entry.clone(), ing_entry.clone(), note_entry.clone(), optional_check.clone()));

    // Live status update
    let status_clone = status_label.clone();
    let ings_clone = Rc::clone(ingredients);
    let update_status = move |entry: &gtk::Entry| {
        let text = entry.text().to_string();
        let folded = fold_for_matching(text.trim());
        // Remove all colour classes first
        status_clone.remove_css_class("success");
        status_clone.remove_css_class("error");
        status_clone.remove_css_class("dim-label");
        if folded.is_empty() {
            status_clone.set_text("·");
        } else if let Some(info) = ings_clone.iter().find(|i| i.forms.iter().any(|f| f == &folded)) {
            if info.in_pantry {
                status_clone.set_text("●");
                status_clone.add_css_class("success");
            } else {
                status_clone.set_text("●");
                status_clone.add_css_class("error");
            }
        } else {
            status_clone.set_text("○");
            status_clone.add_css_class("dim-label");
        }
    };

    // Trigger once for pre-filled value
    update_status(&ing_entry);

    ing_entry.connect_changed(update_status);

    // Remove button: remove the ListBoxRow and the entry tuple
    let list_clone = list.clone();
    let entries_clone = Rc::clone(row_entries);
    let qty_clone = qty_entry.clone();
    let ing_clone = ing_entry.clone();
    remove_btn.connect_clicked(move |_| {
        list_clone.remove(&list_row);
        entries_clone
            .borrow_mut()
            .retain(|(q, i, _, _)| q != &qty_clone || i != &ing_clone);
    });
}

/// Show a dialog for adding (original = None) or editing (original = Some) a recipe.
pub fn show_recipe_dialog(
    parent: &impl gtk::prelude::IsA<gtk::Window>,
    ingredients: Vec<IngredientInfo>,
    existing: Option<&Recipe>,
    sender: ComponentSender<App>,
) {
    use adw::prelude::*;

    let s = i18n::strings();
    let is_edit = existing.is_some();
    let window = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(if is_edit { s.edit_recipe_dialog_title } else { s.add_recipe_dialog_title })
        .default_width(600)
        .default_height(700)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    // ── Form ─────────────────────────────────────────────────────────────────
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let form = gtk::Box::new(gtk::Orientation::Vertical, 12);
    form.set_margin_top(16);
    form.set_margin_bottom(16);
    form.set_margin_start(16);
    form.set_margin_end(16);

    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    group.set_title(s.details_group);

    // Title
    let title_row = adw::EntryRow::new();
    title_row.set_title(s.recipe_title_field);
    if let Some(r) = existing {
        title_row.set_text(&r.title);
    }
    group.add(&title_row);

    // Prep time
    let prep_row = adw::EntryRow::new();
    prep_row.set_title(s.prep_time_field);
    if let Some(r) = existing {
        if let Some(t) = r.prep_time {
            prep_row.set_text(&t.to_string());
        }
    }
    group.add(&prep_row);

    // Downtime (oven / resting)
    let down_row = adw::EntryRow::new();
    down_row.set_title(s.downtime_field);
    if let Some(r) = existing {
        if let Some(t) = r.downtime {
            down_row.set_text(&t.to_string());
        }
    }
    group.add(&down_row);

    // Servings
    let servings_row = adw::EntryRow::new();
    servings_row.set_title(s.servings_field);
    if let Some(r) = existing {
        if let Some(sv) = r.servings {
            servings_row.set_text(&sv.to_string());
        }
    }
    group.add(&servings_row);

    // Tags (comma-separated)
    let tags_row = adw::EntryRow::new();
    tags_row.set_title(s.tags_field);
    if let Some(r) = existing {
        if let Some(tags) = &r.tags {
            tags_row.set_text(&tags.join(", "));
        }
    }
    group.add(&tags_row);

    page.add(&group);

    // Ingredients group
    let ing_group = adw::PreferencesGroup::new();
    ing_group.set_title(s.ingredients_group);

    let ing_list = gtk::ListBox::new();
    ing_list.add_css_class("boxed-list");
    ing_list.set_selection_mode(gtk::SelectionMode::None);

    let all_ings: Rc<Vec<IngredientInfo>> = Rc::new(ingredients);
    let row_entries: Rc<RefCell<Vec<(gtk::Entry, gtk::Entry, gtk::Entry, gtk::CheckButton)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // Populate from existing recipe
    if let Some(r) = existing {
        for i in &r.ingredients {
            let qty_display = match (&i.quantity, &i.quantity_type) {
                (Some(q), Some(u)) if !u.is_empty() => format!("{} {}", q, u),
                (Some(q), _) => q.clone(),
                (None, _) => String::new(),
            };
            add_ingredient_row(
                &ing_list,
                &row_entries,
                &all_ings,
                &qty_display,
                &i.ingredient,
                i.note.as_deref().unwrap_or(""),
                i.optional,
            );
        }
    }

    // "Add ingredient" button
    let add_btn = gtk::Button::with_label(s.add_ingredient_row_btn);
    add_btn.add_css_class("flat");
    add_btn.set_halign(gtk::Align::Start);

    {
        let list_clone = ing_list.clone();
        let entries_clone = Rc::clone(&row_entries);
        let ings_clone = Rc::clone(&all_ings);
        add_btn.connect_clicked(move |_| {
            add_ingredient_row(&list_clone, &entries_clone, &ings_clone, "", "", "", false);
        });
    }

    let ing_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    ing_box.append(&ing_list);
    ing_box.append(&add_btn);

    ing_group.add(&ing_box);
    page.add(&ing_group);

    // Instructions group
    let instr_group = adw::PreferencesGroup::new();
    instr_group.set_title(s.instructions_group);

    let instr_text = gtk::TextView::new();
    instr_text.set_wrap_mode(gtk::WrapMode::Word);
    instr_text.set_top_margin(8);
    instr_text.set_bottom_margin(8);
    instr_text.set_left_margin(8);
    instr_text.set_right_margin(8);
    instr_text.add_css_class("card");
    instr_text.set_size_request(-1, 150);

    if let Some(r) = existing {
        instr_text.buffer().set_text(&r.instructions);
    }

    let instr_scroll = gtk::ScrolledWindow::new();
    instr_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    instr_scroll.set_min_content_height(150);
    instr_scroll.set_child(Some(&instr_text));

    instr_group.add(&instr_scroll);
    page.add(&instr_group);

    form.append(&page);
    scroll.set_child(Some(&form));

    // ── Buttons ───────────────────────────────────────────────────────────────
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(16);
    btn_box.set_margin_start(16);
    btn_box.set_margin_end(16);
    btn_box.set_halign(gtk::Align::End);

    let cancel_btn = gtk::Button::with_label(s.cancel);
    cancel_btn.add_css_class("pill");

    let save_btn = gtk::Button::with_label(if is_edit { s.save } else { s.add });
    save_btn.add_css_class("suggested-action");
    save_btn.add_css_class("pill");

    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.append(&scroll);
    outer.append(&btn_box);
    toolbar_view.set_content(Some(&outer));
    window.set_content(Some(&toolbar_view));

    // ── Event handlers ────────────────────────────────────────────────────────
    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| win_cancel.close());

    let win_save = window.clone();
    let original_title = existing.map(|r| r.title.clone());
    let row_entries_save = Rc::clone(&row_entries);
    let all_ings_save = Rc::clone(&all_ings);
    save_btn.connect_clicked(move |_| {
        let row_entries = &row_entries_save;
        let all_ings = &all_ings_save;
        let title = title_row.text().to_string();
        if title.trim().is_empty() {
            return;
        }
        let prep_time = prep_row.text().parse::<u32>().ok();
        let downtime = down_row.text().parse::<u32>().ok();
        let servings = servings_row.text().parse::<u32>().ok();
        let tags: Option<Vec<String>> = {
            let t = tags_row.text();
            if t.is_empty() {
                None
            } else {
                Some(
                    t.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            }
        };

        // Collect ingredients from row entries
        let ingredients: Vec<RecipeIngredient> = row_entries
            .borrow()
            .iter()
            .filter_map(|(qty_e, ing_e, note_e, opt_check)| {
                let ing_text = ing_e.text().to_string();
                if ing_text.trim().is_empty() {
                    return None;
                }
                let canonical = all_ings
                    .iter()
                    .find(|i| {
                        i.forms
                            .iter()
                            .any(|f| f == &ing_text.trim().to_lowercase())
                    })
                    .map(|i| i.name.clone())
                    .unwrap_or_else(|| ing_text.trim().to_string());
                let qty_text = qty_e.text().to_string();
                let quantity = if qty_text.trim().is_empty() {
                    None
                } else {
                    Some(qty_text.trim().to_string())
                };
                let note_text = note_e.text().to_string();
                let note = if note_text.trim().is_empty() {
                    None
                } else {
                    Some(note_text.trim().to_string())
                };
                Some(RecipeIngredient {
                    ingredient: canonical,
                    quantity,
                    quantity_type: None,
                    note,
                    optional: opt_check.is_active(),
                })
            })
            .collect();

        // Parse instructions
        let (start, end) = instr_text.buffer().bounds();
        let instructions = instr_text
            .buffer()
            .text(&start, &end, false)
            .to_string();

        let recipe = Recipe {
            title: title.trim().to_string(),
            slug: String::new(),
            file_stem: String::new(),
            ingredients,
            prep_time,
            downtime,
            servings,
            tags,
            image: None,
            instructions,
        };

        sender.input(AppMsg::SaveRecipe {
            original: original_title.clone(),
            recipe,
        });
        win_save.close();
    });

    window.present();
}

// ─── Ingredient dialog ────────────────────────────────────────────────────────

/// Show a dialog for adding or editing an ingredient, including pantry status.
pub fn show_ingredient_dialog(
    parent: &impl gtk::prelude::IsA<gtk::Window>,
    categories: Vec<String>,
    existing: Option<&Ingredient>,
    pantry_item: Option<&PantryItem>,
    sender: ComponentSender<App>,
) {
    use adw::prelude::*;

    let s = i18n::strings();
    let is_edit = existing.is_some();
    let window = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(if is_edit { s.edit_ingredient_dialog_title } else { s.add_ingredient_dialog_title })
        .default_width(480)
        .default_height(520)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let form = gtk::Box::new(gtk::Orientation::Vertical, 12);
    form.set_margin_top(16);
    form.set_margin_bottom(16);
    form.set_margin_start(16);
    form.set_margin_end(16);

    let page = adw::PreferencesPage::new();
    let details_group = adw::PreferencesGroup::new();
    details_group.set_title(s.details_group);

    // Name
    let name_row = adw::EntryRow::new();
    name_row.set_title(s.ingredient_name_field);
    if let Some(ing) = existing {
        name_row.set_text(&ing.name);
    }
    details_group.add(&name_row);

    // Plural
    let plural_row = adw::EntryRow::new();
    plural_row.set_title(s.ingredient_plural_field);
    if let Some(ing) = existing {
        if let Some(plural) = &ing.plural {
            plural_row.set_text(plural);
        }
    }
    details_group.add(&plural_row);

    // Category
    let cat_row = adw::EntryRow::new();
    let cat_title = if let Some(example) = categories.first() {
        i18n::fmt_category_hint(example)
    } else {
        s.ingredient_category_field.to_string()
    };
    cat_row.set_title(&cat_title);
    if let Some(ing) = existing {
        cat_row.set_text(&ing.category);
    }
    details_group.add(&cat_row);

    // Tags (comma-separated)
    let tags_row = adw::EntryRow::new();
    tags_row.set_title(s.ingredient_tags_field);
    if let Some(ing) = existing {
        if let Some(tags) = &ing.tags {
            tags_row.set_text(&tags.join(", "));
        }
    }
    details_group.add(&tags_row);

    page.add(&details_group);

    // Pantry group
    let pantry_group = adw::PreferencesGroup::new();
    pantry_group.set_title(s.pantry_group);

    let in_pantry_row = adw::SwitchRow::new();
    in_pantry_row.set_title(s.in_pantry_field);
    let in_pantry = pantry_item.is_some();
    in_pantry_row.set_active(in_pantry);
    pantry_group.add(&in_pantry_row);

    let qty_row = adw::EntryRow::new();
    qty_row.set_title(s.quantity_field);
    qty_row.set_sensitive(in_pantry);
    if let Some(item) = pantry_item {
        if let Some(q) = item.quantity {
            qty_row.set_text(&q.to_string());
        }
    }
    pantry_group.add(&qty_row);

    // Unit row: title shows localized unit examples as a hint
    let unit_hint = {
        let units = i18n::suggested_units();
        let sample = units.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        format!("{} ({})", s.unit_field, sample)
    };
    let unit_row = adw::EntryRow::new();
    unit_row.set_title(&unit_hint);
    unit_row.set_sensitive(in_pantry);
    if let Some(item) = pantry_item {
        unit_row.set_text(&item.quantity_type);
    }
    pantry_group.add(&unit_row);

    // Toggle qty/unit sensitivity based on in_pantry switch
    {
        let qty_clone = qty_row.clone();
        let unit_clone = unit_row.clone();
        in_pantry_row.connect_active_notify(move |row| {
            qty_clone.set_sensitive(row.is_active());
            unit_clone.set_sensitive(row.is_active());
        });
    }

    page.add(&pantry_group);
    form.append(&page);
    scroll.set_child(Some(&form));

    // ── Buttons ───────────────────────────────────────────────────────────────
    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_margin_top(8);
    btn_box.set_margin_bottom(16);
    btn_box.set_margin_start(16);
    btn_box.set_margin_end(16);
    btn_box.set_halign(gtk::Align::End);

    let cancel_btn = gtk::Button::with_label(s.cancel);
    cancel_btn.add_css_class("pill");

    let save_btn = gtk::Button::with_label(if is_edit { s.save } else { s.add });
    save_btn.add_css_class("suggested-action");
    save_btn.add_css_class("pill");

    btn_box.append(&cancel_btn);
    btn_box.append(&save_btn);

    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    outer.append(&scroll);
    outer.append(&btn_box);
    toolbar_view.set_content(Some(&outer));
    window.set_content(Some(&toolbar_view));

    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| win_cancel.close());

    let win_save = window.clone();
    let original_name = existing.map(|i| i.name.clone());
    save_btn.connect_clicked(move |_| {
        let name = name_row.text().to_string().trim().to_string();
        if name.is_empty() {
            return;
        }
        let category = cat_row.text().to_string().trim().to_string();
        let tags_raw = tags_row.text().to_string();
        let tags: Option<Vec<String>> = if tags_raw.is_empty() {
            None
        } else {
            Some(
                tags_raw
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        };

        let plural_text = plural_row.text().to_string();
        let plural = if plural_text.trim().is_empty() {
            None
        } else {
            Some(plural_text.trim().to_string())
        };

        let ingredient = Ingredient {
            name: name.clone(),
            slug: slugify(&name),
            file_stem: String::new(),
            category,
            tags,
            plural,
        };

        let in_pantry = in_pantry_row.is_active();
        let qty = qty_row.text().parse::<f64>().ok();
        let qty_type = unit_row.text().to_string();

        sender.input(AppMsg::SaveIngredient {
            original: original_name.clone(),
            ingredient,
            in_pantry,
            qty,
            qty_type,
        });
        win_save.close();
    });

    window.present();
}

/// Show a simple error in a message dialog.
pub fn show_error_toast(parent: &impl gtk::prelude::IsA<gtk::Window>, message: &str) {
    use adw::prelude::*;
    let s = i18n::strings();
    let dialog = adw::MessageDialog::new(Some(parent), Some("Error"), Some(message));
    dialog.add_response("ok", s.ok);
    dialog.set_default_response(Some("ok"));
    dialog.present();
}
