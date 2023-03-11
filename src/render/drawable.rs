pub trait Drawable {
    fn draw(&self, gl: &glow::Context);
}
