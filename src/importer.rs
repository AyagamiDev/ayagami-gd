use godot::prelude::*;
use godot::global::Error;
use godot::classes::{
	AnimationPlayer, EditorImportPlugin, IEditorImportPlugin, ResourceSaver
};

use crate::loader::AyagamiLoader;

#[derive(GodotClass)]
#[class(tool, init, base=EditorImportPlugin)]
pub struct AyagamiImporter {
	base: Base<EditorImportPlugin>,
}

#[godot_api]
impl IEditorImportPlugin for AyagamiImporter {
	fn get_importer_name(&self) -> GString {
		"ayagami.model".into()
	}

	fn get_recognized_extensions(&self) -> PackedArray<GString> {
		PackedArray::from(["model3.json".into()])
	}

	fn get_save_extension(&self) -> GString {
		"tscn".into()
	}

	fn get_resource_type(&self) -> GString {
		"PackedScene".into()
	}

	fn get_visible_name(&self) -> GString {
		"Scene (Ayagami)".into()
	}

	fn get_preset_name(&self, _: i32) -> GString {
		"Scene".into()
	}

	fn get_import_options(&self, _: godot::prelude::GString, _: i32) -> Array<AnyDictionary> {
		let include_expressions = vdict! {
			"name" => "include_expressions",
			"default_value" => true,
		};
		let include_motions = vdict! {
			"name" => "include_motions",
			"default_value" => true,
		};

		array![
			AnyDictionary::from_variant(&include_expressions.to_variant()),
			AnyDictionary::from_variant(&include_motions.to_variant()),
		]
	}

	fn import(&mut self,
		source_file: GString,
		save_path: GString,
		options: VarDictionary,
		_platform_variants: Array<GString>,
		_gen_files: Array<GString>
	) -> Error {
		let model = AyagamiLoader::singleton().bind().load_model(source_file);

		if options.at("include_motions").to::<bool>() {
			let mut animation_player = model.get_node_as::<AnimationPlayer>(&"MotionController".to_node_path());
			let mut animation_library = AyagamiLoader::singleton().bind().load_motion_library(model.clone());
			animation_library.set_local_to_scene(true);
			animation_player.remove_animation_library("");
			animation_player.add_animation_library("", &animation_library);
		}

		// pack the model into a reusable scene
		let mut scn: Gd<PackedScene> = PackedScene::new_gd();
		scn.pack(&model);
		scn.set_path(
			&format!("{}.{}", save_path, self.get_save_extension())
		);

		ResourceSaver::singleton().save(&scn)
	}
}

#[derive(GodotClass)]
#[class(tool, init, base=EditorImportPlugin)]
pub struct AyagamiMotionImporter {
	base: Base<EditorImportPlugin>,
}

#[godot_api]
impl IEditorImportPlugin for AyagamiMotionImporter {
	fn get_importer_name(&self) -> GString {
		"ayagami.motion".into()
	}

	fn get_recognized_extensions(&self) -> PackedArray<GString> {
		PackedArray::from([
			"motion3.json".into()
		])
	}

	fn get_save_extension(&self) -> GString {
		"tres".into()
	}

	fn get_resource_type(&self) -> GString {
		"Animation".into()
	}

	fn get_visible_name(&self) -> GString {
		"Animation (Ayagami Motion)".into()
	}

	fn get_preset_name(&self, _: i32) -> GString {
		"Animation".into()
	}

	fn get_import_options(&self, _: godot::prelude::GString, _: i32) -> Array<AnyDictionary> {
		array![]
	}

	fn import(&mut self,
		source_file: GString,
		save_path: GString,
		_options: VarDictionary,
		_platform_variants: Array<GString>,
		_gen_files: Array<GString>
	) -> Error {
		match AyagamiLoader::singleton().bind().load_motion(source_file) {
			Some(mut animation) => {
				// pack the model into a reusable scene
				animation.set_path(
					&format!("{}.{}", save_path, self.get_save_extension())
				);

				ResourceSaver::singleton().save(&animation)
			},
			None => Error::FAILED
		}
	}
}
