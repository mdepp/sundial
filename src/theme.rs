pub trait Theme {
    type Color;

    fn background(&self) -> Self::Color;
    fn text(&self) -> Self::Color;
}
