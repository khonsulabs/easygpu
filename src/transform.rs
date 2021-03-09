use euclid::Transform3D;

pub struct ScreenSpace;
pub struct WorldSpace;

pub type ScreenTransformation<S> = Transform3D<S, WorldSpace, ScreenSpace>;
