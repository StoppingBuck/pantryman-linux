// Tests for app types (Tab, AppMsg).
use pantryman_linux::app::{AppMsg, Tab};

#[test]
fn tab_variants_are_distinct() {
    assert_ne!(Tab::Recipes, Tab::Pantry);
    assert_ne!(Tab::Pantry, Tab::Settings);
}

#[test]
fn appmsg_switch_tab_roundtrip() {
    let msg = AppMsg::SwitchTab(Tab::Recipes);
    match msg {
        AppMsg::SwitchTab(Tab::Recipes) => {}
        _ => panic!("unexpected variant"),
    }
}
