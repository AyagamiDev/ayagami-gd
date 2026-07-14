use std::collections::HashMap;

use godot::meta::ClassId;
use godot::prelude::*;
use godot::classes::{
	ArrayMesh, INode2D, MeshInstance2D, ProjectSettings
};
use godot::classes::file_access::ModeFlags;
use godot::classes::notify::CanvasItemNotification;

use ayagami::file::ParsedModel;
use ayagami::core::{Item, Model, Param};
use ayagami::driver::Driver;
use godot::register::info::{PropertyHint, PropertyHintInfo, PropertyInfo, PropertyUsageFlags};

pub struct LoadedModel<T: Model, R: AsRef<T>> {
	pub model: R,
	pub driver: Driver<T>,
}

#[derive(GodotClass)]
#[class(tool, init, base = Node2D)]
pub struct AyagamiModel {
	base: Base<Node2D>,
	pub model: Option<LoadedModel<ParsedModel, Box<ParsedModel>>>,

	// fast access map of driver parameter ids as godot properties
	pub param_lookup: HashMap<StringName, u32>,
	// current state of each set parameter
	pub parameters: HashMap<StringName, f32>
}

#[godot_api]
impl AyagamiModel {
	fn is_loaded(&self) -> bool {
		self.model.is_some()
	}

	pub fn load(&mut self) {
		let file_path = self.base().get_meta("moc").to::<GString>();
		let abs_path = ProjectSettings::singleton().globalize_path(&file_path);
		godot_print!("[Ayagami] model path: {} ({})", file_path, abs_path);
		let mut f = GFile::open(&file_path, ModeFlags::READ).unwrap();

		let model = Box::new(ParsedModel::load(&mut f).unwrap());
		let driver = Driver::new(&*model.as_ref());

		self.param_lookup = model.params().into_iter().fold(
			HashMap::new(),
			|mut acc, p| {
				acc.insert(format!("parameters/{}", p.id()).to_string_name(), p.uid());
				acc
			}
		);
		self.parameters = model.params().into_iter().fold(
			HashMap::new(),
			|mut acc, p| {
				acc.insert(format!("parameters/{}", p.id()).to_string_name(), p.default());
				acc
			}
		);

		self.model = Some(LoadedModel {
			model,
			driver,
		});
	}

	pub fn update_meshes(&mut self) {
		let mut mesh_group = self.base().get_node_as::<Node>("Meshes");
		let meshes = mesh_group.get_children().iter_shared().fold(
			HashMap::new(),
			|mut acc, n| {
				acc.insert(n.get_meta("uid").to::<u32>(), n);
				acc
			}
		);
		
		let binding = self.model.as_mut();
		let md = binding.unwrap();
		let m = md.model.as_ref();
		md.driver.drive(m);

		// reorder meshes if dirty to properly maintain z-index
		// if Godot ever implements sorting groups (https://github.com/godotengine/godot-proposals/issues/9428)
		// then we will be able to sensibly use z-index
		// but as long as z-index is a global sort order, it's better for use the scene tree
		// and pray that a model isn't constantly changing its render order
		if md.driver.order_changed() {
			for (order, uid) in md.driver.sorted_artmeshes().iter().enumerate() {
				let mesh_instance = meshes.get(uid).unwrap();
				mesh_group.move_child(mesh_instance, order as i32);
			}
		}

		let px_size = md.model.canvas_properties().scale;
		let origin = md.model.canvas_properties().center;
		
		// update mesh vertices
		for (uid, child) in meshes.iter() {
			let mut mesh_instance = child.to_owned().cast::<MeshInstance2D>();
			let mut mesh = mesh_instance.get_mesh().unwrap().cast::<ArrayMesh>();
			
			let maybe_m = md.driver.artmesh_state(*uid);
			if maybe_m.is_none() {
				continue;
			}
			let m = maybe_m.unwrap();
			if !m.updated {
				continue;
			}
			
			let verts = m.vertices;
			let count = verts.len();
			mesh_instance.set_visible(m.visual.visible);

			if m.visual.visible {
				let mut ary = PackedVector2Array::new();
				ary.resize(count);

				let mut vtx_min = Vector3::new(f32::MAX, f32::MAX, 0.0); // top-left
				let mut vtx_max = Vector3::new(f32::MIN, f32::MIN, 0.0); // bottom-right

				for (i, vtx) in verts.iter().enumerate() {
					let x = vtx.x * px_size + origin.x;
					let y  = vtx.y * px_size + origin.y;
					vtx_min.x = f32::min(vtx_min.x, x); // left
					vtx_min.y = f32::min(vtx_min.y, y); // top
					vtx_max.x = f32::max(vtx_max.x, x); // right
					vtx_max.y = f32::max(vtx_max.y, y); // bottom

					ary[i] = Vector2::new(x, y);
				}

				mesh.surface_update_vertex_region(0, 0, &ary.to_byte_array());

				// aabb does not get automatically updated when directly updating the vertex region
				let aabb = Aabb::new(Vector3::new(vtx_min.x, vtx_max.y, 0.0), vtx_max - vtx_min);
				mesh.set_custom_aabb(aabb);
			}
		}
	}
}

#[godot_api]
impl INode2D for AyagamiModel {
	fn on_notification(&mut self, what: CanvasItemNotification) {
		// reconnect scene to an ayagami driver when instantiated from an imported resource
		if what == CanvasItemNotification::READY && !self.is_loaded() {
			self.load();
		}
	}

	fn process(&mut self, _delta: f32) {
		if !self.is_loaded() {
			return;
		}

		self.update_meshes();

		// reorder mesh nodes if z-index has changed for any

		// update AABB of model
	}

	fn on_set(&mut self, parameter: StringName, value: Variant) -> bool {
		if !self.is_loaded() {
			return false;
		}

		let md = self.model.as_mut().unwrap();

		// check if attempting to set a value on the internal ayagami driver
		if parameter.begins_with("parameters/") {
			if let Some(uid) = self.param_lookup.get(&parameter) {
				let r = md.driver.set_param(*uid, value.to());
				if r.is_ok() {
					let _ = self.parameters.insert(parameter, value.to());

					return true;
				}
			}
		}
		
		return false;
	}

	fn on_get(&self, parameter: StringName) -> Option<Variant> {
		if !self.is_loaded() {
			return None;
		}

		if parameter.begins_with("parameters/") {
			if let Some(value) = self.parameters.get(&parameter) {
				return Some(value.to_variant());
			}
		}

		return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
		let mut custom_params: Vec<PropertyInfo> = Vec::new();

		if !self.is_loaded() {
			godot_print!("ayagami model not yet loaded");
			return custom_params;
		}

		let md = self.model.as_ref().unwrap();
		let m = md.model.as_ref();

		// expose driver parameters as fields on the model
		custom_params.push(PropertyInfo {
			variant_type: VariantType::NIL,
			class_name: ClassId::none().to_string_name(),
			property_name: "Parameters".to_string_name(),
			hint_info: PropertyHintInfo { hint: PropertyHint::NONE, hint_string: "".to_gstring() },
			usage: PropertyUsageFlags::CATEGORY
		});
		for param in m.params().into_iter() {
			custom_params.push(PropertyInfo {
				variant_type: VariantType::FLOAT,
				class_name: ClassId::none().to_string_name(),
				property_name: format!("parameters/{}", param.id()).to_string_name(),
				hint_info: PropertyHintInfo {
					hint: PropertyHint::RANGE,
					hint_string: format!("{},{}", param.min(), param.max()).to_gstring(),
				},
				usage: PropertyUsageFlags::STORAGE,
			});
		}

		custom_params
	}
}
