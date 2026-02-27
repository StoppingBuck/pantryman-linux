use crate::app::AppMsg;
use crate::config::{Theme, UserSettings};
use libadwaita as adw;
use relm4::gtk;
use relm4::ComponentSender;

/// Build and return the settings page widget.
pub fn build_settings_page(sender: &ComponentSender<crate::app::App>) -> gtk::Widget {
    use adw::prelude::*;

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);

    let page = adw::PreferencesPage::new();

    // ── Data group ────────────────────────────────────────────────────────────
    let data_group = adw::PreferencesGroup::new();
    data_group.set_title("Data");
    data_group.set_description(Some(
        "Location of your recipes, ingredients and pantry files.",
    ));

    let current_dir = UserSettings::effective_data_dir();
    let dir_row = adw::ActionRow::new();
    dir_row.set_title("Data Directory");
    dir_row.set_subtitle(&current_dir.display().to_string());
    dir_row.set_subtitle_lines(1);

    let browse_btn = gtk::Button::with_label("Browse…");
    browse_btn.set_valign(gtk::Align::Center);
    browse_btn.add_css_class("flat");

    {
        let dir_row_clone = dir_row.clone();
        let sender_clone = sender.clone();
        browse_btn.connect_clicked(move |btn| {
            let root = btn.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            let dialog = gtk::FileChooserNative::new(
                Some("Choose Data Directory"),
                root.as_ref(),
                gtk::FileChooserAction::SelectFolder,
                Some("Select"),
                Some("Cancel"),
            );
            let sender_inner = sender_clone.clone();
            let dir_row_inner = dir_row_clone.clone();
            dialog.connect_response(move |d, response| {
                if gtk::ResponseType::from(response) == gtk::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.display().to_string();
                            dir_row_inner.set_subtitle(&path_str);
                            crate::utils::validate_and_create_data_dir(&path);
                            sender_inner.input(AppMsg::SetDataDir(path_str));
                        }
                    }
                }
            });
            dialog.show();
        });
    }

    dir_row.add_suffix(&browse_btn);
    dir_row.set_activatable_widget(Some(&browse_btn));
    data_group.add(&dir_row);
    page.add(&data_group);

    // ── Appearance group ─────────────────────────────────────────────────────
    let appearance_group = adw::PreferencesGroup::new();
    appearance_group.set_title("Appearance");

    let theme_row = adw::ComboRow::new();
    theme_row.set_title("Theme");
    let theme_model = gtk::StringList::new(&["System Default", "Light", "Dark"]);
    theme_row.set_model(Some(&theme_model));

    let settings = UserSettings::load();
    let theme_idx = match settings.theme {
        Theme::System => 0u32,
        Theme::Light => 1,
        Theme::Dark => 2,
    };
    theme_row.set_selected(theme_idx);

    {
        let sender_theme = sender.clone();
        theme_row.connect_selected_notify(move |row| {
            let theme = match row.selected() {
                1 => "Light",
                2 => "Dark",
                _ => "System",
            };
            sender_theme.input(AppMsg::SetTheme(theme.to_string()));
        });
    }

    appearance_group.add(&theme_row);
    page.add(&appearance_group);

    scroll.set_child(Some(&page));
    scroll.upcast()
}
