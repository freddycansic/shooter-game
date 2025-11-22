// Can make these derive from something, then can do a custom serde deserialiser
// Serialize as path, then do a PathBuf -> T conversion using serde::deserialize attribute
// Means I don't have to make a serialisable copy struct

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometryHandle(pub (usize, usize));

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub usize);
