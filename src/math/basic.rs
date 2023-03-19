pub fn normalize_for(mut main: f32, first: f32, second: f32) -> (f32, f32, f32) {
    if main < -1.0 {
        main = -1.0;
    } else if main > 1.0 {
        main = 1.0;
    }

    if second == 0.0 {
        return (main, (1.0 - main * main).sqrt(), 0.0);
    }

    if first == 0.0 {
        return (main, 0.0, (1.0 - main * main).sqrt());
    }

    let goal = 1.0 - main * main;
    let ratio = first / second;

    let new_second = (goal / (1.0 + ratio * ratio)).sqrt();
    (main, ratio * new_second, new_second)
}
