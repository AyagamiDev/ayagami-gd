# Ayagami-gd

Godot bindings for [Ayagami](https://github.com/AyagamiDev/ayagami), enabling MOC3 models to be imported or loaded at runtime as Scenes.

Features
- importing `.model3.json` as `PackedScene` resources
- loading `.model3.json` at runtime as Scenes
- importing `.motion3.json` as `Animation` resources
- importing models with motion libraries, consisting of all path relative animations
- import `.exp3.json` expressions
- control expressions with tweenable weights or groupable toggle states

## TODO
- physics

## Correct Rendering in Godot

Default blend modes available to Shaders in Godot 4.7.x do not match the reference blending.

Until functionality is merged into Godot, a custom build of the engine with patches are required for the shaders to render models properly.
The patch should be applied against 4.7.1

Related PR<br>
https://github.com/godotengine/godot/pull/116686

## Asset Attribution

Godot Editor Icons<br>
https://godotengine.github.io/editor-icons/

