/// Modal dialogs for creating and editing recipes and ingredients.
///
/// Each dialog is a `gtk::Window` shown as a transient child of the main window.
/// When the user confirms, the dialog sends a message via the component sender.
use janus_engine::{Ingredient, PantryItem, Recipe, RecipeIngredient};
use libadwaita as adw;
use relm4::gtk;
use relm4::ComponentSender;

use crate::app::{App, AppMsg};

// ─── Recipe dialog ────────────────────────────────────────────────────────────

/// Show a dialog for adding (original = None) or editing (original = Some) a recipe.
pub fn show_recipe_dialog(
    parent: &impl gtk::prelude::IsA<gtk::Window>,
    ingredient_names: Vec<String>,
    existing: Option<&Recipe>,
    sender: ComponentSender<App>,
) {
    use adw::prelude::*;

    let is_edit = existing.is_some();
    let window = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(if is_edit { "Edit Recipe" } else { "Add Recipe" })
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
    group.set_title("Details");

    // Title
    let title_row = adw::EntryRow::new();
    title_row.set_title("Title");
    if let Some(r) = existing {
        title_row.set_text(&r.title);
    }
    group.add(&title_row);

    // Prep time
    let prep_row = adw::EntryRow::new();
    prep_row.set_title("Prep time (minutes)");
    if let Some(r) = existing {
        if let Some(t) = r.prep_time {
            prep_row.set_text(&t.to_string());
        }
    }
    group.add(&prep_row);

    // Downtime (oven / resting)
    let down_row = adw::EntryRow::new();
    down_row.set_title("Oven / resting time (minutes)");
    if let Some(r) = existing {
        if let Some(t) = r.downtime {
            down_row.set_text(&t.to_string());
        }
    }
    group.add(&down_row);

    // Servings
    let servings_row = adw::EntryRow::new();
    servings_row.set_title("Servings");
    if let Some(r) = existing {
        if let Some(s) = r.servings {
            servings_row.set_text(&s.to_string());
        }
    }
    group.add(&servings_row);

    // Tags (comma-separated)
    let tags_row = adw::EntryRow::new();
    tags_row.set_title("Tags (comma-separated)");
    if let Some(r) = existing {
        if let Some(tags) = &r.tags {
            tags_row.set_text(&tags.join(", "));
        }
    }
    group.add(&tags_row);

    page.add(&group);

    // Ingredients group
    let ing_group = adw::PreferencesGroup::new();
    ing_group.set_title("Ingredients");
    ing_group.set_description(Some(
        "One per line: \"ingredient_name quantity unit\" (e.g. \"potato 2 kg\")",
    ));

    let ing_text = gtk::TextView::new();
    ing_text.set_monospace(true);
    ing_text.set_wrap_mode(gtk::WrapMode::Word);
    ing_text.set_top_margin(8);
    ing_text.set_bottom_margin(8);
    ing_text.set_left_margin(8);
    ing_text.set_right_margin(8);
    ing_text.add_css_class("card");
    ing_text.set_size_request(-1, 100);

    if let Some(r) = existing {
        let lines: Vec<String> = r
            .ingredients
            .iter()
            .map(|i| {
                match (i.quantity, &i.quantity_type) {
                    (Some(q), Some(u)) if !u.is_empty() => {
                        format!("{} {} {}", i.ingredient, q, u)
                    }
                    (Some(q), _) => format!("{} {}", i.ingredient, q),
                    _ => i.ingredient.clone(),
                }
            })
            .collect();
        ing_text.buffer().set_text(&lines.join("\n"));
    }

    let ing_scroll = gtk::ScrolledWindow::new();
    ing_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    ing_scroll.set_min_content_height(100);
    ing_scroll.set_child(Some(&ing_text));

    ing_group.add(&ing_scroll);
    page.add(&ing_group);

    // Instructions group
    let instr_group = adw::PreferencesGroup::new();
    instr_group.set_title("Instructions");

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

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.add_css_class("pill");

    let save_btn = gtk::Button::with_label(if is_edit { "Save" } else { "Add" });
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
    save_btn.connect_clicked(move |_| {
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

        // Parse ingredients from text view
        let (start, end) = ing_text.buffer().bounds();
        let ing_raw = ing_text.buffer().text(&start, &end, false).to_string();
        let ingredients: Vec<RecipeIngredient> = ing_raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                match parts.as_slice() {
                    [name] => RecipeIngredient {
                        ingredient: name.to_string(),
                        quantity: None,
                        quantity_type: None,
                    },
                    [name, qty] => RecipeIngredient {
                        ingredient: name.to_string(),
                        quantity: qty.parse::<f64>().ok(),
                        quantity_type: None,
                    },
                    [name, qty, unit, ..] => RecipeIngredient {
                        ingredient: name.to_string(),
                        quantity: qty.parse::<f64>().ok(),
                        quantity_type: Some(unit.to_string()),
                    },
                    _ => RecipeIngredient {
                        ingredient: line.trim().to_string(),
                        quantity: None,
                        quantity_type: None,
                    },
                }
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
    let _ = ingredient_names; // reserved for future autocomplete
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

    let is_edit = existing.is_some();
    let window = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(if is_edit { "Edit Ingredient" } else { "Add Ingredient" })
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
    details_group.set_title("Details");

    // Name
    let name_row = adw::EntryRow::new();
    name_row.set_title("Name (slug)");
    if let Some(ing) = existing {
        name_row.set_text(&ing.name);
    }
    details_group.add(&name_row);

    // Category (combo from existing categories + free entry)
    let cat_row = adw::EntryRow::new();
    cat_row.set_title("Category");
    if let Some(ing) = existing {
        cat_row.set_text(&ing.category);
    }
    // Update title to hint at existing categories
    if !categories.is_empty() {
        cat_row.set_title(&format!("Category (e.g. {})", categories.first().unwrap()));
    }
    details_group.add(&cat_row);

    // Tags (comma-separated)
    let tags_row = adw::EntryRow::new();
    tags_row.set_title("Tags (comma-separated)");
    if let Some(ing) = existing {
        if let Some(tags) = &ing.tags {
            tags_row.set_text(&tags.join(", "));
        }
    }
    details_group.add(&tags_row);

    page.add(&details_group);

    // Pantry group
    let pantry_group = adw::PreferencesGroup::new();
    pantry_group.set_title("Pantry");

    let in_pantry_row = adw::SwitchRow::new();
    in_pantry_row.set_title("In pantry");
    let in_pantry = pantry_item.is_some();
    in_pantry_row.set_active(in_pantry);
    pantry_group.add(&in_pantry_row);

    let qty_row = adw::EntryRow::new();
    qty_row.set_title("Quantity");
    qty_row.set_sensitive(in_pantry);
    if let Some(item) = pantry_item {
        if let Some(q) = item.quantity {
            qty_row.set_text(&q.to_string());
        }
    }
    pantry_group.add(&qty_row);

    let unit_row = adw::EntryRow::new();
    unit_row.set_title("Unit (e.g. kg, g, ml, pcs)");
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

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.add_css_class("pill");

    let save_btn = gtk::Button::with_label(if is_edit { "Save" } else { "Add" });
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

        let ingredient = Ingredient {
            name: name.clone(),
            slug: name.to_lowercase().replace(' ', "_"),
            category,
            kb: None,
            tags,
            translations: None,
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
    let dialog = adw::MessageDialog::new(Some(parent), Some("Error"), Some(message));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present();
}
