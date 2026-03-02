/// Lightweight translation layer for the Linux app.
///
/// Translations are stored as static `Strings` structs — one per language.
/// A global `RwLock<Language>` holds the active language so the UI can
/// change it at runtime without a restart.
use std::sync::RwLock;

// ── Language enum ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// Follow the system locale (detected from LANGUAGE / LANG env vars).
    System,
    English,
    Danish,
}

impl Language {
    /// Parse a settings tag (`"system"`, `"en"`, `"da"`).
    pub fn from_tag(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "en" => Language::English,
            "da" => Language::Danish,
            _ => Language::System,
        }
    }

    pub fn to_tag(self) -> &'static str {
        match self {
            Language::System => "system",
            Language::English => "en",
            Language::Danish => "da",
        }
    }

    /// Resolve `System` to a concrete language by inspecting the environment.
    fn resolve(self) -> Language {
        if self != Language::System {
            return self;
        }
        detect_from_env()
    }
}

fn detect_from_env() -> Language {
    for var in &["LANGUAGE", "LANG", "LC_ALL", "LC_MESSAGES"] {
        if let Ok(val) = std::env::var(var) {
            let v = val.to_lowercase();
            if v.starts_with("da") {
                return Language::Danish;
            }
            if v.starts_with("en") {
                return Language::English;
            }
        }
    }
    Language::English
}

// ── Global active language ────────────────────────────────────────────────────

static ACTIVE: RwLock<Language> = RwLock::new(Language::System);

pub fn set_language(lang: Language) {
    if let Ok(mut w) = ACTIVE.write() {
        *w = lang;
    }
}

/// Returns the currently active concrete language (System is resolved).
pub fn active() -> Language {
    ACTIVE.read().map(|l| *l).unwrap_or(Language::English).resolve()
}

// ── Strings struct ────────────────────────────────────────────────────────────

pub struct Strings {
    // Navigation
    pub nav_recipes: &'static str,
    pub nav_pantry: &'static str,
    pub nav_settings: &'static str,

    // Common actions
    pub edit: &'static str,
    pub delete: &'static str,
    pub cancel: &'static str,
    pub save: &'static str,
    pub add: &'static str,
    pub ok: &'static str,
    pub browse: &'static str,

    // Recipes tab
    pub search_recipes: &'static str,
    pub add_recipe: &'static str,
    pub no_recipes_found: &'static str,
    pub recipe_placeholder_title: &'static str,
    pub recipe_placeholder_desc: &'static str,
    pub all_required_available: &'static str,
    pub ingredients_heading: &'static str,
    pub instructions_heading: &'static str,
    pub optional_suffix: &'static str,
    pub no_data_dir: &'static str,

    // Recipe dialog
    pub add_recipe_dialog_title: &'static str,
    pub edit_recipe_dialog_title: &'static str,
    pub details_group: &'static str,
    pub recipe_title_field: &'static str,
    pub prep_time_field: &'static str,
    pub downtime_field: &'static str,
    pub servings_field: &'static str,
    pub tags_field: &'static str,
    pub ingredients_group: &'static str,
    pub qty_placeholder: &'static str,
    pub ingredient_placeholder: &'static str,
    pub note_placeholder: &'static str,
    pub optional_check_label: &'static str,
    pub optional_check_tooltip: &'static str,
    pub add_ingredient_row_btn: &'static str,
    pub instructions_group: &'static str,
    pub delete_recipe_body: &'static str,

    // Pantry tab
    pub search_pantry: &'static str,
    pub add_ingredient_btn: &'static str,
    pub no_ingredients_found: &'static str,
    pub uncategorised: &'static str,
    pub in_stock_only_label: &'static str,
    pub used_in_recipes: &'static str,
    pub pantry_heading: &'static str,
    pub in_stock_status: &'static str,
    pub not_in_stock_status: &'static str,
    pub ingredient_placeholder_title: &'static str,
    pub ingredient_placeholder_desc: &'static str,
    pub delete_ingredient_body: &'static str,

    // Ingredient dialog
    pub add_ingredient_dialog_title: &'static str,
    pub edit_ingredient_dialog_title: &'static str,
    pub ingredient_name_field: &'static str,
    pub ingredient_plural_field: &'static str,
    pub ingredient_category_field: &'static str,
    pub ingredient_tags_field: &'static str,
    pub pantry_group: &'static str,
    pub in_pantry_field: &'static str,
    pub quantity_field: &'static str,
    pub unit_field: &'static str,

    // Settings
    pub settings_group_data: &'static str,
    pub settings_data_desc: &'static str,
    pub settings_data_dir: &'static str,
    pub settings_group_appearance: &'static str,
    pub settings_theme: &'static str,
    pub theme_system: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    pub settings_group_language: &'static str,
    pub settings_language: &'static str,
    pub lang_system: &'static str,
    pub lang_en: &'static str,
    pub lang_da: &'static str,
}

// ── English ───────────────────────────────────────────────────────────────────

pub static EN: Strings = Strings {
    nav_recipes: "Recipes",
    nav_pantry: "Pantry",
    nav_settings: "Settings",

    edit: "Edit",
    delete: "Delete",
    cancel: "Cancel",
    save: "Save",
    add: "Add",
    ok: "OK",
    browse: "Browse…",

    search_recipes: "Search recipes…",
    add_recipe: "Add Recipe",
    no_recipes_found: "No recipes found",
    recipe_placeholder_title: "Recipes",
    recipe_placeholder_desc:
        "Select a recipe to view it, or add a new one.\n\
         Pie chart = proportion of required ingredients in pantry",
    all_required_available: "✓ All required ingredients available — ready to cook!",
    ingredients_heading: "Ingredients",
    instructions_heading: "Instructions",
    optional_suffix: " (optional)",
    no_data_dir: "No data directory set",

    add_recipe_dialog_title: "Add Recipe",
    edit_recipe_dialog_title: "Edit Recipe",
    details_group: "Details",
    recipe_title_field: "Title",
    prep_time_field: "Prep time (minutes)",
    downtime_field: "Oven / resting time (minutes)",
    servings_field: "Servings",
    tags_field: "Tags (comma-separated)",
    ingredients_group: "Ingredients",
    qty_placeholder: "qty",
    ingredient_placeholder: "ingredient",
    note_placeholder: "note",
    optional_check_label: "opt.",
    optional_check_tooltip: "Optional ingredient",
    add_ingredient_row_btn: "+ Add ingredient",
    instructions_group: "Instructions",
    delete_recipe_body: "This recipe will be permanently removed. This cannot be undone.",

    search_pantry: "Search ingredients…",
    add_ingredient_btn: "Add Ingredient",
    no_ingredients_found: "No ingredients found",
    uncategorised: "Uncategorised",
    in_stock_only_label: "In stock only",
    used_in_recipes: "Used in recipes",
    pantry_heading: "Pantry",
    in_stock_status: "✓ In stock",
    not_in_stock_status: "✗ Not in stock",
    ingredient_placeholder_title: "Pantry",
    ingredient_placeholder_desc: "Select an ingredient to view details, or add a new one.",
    delete_ingredient_body:
        "This will remove the ingredient and its pantry entry. This cannot be undone.",

    add_ingredient_dialog_title: "Add Ingredient",
    edit_ingredient_dialog_title: "Edit Ingredient",
    ingredient_name_field: "Name (singular)",
    ingredient_plural_field: "Plural (optional)",
    ingredient_category_field: "Category",
    ingredient_tags_field: "Tags (comma-separated)",
    pantry_group: "Pantry",
    in_pantry_field: "In pantry",
    quantity_field: "Quantity",
    unit_field: "Unit",

    settings_group_data: "Data",
    settings_data_desc: "Location of your recipes, ingredients and pantry files.",
    settings_data_dir: "Data Directory",
    settings_group_appearance: "Appearance",
    settings_theme: "Theme",
    theme_system: "System Default",
    theme_light: "Light",
    theme_dark: "Dark",
    settings_group_language: "Language",
    settings_language: "Language",
    lang_system: "System Default",
    lang_en: "English",
    lang_da: "Danish",
};

// ── Danish ────────────────────────────────────────────────────────────────────

pub static DA: Strings = Strings {
    nav_recipes: "Opskrifter",
    nav_pantry: "Spisekammer",
    nav_settings: "Indstillinger",

    edit: "Rediger",
    delete: "Slet",
    cancel: "Annuller",
    save: "Gem",
    add: "Tilføj",
    ok: "OK",
    browse: "Gennemse…",

    search_recipes: "Søg i opskrifter…",
    add_recipe: "Tilføj opskrift",
    no_recipes_found: "Ingen opskrifter fundet",
    recipe_placeholder_title: "Opskrifter",
    recipe_placeholder_desc:
        "Vælg en opskrift for at se den, eller tilføj en ny.\n\
         Lagkagediagram = andel af nødvendige ingredienser på lager",
    all_required_available: "✓ Alle nødvendige ingredienser er tilgængelige — klar til madlavning!",
    ingredients_heading: "Ingredienser",
    instructions_heading: "Fremgangsmåde",
    optional_suffix: " (valgfri)",
    no_data_dir: "Ingen datamappe valgt",

    add_recipe_dialog_title: "Tilføj opskrift",
    edit_recipe_dialog_title: "Rediger opskrift",
    details_group: "Detaljer",
    recipe_title_field: "Titel",
    prep_time_field: "Forberedelstid (minutter)",
    downtime_field: "Ovn-/hviletid (minutter)",
    servings_field: "Portioner",
    tags_field: "Tags (kommasepareret)",
    ingredients_group: "Ingredienser",
    qty_placeholder: "antal",
    ingredient_placeholder: "ingrediens",
    note_placeholder: "note",
    optional_check_label: "valgf.",
    optional_check_tooltip: "Valgfri ingrediens",
    add_ingredient_row_btn: "+ Tilføj ingrediens",
    instructions_group: "Fremgangsmåde",
    delete_recipe_body: "Denne opskrift fjernes permanent. Dette kan ikke fortrydes.",

    search_pantry: "Søg i ingredienser…",
    add_ingredient_btn: "Tilføj ingrediens",
    no_ingredients_found: "Ingen ingredienser fundet",
    uncategorised: "Ukategoriseret",
    in_stock_only_label: "Kun på lager",
    used_in_recipes: "Bruges i opskrifter",
    pantry_heading: "Spisekammer",
    in_stock_status: "✓ På lager",
    not_in_stock_status: "✗ Ikke på lager",
    ingredient_placeholder_title: "Spisekammer",
    ingredient_placeholder_desc: "Vælg en ingrediens for at se detaljer, eller tilføj en ny.",
    delete_ingredient_body:
        "Dette fjerner ingrediensen og dens opbevaring permanent. Dette kan ikke fortrydes.",

    add_ingredient_dialog_title: "Tilføj ingrediens",
    edit_ingredient_dialog_title: "Rediger ingrediens",
    ingredient_name_field: "Navn (ental)",
    ingredient_plural_field: "Flertal (valgfri)",
    ingredient_category_field: "Kategori",
    ingredient_tags_field: "Tags (kommasepareret)",
    pantry_group: "Spisekammer",
    in_pantry_field: "På lager",
    quantity_field: "Mængde",
    unit_field: "Enhed",

    settings_group_data: "Data",
    settings_data_desc: "Placering af dine opskrifter, ingredienser og spisekammerfiler.",
    settings_data_dir: "Datamappe",
    settings_group_appearance: "Udseende",
    settings_theme: "Tema",
    theme_system: "Systemstandard",
    theme_light: "Lys",
    theme_dark: "Mørk",
    settings_group_language: "Sprog",
    settings_language: "Sprog",
    lang_system: "Systemstandard",
    lang_en: "Engelsk",
    lang_da: "Dansk",
};

// ── Accessor ──────────────────────────────────────────────────────────────────

/// Returns the `Strings` for the currently active language.
pub fn strings() -> &'static Strings {
    match active() {
        Language::Danish => &DA,
        _ => &EN,
    }
}

// ── Parameterised strings ─────────────────────────────────────────────────────

pub fn fmt_missing_required(n: usize) -> String {
    match active() {
        Language::Danish => format!("⚠ Mangler {} nødvendig(e) ingrediens(er)", n),
        _ => format!("⚠ Missing {} required ingredient(s)", n),
    }
}

pub fn fmt_required_tooltip(in_stock: usize, total: usize) -> String {
    match active() {
        Language::Danish => format!("{}/{} nødvendige ingredienser på lager", in_stock, total),
        _ => format!("{}/{} required ingredients in pantry", in_stock, total),
    }
}

pub fn fmt_prep_time(mins: u32) -> String {
    match active() {
        Language::Danish => format!("⏱ {} min forberedelse", mins),
        _ => format!("⏱ {} min prep", mins),
    }
}

pub fn fmt_cook_time(mins: u32) -> String {
    format!("🔥 {} min", mins)
}

pub fn fmt_servings(n: u32) -> String {
    match active() {
        Language::Danish => format!("👤 {} portioner", n),
        _ => format!("👤 {} servings", n),
    }
}

pub fn fmt_tags(tags: &str) -> String {
    match active() {
        Language::Danish => format!("Tags: {}", tags),
        _ => format!("Tags: {}", tags),
    }
}

pub fn fmt_category(cat: &str) -> String {
    match active() {
        Language::Danish => format!("Kategori: {}", cat),
        _ => format!("Category: {}", cat),
    }
}

pub fn fmt_quantity(qty: f64, unit: &str) -> String {
    match active() {
        Language::Danish => {
            if unit.is_empty() {
                format!("Mængde: {}", qty)
            } else {
                format!("Mængde: {} {}", qty, unit)
            }
        }
        _ => {
            if unit.is_empty() {
                format!("Quantity: {}", qty)
            } else {
                format!("Quantity: {} {}", qty, unit)
            }
        }
    }
}

pub fn fmt_last_updated(date: &str) -> String {
    match active() {
        Language::Danish => format!("Sidst opdateret: {}", date),
        _ => format!("Last updated: {}", date),
    }
}

pub fn fmt_delete_recipe_title(title: &str) -> String {
    match active() {
        Language::Danish => format!("Slet \"{}\"?", title),
        _ => format!("Delete \"{}\"?", title),
    }
}

pub fn fmt_delete_ingredient_title(name: &str) -> String {
    match active() {
        Language::Danish => format!("Slet \"{}\"?", name),
        _ => format!("Delete \"{}\"?", name),
    }
}

pub fn fmt_category_hint(example: &str) -> String {
    match active() {
        Language::Danish => format!("Kategori (f.eks. {})", example),
        _ => format!("Category (e.g. {})", example),
    }
}

// ── Unit suggestions ──────────────────────────────────────────────────────────

/// Localised common unit names for the ingredient editor.
pub fn suggested_units() -> &'static [&'static str] {
    match active() {
        Language::Danish => &[
            "stk", "kg", "g", "l", "dl", "ml",
            "dåse", "pakke", "pose", "flaske", "karton",
            "tsk", "spsk", "nip",
        ],
        _ => &[
            "pcs", "kg", "g", "l", "dl", "ml",
            "can", "pack", "bag", "bottle", "carton",
            "tsp", "tbsp", "pinch",
        ],
    }
}
