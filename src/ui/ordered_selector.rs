pub fn ordered_selector(
    ui: &imgui::Ui,
    elements: Vec<(usize, String, bool)>,
) -> Vec<(usize, bool)> {
    let mut new_selects = Vec::with_capacity(elements.len());
    let mut swaps = Vec::new();

    for i in 0..elements.len() {
        let (id, ref name, selected) = elements[i];
        let _token = ui.push_id(format!("{}_entity", name));

        if selected {
            ui.columns(3, "columns", false);
        }

        new_selects.push((
            id,
            ui.selectable_config(name).selected(selected).build() ^ selected,
        ));

        if selected {
            ui.next_column();
            ui.set_current_column_width(30.0);
            if i != 0 && ui.button("^") {
                swaps.push((i, i - 1));
            }

            ui.next_column();
            ui.set_current_column_width(30.0);
            if i != elements.len() - 1 && elements[i + 1].2 && ui.button("v") {
                swaps.push((i, i + 1));
            }

            ui.next_column();
        }
    }

    for (idx1, idx2) in swaps {
        new_selects.swap(idx1, idx2);
    }

    new_selects
}

pub fn selected_only(new_selection: &[(usize, bool)]) -> Vec<usize> {
    new_selection
        .iter()
        .filter(|(_, selected)| *selected)
        .map(|(id, _)| *id)
        .collect()
}

pub fn changed(new_selected: &[usize], old_selected: &[usize]) -> bool {
    old_selected.iter().ne(new_selected.iter())
}
