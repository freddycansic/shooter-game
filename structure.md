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

Systems
- Reads and modifies world