pub fn single_selector(
    ui: &imgui::Ui,
    elements: &[(usize, String)],
    selected: Option<usize>,
) -> Option<usize> {
    let mut new_selected = selected;

    for (id, name) in elements {
        let _token = ui.push_id(format!("{}_single_selector_entity", name));

        if selected.map_or(false, |selected| selected == *id) {
            if ui.selectable_config(name).selected(true).build() {
                new_selected = None;
            }
        } else if ui.selectable_config(name).selected(false).build() {
            new_selected = Some(*id);
        }
    }

    new_selected
}
