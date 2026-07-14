# Ayagami-gd

Godot bindings for Ayagami, enabling MOC3 models to be imported or loaded at runtime as Scenes.

## Correct Rendering in Godot

Default blend modes available to Shaders in Godot 4.7.x do not match the reference blending.

Until functionality is merged into Godot, a custom build of the engine with patches are required for the shaders to render models properly.
The patch should be applied against 4.7.1

Related PR

https://github.com/godotengine/godot/pull/116686

