use nalgebra::Transform3;

struct SceneNode {
    local_transform: Transform3<f32>,
    world_transform: Transform3<f32>,
    world_transform_dirty: bool,
    visible: bool,
    kind: NodeType,
}

enum NodeType {
    Renderable(Renderable),
    Group,
}

struct Renderable {
    geometry_id: GeometryId,
    material_id: MaterialId,
}
