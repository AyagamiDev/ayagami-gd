use std::collections::HashMap;
use std::sync::Arc;

use godot::meta::ClassId;
use godot::prelude::*;
use godot::classes::{
	ArrayMesh, INode2D, MeshInstance2D, ShaderMaterial, SubViewport
};
use godot::classes::file_access::ModeFlags;
use godot::classes::notify::CanvasItemNotification;

use ayagami::file::ParsedModel;
use ayagami::core::Model;
use ayagami::driver::Driver;
use ayagami::pose::{Descriptor, Key, Pose, PoseMap};
use godot::register::info::{PropertyHint, PropertyHintInfo, PropertyInfo, PropertyUsageFlags};

use crate::mutator::{IMutator};

pub const PARAMETER_PREFIX: &str = "parameters/";
const PARAMETER_RANGE_SUFFIX: &str = "/range";
const PARAMETER_DEFAULT_SUFFIX: &str = "/default";

pub const PART_PREFIX: &str = "parts/";

pub fn key_param<'a>(property: StringName) -> Key<'a> {
	Key::from_param(property.trim_prefix(PARAMETER_PREFIX).to_string())
}
pub fn key_part<'a>(property: StringName) -> Key<'a> {
	Key::from_part(property.trim_prefix(PART_PREFIX).to_string())
}

pub struct LoadedModel<T: Model, R: AsRef<T>> {
	pub model: R,
	pub driver: Driver<T>,
}

#[derive(GodotClass)]
#[class(tool, init, base = Node2D)]
pub struct AyagamiModel {
	base: Base<Node2D>,
	#[var(pub)]
	size: Vector2i,
	#[var(pub)]
	origin: Vector2,

	pub model: Option<LoadedModel<ParsedModel, Box<ParsedModel>>>,

	dirty: bool,

	mesh_group: Option<Gd<Node2D>>,
	meshes: HashMap<u32, Gd<MeshInstance2D>>,
	masks: Vec<Gd<SubViewport>>,
	mask_lookup: HashMap<StringName, Vec<Gd<MeshInstance2D>>>,

	pub pose: Option<Pose>,
	pub pose_map: Arc<PoseMap>,
}

#[godot_api]
impl AyagamiModel {
	#[signal]
	pub fn loaded();

	fn is_loaded(&self) -> bool {
		self.model.is_some()
	}

	pub fn load(&mut self) {
		let file_path = self.base().get_meta("moc").to::<GString>();
		let mut f = GFile::open(&file_path, ModeFlags::READ).unwrap();

		let model = Box::new(ParsedModel::load(&mut f).unwrap());
		self.pose_map = model.pose_map().clone();
		self.pose = Some(Pose::new(&*model));
		let driver = Driver::new(&*model.as_ref());

		let loaded = LoadedModel {
			model,
			driver,
		};
		self.model = Some(loaded);

		self.dirty = true;
	}

	fn reorder_meshes(&mut self) {
		let mesh_group = self.mesh_group.as_mut().unwrap();
		let md = self.model.as_mut().unwrap();

		// reorder meshes if dirty to properly maintain z-index
		// if Godot ever implements sorting groups (https://github.com/godotengine/godot-proposals/issues/9428)
		// then we will be able to sensibly use z-index
		// but as long as z-index is a global sort order, it's better for use the scene tree
		// and pray that a model isn't constantly changing its render order
		if md.driver.order_changed() {
			for (order, uid) in md.driver.sorted_artmeshes().iter().enumerate() {
				let mesh_instance = &self.meshes[uid];
				
				mesh_group.move_child(mesh_instance, order as i32);
			}
		}
	}

	fn update_meshes(&mut self, force: bool) {
		let binding = self.model.as_mut();
		let md = binding.unwrap();
		let m = md.model.as_ref();
		md.driver.drive(m);

		let px_size = md.model.canvas_properties().scale;

		// update mesh vertices
		for (uid, child) in self.meshes.iter() {
			let mut mesh_instance = child.to_owned().cast::<MeshInstance2D>();
			let mut mesh = mesh_instance.get_mesh().unwrap().cast::<ArrayMesh>();
			
			let m = md.driver.artmesh_state(*uid).unwrap();
			
			let verts = m.vertices;
			let count = verts.len();

			if count < 3 {
				mesh_instance.set_visible(false);
				continue;
			}

			mesh_instance.set_visible(m.visual.visible);
			mesh_instance.set_self_modulate(Color {
				r: 1.0,
				g: 1.0,
				b: 1.0,
				a: m.visual.opacity
			});
			if mesh_instance.get_instance_shader_parameter("color_override") != true.to_variant() {
				mesh_instance.set_instance_shader_parameter("color_multiply", &Color {
					r: m.visual.multiply_color.x,
					g: m.visual.multiply_color.y,
					b: m.visual.multiply_color.z,
					a: 1.0,
				}.to_variant());
				mesh_instance.set_instance_shader_parameter("color_screen", &Color {
					r: m.visual.screen_color.x,
					g: m.visual.screen_color.y,
					b: m.visual.screen_color.z,
					a: 1.0,
				}.to_variant());
			}

			if !force {
				if !m.updated {
					continue;
				}

				if !m.visual.visible {
					continue;
				}
			}
			
			let mut ary = PackedVector2Array::new();
			ary.resize(count);

			let mut vtx_min = Vector3::new(f32::MAX, f32::MAX, 0.0); // top-left
			let mut vtx_max = Vector3::new(f32::MIN, f32::MIN, 0.0); // bottom-right

			for (i, vtx) in verts.iter().enumerate() {
				let x = vtx.x * px_size;
				let y  = vtx.y * px_size;
				vtx_min.x = f32::min(vtx_min.x, x); // left
				vtx_min.y = f32::min(vtx_min.y, y); // top
				vtx_max.x = f32::max(vtx_max.x, x); // right
				vtx_max.y = f32::max(vtx_max.y, y); // bottom

				ary[i] = Vector2::new(x, y);
			}

			mesh.surface_update_vertex_region(0, 0, &ary.to_byte_array());

			// aabb does not get automatically updated when directly updating the vertex region
			let aabb = Aabb::new(Vector3::new(vtx_min.x, vtx_min.y, 0.0), vtx_max - vtx_min);
			let existing_aabb = mesh.get_custom_aabb();

			// only mark as dirty of the bounds of the mesh have shifted so that we may
			// update affected mask viewports
			if aabb != existing_aabb {
				mesh.set_custom_aabb(aabb);
			}
		}
	}

	fn update_masks(&mut self) {
		// update viewport dimensions and transform for masks
		for mask in self.masks.iter_mut() {
			let meshes: Vec<Gd<MeshInstance2D>> = mask.get_children().iter_shared().map(|n| n.cast::<MeshInstance2D>()).collect();
			let mut group_aabb: Aabb = {
				let node = &meshes[0];
				let mesh = node.get_mesh().unwrap().cast::<ArrayMesh>();
				
				mesh.get_custom_aabb()
			};

			for node in meshes.iter() {
				let mesh = node.get_mesh().unwrap().cast::<ArrayMesh>();
				let aabb = mesh.get_custom_aabb();
				group_aabb = group_aabb.merge(aabb);
			}
			
			group_aabb = group_aabb.grow(4.0);

			let dimensions = Vector2i {
				x: group_aabb.size.x as i32,
				y: group_aabb.size.y as i32,
			};
			let offset = Vector2 {
				x: group_aabb.position.x,
				y: group_aabb.position.y,
			};
			mask.set_size(dimensions);
			mask.set_canvas_transform(Transform2D::from_angle_origin(0.0, -offset));

			let dependent_meshes = self.mask_lookup.get_mut(&mask.get_name()).unwrap();
			for node in dependent_meshes.iter_mut() {
				node.set_instance_shader_parameter("mask_offset", &offset.to_variant());
				//node.set_instance_shader_parameter("canvas_size", &offset.to_variant());
			}
		}
	}

	#[func]
	pub fn get_parameters(&self) -> Array<StringName> {
		let mut arr: Vec<_> = self.pose_map.keys().filter_map(
			|k| match k {
				Key::Param(id) => Some(id.to_string()),
				_ => None
			}
		).collect();
		arr.sort();

		Array::from_iter(arr.iter().map(|v| v.to_string_name()))
	}

	#[func]
	pub fn get_parts(&self) -> Array<StringName> {
		let mut arr: Vec<_> = self.pose_map.keys().filter_map(
			|k| match k {
				Key::Part(id) => Some(id.to_string()),
				_ => None
			}
		).collect();
		arr.sort();

		Array::from_iter(arr.iter().map(|v| v.to_string_name()))
	}
}

#[godot_api]
impl INode2D for AyagamiModel {
	fn on_notification(&mut self, what: CanvasItemNotification) {
		// reconnect scene to an ayagami driver when instantiated from an imported resource
		if what == CanvasItemNotification::READY {
			if !self.is_loaded() {
				self.load();
			}

			// cache references so we don't have to go through the scene tree every update
			let mesh_group = self.base().get_node_as::<Node2D>("Meshes");
			self.mesh_group = Some(mesh_group.clone());
			self.meshes = mesh_group.get_children().iter_shared().fold(
				HashMap::new(),
				|mut acc, n| {
					acc.insert(n.get_meta("uid").to::<u32>(), n.cast::<MeshInstance2D>());
					acc
				}
			);

			let mask_group = self.base().get_node_as::<Node>("Masks");
			self.masks = mask_group.get_children().iter_shared().map(|n| n.cast::<SubViewport>()).collect();
			self.mask_lookup = self.masks.clone().into_iter().fold(
				HashMap::new(),
				|mut acc, n| {
					let meshes = n.get_meta("meshes")
						.to::<VarArray>()
						.iter_shared()
						.map(|np| np.to::<NodePath>())
						.map(|path| self.base().get_node_as::<MeshInstance2D>(&path))
						.collect();
					acc.insert(n.get_name(), meshes);
					acc
				}
			);

			self.update_meshes(true);
			self.update_masks();
			self.reorder_meshes();

			self.base_mut().set_process_internal(true);

			self.signals().loaded().emit();
		}
		// reconnect all mask viewport textures to the dependent mesh shaders
		// this is necessary because Viewport texture paths are relative to the absolute scene tree
		else if what == CanvasItemNotification::ENTER_TREE {
			let mask_group = self.base().get_node_as::<Node>("Masks");
			for mask in mask_group.get_children().iter_shared().map(|n| n.cast::<SubViewport>()) {
				for np in mask.get_meta("meshes").to::<VarArray>().iter_shared().map(|v| v.to::<NodePath>()) {
					let node = self.base().get_node_as::<MeshInstance2D>(&np);
					let mut mat = node.get_material().unwrap().cast::<ShaderMaterial>();
					mat.set_shader_parameter(
						"tex_mask",
						&mask.get_texture().unwrap().to_variant()
					);
				}
			}
			/* Bug: godot-rust has not properly scoped this for public access
			RenderingServer::singleton().canvas_item_set_custom_rect_full(
				self.base().get_canvas_item(), true, 
				Rect2i {
					position: Vector2i::ZERO,
					size: self.size
				}
			);
			*/
		}
		// apply mutators to pose and send to driver
		else if what == CanvasItemNotification::INTERNAL_PROCESS && self.is_loaded() {
			let state = &mut Pose::with_map(self.pose_map.clone());
			// start our output pose from base set driven by model properties
			if let Some(pose) = &self.pose {
				state.update(pose);
			}
			
			for child in self.base_mut().get_children().iter_shared() {
				if let Ok(mut mutator) = child.try_dynify::<dyn IMutator>() {
					mutator.dyn_bind_mut().apply(state);
				}
			}

			self.model.as_mut().unwrap().driver.apply_pose(&state);

			self.update_meshes(false);
			self.update_masks();
			self.reorder_meshes();
		}
	}

	fn on_set(&mut self, property: StringName, value: Variant) -> bool {
		if !self.is_loaded() {
			return false;
		}

		// check if attempting to set a value on the internal ayagami driver
		if let Some(pose) = self.pose.as_mut() {
			if property.begins_with(PARAMETER_PREFIX) {
				let key = key_param(property);
				if let Ok(v) = value.try_to::<f32>() {
					pose.set(&key, v);
					return true;
				}
			}
			else if property.begins_with(PART_PREFIX) {
				let key = key_part(property);
				if let Ok(v) = value.try_to::<f32>() {
					pose.set(&key, v);
					return true;
				}
			}
		}
		return false;
	}

	fn on_get(&self, property: StringName) -> Option<Variant> {
		if !self.is_loaded() {
			return None;
		}

		if property.begins_with(PARAMETER_PREFIX) {
			if property.ends_with(PARAMETER_RANGE_SUFFIX) {
				let key = key_param(property.trim_suffix(PARAMETER_RANGE_SUFFIX).to_string_name());
				if let Some((_, p)) = self.pose_map.get(&key) {
					return Some(Vector2 {
						x: p.min, y: p.max
					}.to_variant());
				}
			}
			else if property.ends_with(PARAMETER_DEFAULT_SUFFIX) {
				let key = key_param(property.trim_suffix(PARAMETER_DEFAULT_SUFFIX).to_string_name());
				if let Some((_, p)) = self.pose_map.get(&key) {
					return Some(p.default.to_variant());
				}
			}
			else {
				let key = key_param(property);
				if let Some(pose) = self.pose.as_ref() {
					if let Some(value) = pose.get_flattened(&key) {
						return Some(value.to_variant());
					}
				}
				if let Some((_, p)) = self.pose_map.get(&key) {
					return Some(p.default.to_variant());
				}
			}
		}
		else if property.begins_with(PART_PREFIX) {
			let key = key_part(property);
			if let Some(pose) = self.pose.as_ref() {
				if let Some(value) = pose.get_flattened(&key) {
					return Some(value.to_variant());
				}
			}
			if let Some((_, p)) = self.pose_map.get(&key) {
				return Some(p.default.to_variant());
			}
		}

		return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
		if !self.is_loaded() {
			return Vec::new();
		}

		// expose driver parameters as fields on the model
		let mut descriptors: Vec<&Descriptor> = self.pose_map.descriptors().collect();
		descriptors.sort_by_key(|d| match d.key.clone() {
			Key::Param(id) => id.to_lowercase(),
			Key::Part(id) => id.to_lowercase(),
		});

		descriptors.into_iter().flat_map(
			|param| match param.key.clone() {
				Key::Param(id) => vec![
					PropertyInfo {
						variant_type: VariantType::FLOAT,
						class_name: ClassId::none().to_string_name(),
						property_name: format!("{}{}", PARAMETER_PREFIX, id).to_string_name(),
						hint_info: PropertyHintInfo {
							hint: PropertyHint::RANGE,
							hint_string: format!("{},{}", param.min, param.max).to_gstring(),
						},
						usage: PropertyUsageFlags::STORAGE | PropertyUsageFlags::EDITOR,
					},
					PropertyInfo {
						variant_type: VariantType::VECTOR2,
						class_name: ClassId::none().to_string_name(),
						property_name: format!("{}{}{}", PARAMETER_PREFIX, id, PARAMETER_RANGE_SUFFIX).to_string_name(),
						hint_info: PropertyHintInfo::none(),
						usage: PropertyUsageFlags::READ_ONLY | PropertyUsageFlags::EDITOR,
					},
					PropertyInfo {
						variant_type: VariantType::FLOAT,
						class_name: ClassId::none().to_string_name(),
						property_name: format!("{}{}{}", PARAMETER_PREFIX, id, PARAMETER_DEFAULT_SUFFIX).to_string_name(),
						hint_info: PropertyHintInfo::none(),
						usage: PropertyUsageFlags::READ_ONLY | PropertyUsageFlags::EDITOR,
					}
				],
				Key::Part(id) => vec![
					PropertyInfo {
						variant_type: VariantType::FLOAT,
						class_name: ClassId::none().to_string_name(),
						property_name: format!("{}{}", PART_PREFIX, id).to_string_name(),
						hint_info: PropertyHintInfo {
							hint: PropertyHint::RANGE,
							hint_string: format!("{},{}", 0.0, 1.0).to_gstring(),
						},
						usage: PropertyUsageFlags::STORAGE | PropertyUsageFlags::EDITOR,
					}
				]
			}
		).collect()
	}

	fn on_property_get_revert(&self, property: StringName) -> Option<Variant> {
		if self.model.is_none() {
			return None;
		}

		if property.begins_with(PARAMETER_PREFIX) {
			let key = key_param(property.clone());
			if let Some((_, p)) = self.pose_map.get(&key) {
				return Some(p.default.to_variant());
			}
		}
		else if property.begins_with(PART_PREFIX) {
			return Some(1.0.to_variant());
		}

		return None;
	}
}
