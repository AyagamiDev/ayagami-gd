use godot::prelude::*;
use godot::classes::{
	Resource
};

#[derive(GodotClass)]
#[class(tool, init, base = Resource)]
pub struct AyagamiExpression;
