use ayagami::meta::Physics3;
use ayagami::physics::{PhysicsEngine, PhysicsOptions};
use godot::classes::{FileAccess, IResourceFormatLoader, ResourceFormatLoader, ResourceLoader};
use godot::prelude::*;
use godot::global::Error;

use crate::mutator::{IMutator};
use ayagami::pose::Pose;

pub const PHYSICS_EXTENSION: &str = "physics3.json";

#[derive(GodotClass)]
#[class(tool, no_init, base = Resource)]
pub struct AyagamiPhysicsMeta {
    pub physics: Physics3
}

impl AyagamiPhysicsMeta {
    pub fn as_options(&self) -> PhysicsOptions {
        PhysicsOptions::accurate(Some(60.0))
    }
}

#[derive(GodotClass)]
#[class(tool, init, base=ResourceFormatLoader)]
pub struct AyagamiPhysicsLoader {
	base: Base<ResourceFormatLoader>,
}

#[godot_api]
impl IResourceFormatLoader for AyagamiPhysicsLoader {

	fn get_recognized_extensions(&self) -> PackedStringArray {
		PackedArray::from([
			"json".into()
		])
	}

    fn get_resource_script_class(&self, path: GString) -> GString {
		if path.ends_with(PHYSICS_EXTENSION) {
			"AyagamiPhysicsMeta".into()
		} else {
			GString::new()
		}
    }

    fn handles_type(&self, ty: StringName) -> bool {
        ty == "AyagamiPhysicsMeta".to_string_name()
    }

	fn get_resource_type(&self, path: GString) -> GString {
		if path.ends_with(PHYSICS_EXTENSION) {
			"AyagamiPhysicsMeta".into()
		} else {
			GString::new()
		}
	}

	fn load(&self,
		path: GString,
		original_path: GString,
		_sub_threads: bool,
		_cache_mode: i32
	) -> Variant {
		let s = FileAccess::get_file_as_string(if !original_path.is_empty() { &original_path } else { &path });
        if !s.is_empty() {
            if let Ok(physics) = serde_json::from_str(&s.to_string()) {
				return Gd::from_object(AyagamiPhysicsMeta {
                    physics
				}).to_variant();
            }
        }
        return Error::FAILED.to_variant();
	}
}

#[derive(GodotConvert, Debug, Var, Export, Default, Clone)]
#[godot(via = i64)]
pub enum PhysicsMode {
	#[default]
	COMPATIBLE,
	USEFUL,
	ACCURATE,
}

#[derive(GodotClass)]
#[class(tool, init, base=Node)]
pub struct AyagamiPhysicsMutator {
    base: Base<Node>,
    physics_controller: Option<ayagami::physics::PhysicsEngine>,

	#[export(file = "*.physics3.json")]
    #[var(set = set_definition)]
	pub definition: GString,
    options: Option<Gd<AyagamiPhysicsMeta>>,

	#[export]
    #[var(set = set_fps)]
	pub fps: f32,

    #[export]
    #[var(set = set_mode)]
    pub mode: PhysicsMode
}

#[godot_api]
impl AyagamiPhysicsMutator {
    fn reload_physics(&mut self) {
        if let Some(options) = self.options.as_ref() {
            let maybe_fps = if self.fps > 0.0 { Some(self.fps) } else { None };
            self.physics_controller = Some(PhysicsEngine::new(
                options.bind().physics.clone(),
                match self.mode {
                    PhysicsMode::ACCURATE => PhysicsOptions::accurate(maybe_fps),
                    PhysicsMode::USEFUL => PhysicsOptions::useful(maybe_fps),
                    PhysicsMode::COMPATIBLE => PhysicsOptions::compatible(maybe_fps),
                }
            ));
        } else {
            self.physics_controller = None;
        }
    }

    #[func]
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
        self.reload_physics();
    }


    #[func]
    pub fn set_mode(&mut self, mode: PhysicsMode) {
        self.mode = mode;
        self.reload_physics();
    }

    #[func]
    pub fn set_definition(&mut self, path: GString) {
        if path.is_empty() {
            self.options = None;
            return;
        }

        let Some(options) = ResourceLoader::singleton()
            .load_ex(&path)
                .type_hint("AyagamiPhysicsMeta")
            .done()
            .map(|r| r.try_cast::<AyagamiPhysicsMeta>().unwrap() ) 
            else {
                panic!("Could not load physics from {}", path)
            };
        self.definition = path;
        self.options = Some(options);

        self.reload_physics();
    }
}

#[godot_dyn]
impl IMutator for AyagamiPhysicsMutator {
	fn apply(&mut self, pose: &mut Pose) {
        let delta: f32 = self.base().get_process_delta_time() as f32;
        if let Some(controller) = self.physics_controller.as_mut() {
            controller.update(pose, delta);
        }
	}
}
