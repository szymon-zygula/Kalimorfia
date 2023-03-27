pub fn ordered_selelector(
    ui: &imgui::Ui,
    elements: Vec<(usize, String, bool)>,
) -> Vec<(usize, bool)> {
    ui.columns(3, "columns", false);
    elements
        .iter()
        .map(|(id, name, selected)| {
            (
                *id,
                ui.selectable_config(name).selected(*selected).build() ^ *selected,
            )
        })
        .collect()
}
