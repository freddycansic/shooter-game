Game / Editor
- Engine
- World
- Vec\<System\>
- Application specific state e.g. ui

Engine
- Renderer
- Input
- Gui
- Physics backend (does the work)

World
- World graph = transforms only
- List of components and which nodes they belong to
- Physics context = which entities own which colliders + state

# ECS

## Entities
Bound together components

```rust
type Entity = NodeIndex; // to index into the world graph

world.spawn((Transform::default(), Geometry::from_path(path)));
```

## Components
Actual data


## Systems
Reads and modifies world

```rust
fn system(camera: QueryOne<(&Camera, &Transform)>, geometry: Query<(&Geometry, Option<&Material>)>)
```

Archetypes
- Unique sets of components
- All entities in structure of arrays so that components are contiguous

