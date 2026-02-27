// Tests for app types (Tab, AppMsg).
use cookbook_gtk::app::{AppMsg, Tab};

#[test]
fn tab_variants_are_distinct() {
    assert_ne!(Tab::Recipes, Tab::Pantry);
    assert_ne!(Tab::Pantry, Tab::Kb);
    assert_ne!(Tab::Kb, Tab::Settings);
}

#[test]
fn appmsg_switch_tab_roundtrip() {
    let msg = AppMsg::SwitchTab(Tab::Recipes);
    match msg {
        AppMsg::SwitchTab(Tab::Recipes) => {}
        _ => panic!("unexpected variant"),
    }
}
