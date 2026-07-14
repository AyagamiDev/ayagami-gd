use godot::prelude::*;
use godot::global::Error;
use godot::classes::{
	EditorImportPlugin, IEditorImportPlugin, ResourceSaver
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
		"Ayagami".into()
	}

	fn get_preset_name(&self, _: i32) -> GString {
		"Scene".into()
	}

	fn get_import_options(&self, _: godot::prelude::GString, _: i32) -> Array<AnyDictionary> {
		Array::new()
	}

	fn import(&mut self,
		source_file: GString,
		save_path: GString,
		_options: VarDictionary,
		_platform_variants: Array<GString>,
		_gen_files: Array<GString>
	) -> Error {
		let model = AyagamiLoader::singleton().bind().load_model(source_file);

		// pack the model into a reusable scene
		let mut scn: Gd<PackedScene> = PackedScene::new_gd();
		scn.pack(&model);
		scn.set_path(
			&format!("{}.{}", save_path, self.get_save_extension())
		);

		ResourceSaver::singleton().save(&scn)
	}
}
