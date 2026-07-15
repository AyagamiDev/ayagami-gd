use godot::obj::NewGd;
use godot::prelude::*;
use godot::classes::{
    EditorPlugin,
    IEditorPlugin
};

use crate::importer::*;

struct AyagamiExtension;

pub mod model;
pub mod expression;
pub mod loader;
pub mod importer;

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct AyagamiPlugin {
    base: Base<EditorPlugin>,
    model_importer: Gd<AyagamiImporter>,
    motion_importer: Gd<AyagamiMotionImporter>,
}

#[godot_api]
impl IEditorPlugin for AyagamiPlugin {
    fn enter_tree(&mut self) {
        {
            let plugin: Gd<AyagamiImporter> = AyagamiImporter::new_gd();
            self.model_importer = plugin.clone();
            self.base_mut().add_import_plugin(&plugin);
        }

        {
            let plugin: Gd<AyagamiMotionImporter> = AyagamiMotionImporter::new_gd();
            self.motion_importer = plugin.clone();
            self.base_mut().add_import_plugin(&plugin);
        }
    }

    fn exit_tree(&mut self) {
        {
            let plugin = self.model_importer.clone();
            self.base_mut().remove_import_plugin(&plugin);
        }

        {
            let plugin = self.motion_importer.clone();
            self.base_mut().remove_import_plugin(&plugin);
        }
    }
}

#[gdextension]
unsafe impl ExtensionLibrary for AyagamiExtension {

}

