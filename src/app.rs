/// Main application component: AppModel (state), AppMsg (messages), AppWidgets (UI references).
use crate::config::{Theme, UserSettings};
use crate::ui_constants::*;
use janus_engine::{DataManager, Ingredient, Recipe};
use libadwaita as adw;
use relm4::gtk;
use relm4::{gtk::prelude::*, ComponentParts, ComponentSender, SimpleComponent};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

// ── Tab enum ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Recipes,
    Pantry,
    Kb,
    Settings,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum AppMsg {
    // Navigation
    SwitchTab(Tab),

    // Recipes
    SearchRecipes(String),
    SelectRecipe(Option<String>),
    AddRecipe,
    EditRecipe(String),
    DeleteRecipe(String),
    SaveRecipe { original: Option<String>, recipe: Recipe },

    // Pantry
    SearchIngredients(String),
    SelectIngredient(Option<String>),
    ToggleInStockOnly(bool),
    AddIngredient,
    EditIngredient(String),
    DeleteIngredient(String),
    SaveIngredient {
        original: Option<String>,
        ingredient: Ingredient,
        in_pantry: bool,
        qty: Option<f64>,
        qty_type: String,
    },

    // Knowledge Base
    SelectKb(Option<String>),

    // Settings
    SetDataDir(String),
    DataDirReady(String),
    SetTheme(String),

    // System
    ShowToast(String),
    ReloadAll,
}

// ── Application state ─────────────────────────────────────────────────────────

pub struct App {
    pub dm: Option<Rc<RefCell<DataManager>>>,
    pub data_dir: PathBuf,
    pub settings: Rc<RefCell<UserSettings>>,

    pub tab: Tab,

    // Recipes state
    pub recipe_search: String,
    pub selected_recipe: Option<String>,

    // Pantry state
    pub ingredient_search: String,
    pub selected_ingredient: Option<String>,
    pub category_filter: Vec<String>,
    pub in_stock_only: bool,

    // KB state
    pub selected_kb: Option<String>,

    // Dirty flags (Cell<bool> avoids &mut self in update_view)
    pub recipes_dirty: Cell<bool>,
    pub pantry_dirty: Cell<bool>,
    pub kb_dirty: Cell<bool>,
    pub recipe_detail_dirty: Cell<bool>,
    pub ingredient_detail_dirty: Cell<bool>,
    pub kb_detail_dirty: Cell<bool>,

    // Pending dialog requests (RefCell allows mutation from &self in update_view)
    pub pending_add_recipe: Cell<bool>,
    pub pending_edit_recipe: RefCell<Option<String>>,
    pub pending_add_ingredient: Cell<bool>,
    pub pending_edit_ingredient: RefCell<Option<String>>,

    // Channel for receiving a DataManager loaded on a background thread
    pub pending_dm: Option<mpsc::Receiver<Result<DataManager, String>>>,
}

// ── Widget references ─────────────────────────────────────────────────────────

pub struct AppWidgets {
    pub window: adw::ApplicationWindow,
    pub toast_overlay: adw::ToastOverlay,
    pub main_stack: gtk::Stack,
    pub nav_list: gtk::ListBox,

    // Recipes
    pub recipe_list: gtk::ListBox,
    pub recipe_detail: gtk::Box,

    // Pantry
    pub pantry_list: gtk::ListBox,
    pub ingredient_detail: gtk::Box,
    pub in_stock_switch: gtk::Switch,

    // KB
    pub kb_list: gtk::ListBox,
    pub kb_detail: gtk::Box,
}

// ── SimpleComponent impl ──────────────────────────────────────────────────────

impl SimpleComponent for App {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Root = adw::ApplicationWindow;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        adw::ApplicationWindow::builder()
            .title("Cookbook")
            .default_width(DEFAULT_WINDOW_WIDTH)
            .default_height(DEFAULT_WINDOW_HEIGHT)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        use adw::prelude::*;

        let settings = UserSettings::load();
        let data_dir = UserSettings::effective_data_dir();

        // Load DataManager on a background thread so the window appears immediately
        // even if the data directory is on a slow/network filesystem (e.g. pCloud FUSE).
        let (tx, rx) = mpsc::channel();
        let sender_startup = sender.clone();
        let data_dir_startup = data_dir.clone();
        std::thread::spawn(move || {
            let result = DataManager::new(&data_dir_startup).map_err(|e| e.to_string());
            let _ = tx.send(result);
            sender_startup.input(AppMsg::DataDirReady(
                data_dir_startup.display().to_string(),
            ));
        });

        let app_state = App {
            dm: None,
            data_dir: data_dir.clone(),
            settings: Rc::new(RefCell::new(settings.clone())),
            tab: Tab::Recipes,
            recipe_search: String::new(),
            selected_recipe: None,
            ingredient_search: String::new(),
            selected_ingredient: None,
            category_filter: Vec::new(),
            in_stock_only: false,
            selected_kb: None,
            recipes_dirty: Cell::new(true),
            pantry_dirty: Cell::new(true),
            kb_dirty: Cell::new(true),
            recipe_detail_dirty: Cell::new(false),
            ingredient_detail_dirty: Cell::new(false),
            kb_detail_dirty: Cell::new(false),
            pending_add_recipe: Cell::new(false),
            pending_edit_recipe: RefCell::new(None),
            pending_add_ingredient: Cell::new(false),
            pending_edit_ingredient: RefCell::new(None),
            pending_dm: Some(rx),
        };

        // ── Apply initial theme ───────────────────────────────────────────────
        apply_theme(&settings.theme);

        // ── Build main layout ─────────────────────────────────────────────────
        let toast_overlay = adw::ToastOverlay::new();

        let toolbar_view = adw::ToolbarView::new();

        // Header bar
        let header = adw::HeaderBar::new();
        let win_title = adw::WindowTitle::new("Cookbook", "");
        header.set_title_widget(Some(&win_title));
        toolbar_view.add_top_bar(&header);

        // Content: sidebar + stack
        let content_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // ── Sidebar ───────────────────────────────────────────────────────────
        let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sidebar.set_width_request(SIDEBAR_WIDTH);

        let nav_list = gtk::ListBox::new();
        nav_list.set_selection_mode(gtk::SelectionMode::Single);
        nav_list.add_css_class("navigation-sidebar");
        nav_list.set_vexpand(true);

        for (icon, label, tab_name) in &[
            ("emblem-documents-symbolic", "Recipes", "recipes"),
            ("view-list-symbolic", "Pantry", "pantry"),
            ("system-help-symbolic", "Knowledge Base", "kb"),
            ("preferences-system-symbolic", "Settings", "settings"),
        ] {
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(tab_name);
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            let icon_img = gtk::Image::from_icon_name(icon);
            let lbl = gtk::Label::new(Some(label));
            lbl.set_halign(gtk::Align::Start);
            lbl.set_hexpand(true);
            row_box.append(&icon_img);
            row_box.append(&lbl);
            row.set_child(Some(&row_box));
            nav_list.append(&row);
        }

        // Select first row (Recipes)
        if let Some(first_row) = nav_list.row_at_index(0) {
            nav_list.select_row(Some(&first_row));
        }

        {
            let sender_nav = sender.clone();
            nav_list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    let name = row.widget_name().to_string();
                    let tab = match name.as_str() {
                        "recipes" => Tab::Recipes,
                        "pantry" => Tab::Pantry,
                        "kb" => Tab::Kb,
                        "settings" => Tab::Settings,
                        _ => Tab::Recipes,
                    };
                    sender_nav.input(AppMsg::SwitchTab(tab));
                }
            });
        }

        sidebar.append(&nav_list);

        // ── Main stack ────────────────────────────────────────────────────────
        let main_stack = gtk::Stack::new();
        main_stack.set_hexpand(true);
        main_stack.set_vexpand(true);
        main_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        main_stack.set_transition_duration(150);

        // Recipes tab
        let (recipes_widget, recipe_list, recipe_detail) =
            crate::recipes::build_recipes_tab(&None, sender.clone());
        main_stack.add_named(&recipes_widget, Some("recipes"));

        // Pantry tab
        let (pantry_widget, pantry_list, ingredient_detail, in_stock_switch) =
            crate::pantry::build_pantry_tab(&None, false, sender.clone());
        main_stack.add_named(&pantry_widget, Some("pantry"));

        // KB tab
        let (kb_widget, kb_list, kb_detail) =
            crate::kb::build_kb_tab(&None, sender.clone());
        main_stack.add_named(&kb_widget, Some("kb"));

        // Settings tab
        let settings_widget = crate::settings::build_settings_page(&sender);
        main_stack.add_named(&settings_widget, Some("settings"));

        main_stack.set_visible_child_name("recipes");

        content_box.append(&sidebar);
        content_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        content_box.append(&main_stack);

        toolbar_view.set_content(Some(&content_box));
        toast_overlay.set_child(Some(&toolbar_view));
        root.set_content(Some(&toast_overlay));

        let widgets = AppWidgets {
            window: root,
            toast_overlay,
            main_stack,
            nav_list,
            recipe_list,
            recipe_detail,
            pantry_list,
            ingredient_detail,
            in_stock_switch,
            kb_list,
            kb_detail,
        };

        ComponentParts {
            model: app_state,
            widgets,
        }
    }

    // ── Update (message handling) ─────────────────────────────────────────────

    fn update(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::SwitchTab(tab) => {
                self.tab = tab;
            }

            // ── Recipes ───────────────────────────────────────────────────────
            AppMsg::SearchRecipes(q) => {
                self.recipe_search = q;
                self.recipes_dirty.set(true);
            }
            AppMsg::SelectRecipe(title) => {
                self.selected_recipe = title;
                self.recipe_detail_dirty.set(true);
            }
            AppMsg::AddRecipe => {
                self.pending_add_recipe.set(true);
            }
            AppMsg::EditRecipe(title) => {
                *self.pending_edit_recipe.borrow_mut() = Some(title);
            }
            AppMsg::DeleteRecipe(title) => {
                if let Some(dm) = &self.dm {
                    match dm.borrow_mut().delete_recipe(&title) {
                        Ok(_) => {
                            if self.selected_recipe.as_deref() == Some(&title) {
                                self.selected_recipe = None;
                            }
                            self.recipes_dirty.set(true);
                            self.recipe_detail_dirty.set(true);
                        }
                        Err(e) => {
                            sender.input(AppMsg::ShowToast(format!("Error: {}", e)));
                        }
                    }
                }
            }
            AppMsg::SaveRecipe { original, recipe } => {
                if let Some(dm) = &self.dm {
                    let result = match original {
                        Some(ref orig) => dm.borrow_mut().update_recipe(orig, recipe.clone()),
                        None => dm
                            .borrow_mut()
                            .create_recipe(recipe.clone())
                            .map(|_| true),
                    };
                    match result {
                        Ok(_) => {
                            self.selected_recipe = Some(recipe.title.clone());
                            self.recipes_dirty.set(true);
                            self.recipe_detail_dirty.set(true);
                        }
                        Err(e) => {
                            sender.input(AppMsg::ShowToast(format!("Error saving recipe: {}", e)));
                        }
                    }
                }
            }

            // ── Pantry ────────────────────────────────────────────────────────
            AppMsg::SearchIngredients(q) => {
                self.ingredient_search = q;
                self.pantry_dirty.set(true);
            }
            AppMsg::SelectIngredient(name) => {
                self.selected_ingredient = name;
                self.ingredient_detail_dirty.set(true);
            }
            AppMsg::ToggleInStockOnly(val) => {
                self.in_stock_only = val;
                self.pantry_dirty.set(true);
            }
            AppMsg::AddIngredient => {
                self.pending_add_ingredient.set(true);
            }
            AppMsg::EditIngredient(name) => {
                *self.pending_edit_ingredient.borrow_mut() = Some(name);
            }
            AppMsg::DeleteIngredient(name) => {
                if let Some(dm) = &self.dm {
                    match dm.borrow_mut().delete_ingredient(&name) {
                        Ok(_) => {
                            if self.selected_ingredient.as_deref() == Some(&name) {
                                self.selected_ingredient = None;
                            }
                            self.pantry_dirty.set(true);
                            self.ingredient_detail_dirty.set(true);
                            self.recipes_dirty.set(true); // availability may have changed
                        }
                        Err(e) => {
                            sender.input(AppMsg::ShowToast(format!("Error: {}", e)));
                        }
                    }
                }
            }
            AppMsg::SaveIngredient {
                original,
                ingredient,
                in_pantry,
                qty,
                qty_type,
            } => {
                if let Some(dm) = &self.dm {
                    let result = if let Some(ref orig) = original {
                        dm.borrow_mut().update_ingredient_with_pantry(
                            orig,
                            ingredient.clone(),
                            if in_pantry { qty } else { None },
                            if in_pantry { Some(qty_type) } else { None },
                            !in_pantry,
                        )
                    } else {
                        // Create new ingredient first
                        let create_result =
                            dm.borrow_mut().create_ingredient(ingredient.clone());
                        if let Err(e) = create_result {
                            sender.input(AppMsg::ShowToast(format!("Error: {}", e)));
                            return;
                        }
                        if in_pantry {
                            dm.borrow_mut().update_pantry_item(
                                &ingredient.name,
                                qty,
                                if qty_type.is_empty() { None } else { Some(qty_type) },
                            )
                        } else {
                            Ok(true)
                        }
                    };
                    match result {
                        Ok(_) => {
                            self.selected_ingredient = Some(ingredient.name.clone());
                            self.pantry_dirty.set(true);
                            self.ingredient_detail_dirty.set(true);
                            self.recipes_dirty.set(true);
                        }
                        Err(e) => {
                            sender.input(AppMsg::ShowToast(format!("Error: {}", e)));
                        }
                    }
                }
            }

            // ── KB ────────────────────────────────────────────────────────────
            AppMsg::SelectKb(slug) => {
                self.selected_kb = slug;
                self.kb_detail_dirty.set(true);
            }

            // ── Settings ──────────────────────────────────────────────────────
            AppMsg::SetDataDir(dir) => {
                let path = PathBuf::from(&dir);
                self.data_dir = path.clone();
                {
                    let mut s = self.settings.borrow_mut();
                    s.data_dir = Some(dir.clone());
                    s.save();
                }
                // Load DataManager on a background thread to avoid blocking the UI
                // (pCloud FUSE can take time for network reads).
                let (tx, rx) = mpsc::channel();
                self.pending_dm = Some(rx);
                let sender_clone = sender.clone();
                std::thread::spawn(move || {
                    let result = DataManager::new(&path).map_err(|e| e.to_string());
                    let _ = tx.send(result);
                    sender_clone.input(AppMsg::DataDirReady(dir));
                });
            }
            AppMsg::DataDirReady(dir) => {
                match self.pending_dm.take().and_then(|rx| rx.recv().ok()) {
                    Some(Ok(new_dm)) => {
                        self.dm = Some(Rc::new(RefCell::new(new_dm)));
                    }
                    Some(Err(e)) => {
                        sender.input(AppMsg::ShowToast(format!(
                            "Could not load data from {dir}: {e}"
                        )));
                    }
                    None => {}
                }
                self.recipes_dirty.set(true);
                self.pantry_dirty.set(true);
                self.kb_dirty.set(true);
                self.selected_recipe = None;
                self.selected_ingredient = None;
                self.selected_kb = None;
            }
            AppMsg::SetTheme(theme_str) => {
                let theme = match theme_str.as_str() {
                    "Light" => Theme::Light,
                    "Dark" => Theme::Dark,
                    _ => Theme::System,
                };
                apply_theme(&theme);
                let mut s = self.settings.borrow_mut();
                s.theme = theme;
                s.save();
            }

            // ── System ────────────────────────────────────────────────────────
            AppMsg::ShowToast(msg) => {
                log::info!("Toast: {}", msg);
                // Actual toast shown in update_view
            }
            AppMsg::ReloadAll => {
                if let Ok(new_dm) = DataManager::new(&self.data_dir) {
                    self.dm = Some(Rc::new(RefCell::new(new_dm)));
                }
                self.recipes_dirty.set(true);
                self.pantry_dirty.set(true);
                self.kb_dirty.set(true);
            }
        }
    }

    // ── Update view (UI sync from model state) ────────────────────────────────

    fn update_view(&self, widgets: &mut AppWidgets, sender: ComponentSender<Self>) {
        // Switch the visible tab
        let tab_name = match self.tab {
            Tab::Recipes => "recipes",
            Tab::Pantry => "pantry",
            Tab::Kb => "kb",
            Tab::Settings => "settings",
        };
        widgets.main_stack.set_visible_child_name(tab_name);

        // Rebuild recipe list if dirty
        if self.recipes_dirty.get() {
            crate::recipes::populate_recipe_list(
                &widgets.recipe_list,
                &self.dm,
                &self.recipe_search,
                &sender,
            );
            self.recipes_dirty.set(false);
            // Re-select if there is a selection
            if let Some(ref title) = self.selected_recipe {
                select_row_by_name(&widgets.recipe_list, title);
            }
        }

        // Update recipe detail if dirty or tab changed
        if self.recipe_detail_dirty.get() || self.tab == Tab::Recipes {
            if let Some(ref title) = self.selected_recipe {
                crate::recipes::update_recipe_detail(
                    &widgets.recipe_detail,
                    &self.dm,
                    title,
                    &sender,
                );
            } else {
                crate::recipes::show_recipe_placeholder(&widgets.recipe_detail);
            }
            self.recipe_detail_dirty.set(false);
        }

        // Rebuild pantry list if dirty
        if self.pantry_dirty.get() {
            crate::pantry::populate_pantry_list(
                &widgets.pantry_list,
                &self.dm,
                &self.ingredient_search,
                &self.category_filter,
                self.in_stock_only,
                &sender,
            );
            // Sync in-stock switch
            if widgets.in_stock_switch.is_active() != self.in_stock_only {
                widgets.in_stock_switch.set_active(self.in_stock_only);
            }
            self.pantry_dirty.set(false);
            if let Some(ref name) = self.selected_ingredient {
                select_row_by_name(&widgets.pantry_list, name);
            }
        }

        // Update ingredient detail if dirty or tab changed
        if self.ingredient_detail_dirty.get() || self.tab == Tab::Pantry {
            if let Some(ref name) = self.selected_ingredient {
                crate::pantry::update_ingredient_detail(
                    &widgets.ingredient_detail,
                    &self.dm,
                    name,
                    &sender,
                );
            } else {
                crate::pantry::show_ingredient_placeholder(&widgets.ingredient_detail);
            }
            self.ingredient_detail_dirty.set(false);
        }

        // KB list rebuild
        if self.kb_dirty.get() {
            crate::kb::populate_kb_list(&widgets.kb_list, &self.dm, &sender);
            self.kb_dirty.set(false);
        }

        // KB detail
        if self.kb_detail_dirty.get() || self.tab == Tab::Kb {
            if let Some(ref slug) = self.selected_kb {
                crate::kb::update_kb_detail(&widgets.kb_detail, &self.dm, slug);
            } else {
                crate::kb::show_kb_placeholder(&widgets.kb_detail);
            }
            self.kb_detail_dirty.set(false);
        }

        // ── Open pending dialogs (need widget references for parent window) ───
        if self.pending_add_recipe.get() {
            self.pending_add_recipe.set(false);
            open_add_recipe_dialog(&widgets.window, &self.dm, sender.clone());
        }
        if let Some(title) = self.pending_edit_recipe.borrow_mut().take() {
            open_edit_recipe_dialog(&widgets.window, &self.dm, &title, sender.clone());
        }
        if self.pending_add_ingredient.get() {
            self.pending_add_ingredient.set(false);
            open_add_ingredient_dialog(&widgets.window, &self.dm, sender.clone());
        }
        if let Some(name) = self.pending_edit_ingredient.borrow_mut().take() {
            open_edit_ingredient_dialog(&widgets.window, &self.dm, &name, sender.clone());
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Select the list box row whose widget name matches `name`.
fn select_row_by_name(list: &gtk::ListBox, name: &str) {
    let mut i = 0;
    while let Some(row) = list.row_at_index(i) {
        if row.widget_name().as_str() == name {
            list.select_row(Some(&row));
            return;
        }
        i += 1;
    }
}

/// Apply the chosen theme via the libadwaita style manager.
pub fn apply_theme(theme: &Theme) {
    let mgr = adw::StyleManager::default();
    let scheme = match theme {
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
        Theme::System => adw::ColorScheme::Default,
    };
    mgr.set_color_scheme(scheme);
}

// ── Dialog helpers called from the App ───────────────────────────────────────
// (These live here rather than in update() because they need widget references
// to find the parent window.)

pub fn open_add_recipe_dialog(window: &adw::ApplicationWindow, dm: &Option<Rc<RefCell<DataManager>>>, sender: ComponentSender<App>) {
    let names = dm
        .as_ref()
        .map(|d| {
            d.borrow()
                .get_all_ingredients()
                .into_iter()
                .map(|i| i.name.clone())
                .collect()
        })
        .unwrap_or_default();
    crate::dialogs::show_recipe_dialog(window, names, None, sender);
}

pub fn open_edit_recipe_dialog(
    window: &adw::ApplicationWindow,
    dm: &Option<Rc<RefCell<DataManager>>>,
    title: &str,
    sender: ComponentSender<App>,
) {
    let dm_borrow;
    let recipe = if let Some(d) = dm {
        dm_borrow = d.borrow();
        dm_borrow.get_recipe(title).cloned()
    } else {
        None
    };
    if let Some(ref recipe) = recipe {
        let names = dm
            .as_ref()
            .map(|d| {
                d.borrow()
                    .get_all_ingredients()
                    .into_iter()
                    .map(|i| i.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        crate::dialogs::show_recipe_dialog(window, names, Some(recipe), sender);
    }
}

pub fn open_add_ingredient_dialog(
    window: &adw::ApplicationWindow,
    dm: &Option<Rc<RefCell<DataManager>>>,
    sender: ComponentSender<App>,
) {
    let categories = dm
        .as_ref()
        .map(|d| d.borrow().get_all_ingredient_categories())
        .unwrap_or_default();
    crate::dialogs::show_ingredient_dialog(window, categories, None, None, sender);
}

pub fn open_edit_ingredient_dialog(
    window: &adw::ApplicationWindow,
    dm: &Option<Rc<RefCell<DataManager>>>,
    name: &str,
    sender: ComponentSender<App>,
) {
    let dm_borrow;
    let (ingredient, pantry_item) = if let Some(d) = dm {
        dm_borrow = d.borrow();
        let ing = dm_borrow.get_ingredient(name).cloned();
        let pantry = dm_borrow.get_pantry_item(name).cloned();
        (ing, pantry)
    } else {
        (None, None)
    };
    if let Some(ref ingredient) = ingredient {
        let categories = dm
            .as_ref()
            .map(|d| d.borrow().get_all_ingredient_categories())
            .unwrap_or_default();
        crate::dialogs::show_ingredient_dialog(
            window,
            categories,
            Some(ingredient),
            pantry_item.as_ref(),
            sender,
        );
    }
}
