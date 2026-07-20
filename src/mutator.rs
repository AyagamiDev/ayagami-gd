use godot::prelude::*;

pub type Pose = Dictionary<StringName, f32>;

pub trait IPoseMutator {
	fn apply(&mut self, _pose: Pose) {}
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiPoseMutator {
	base: Base<Node>
}

#[godot_dyn]
impl IPoseMutator for AyagamiPoseMutator {
	fn apply(&mut self, pose: Pose) {
		<Self>::apply(self, pose);
	}
}

#[godot_api]
pub impl AyagamiPoseMutator {
	#[func(virtual)]
	fn apply(&mut self, _pose: Pose) {
		
	}
}
