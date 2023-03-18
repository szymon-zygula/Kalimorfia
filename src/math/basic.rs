pub fn normalize_for(main: f32, first: f32, second: f32) -> (f32, f32) {
    if second == 0.0 {
        return ((1.0 - main * main).sqrt(), 0.0);
    }

    if first == 0.0 {
        return (0.0, (1.0 - main * main).sqrt());
    }

    let goal = 1.0 - main * main;
    let ratio = first / second;

    let new_second = (goal / (1.0 + ratio * ratio)).sqrt();
    (ratio * new_second, new_second)
}
