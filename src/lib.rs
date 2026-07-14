use godot::obj::NewGd;
use godot::prelude::*;
use godot::classes::{
    EditorPlugin,
    IEditorPlugin
};

use crate::importer::AyagamiImporter;

struct AyagamiExtension;

pub mod model;
pub mod loader;
pub mod importer;

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct AyagamiPlugin {
    base: Base<EditorPlugin>,
    importer: Gd<AyagamiImporter>,
}

#[godot_api]
impl IEditorPlugin for AyagamiPlugin {
    fn enter_tree(&mut self) {
        let plugin: Gd<AyagamiImporter> = AyagamiImporter::new_gd();
        self.importer = plugin.clone();
        self.base_mut().add_import_plugin(&plugin);
    }

    fn exit_tree(&mut self) {
        let plugin = self.importer.clone();
        self.base_mut().remove_import_plugin(&plugin);
    }
}

#[gdextension]
unsafe impl ExtensionLibrary for AyagamiExtension {

}

