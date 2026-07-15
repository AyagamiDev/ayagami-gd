use std::collections::HashMap;

use glob::glob;
use godot::classes::animation::{LoopMode, TrackType};
use godot::classes::mesh::PrimitiveType;
use godot::prelude::*;
use godot::classes::{
	Animation, AnimationLibrary, AnimationPlayer, ArrayMesh, FileAccess, Image, ImageTexture, Json, MeshInstance2D, ProjectSettings, ResourceLoader, Shader, ShaderMaterial, SubViewport, Texture2D, ViewportTexture, mesh
};

use ayagami::core::{
	ArtMesh, BlendMode, Collection, Item, Model
};

use crate::expression::AyagamiExpression;
use crate::model::AyagamiModel;

fn shader_material( s: &str ) -> Gd<ShaderMaterial> {
	let mut rl = ResourceLoader::singleton();
	let shader: Gd<Shader> = rl.load(s).unwrap().cast();

	let mut material = ShaderMaterial::new_gd();
	material.set_shader(&shader);
	material.set_local_to_scene(true);

	material
}

#[derive(GodotClass)]
#[class(tool, init, singleton)]
pub struct AyagamiLoader;

#[godot_api]
impl AyagamiLoader {
	#[func]
	pub fn load_model(&self, file_path: GString) -> Gd<AyagamiModel> {
		let json = FileAccess::get_file_as_string(&file_path);
		let settings: VarDictionary = Json::parse_string(&json).to();
		let base_path = file_path.get_base_dir();

		let file_refs: VarDictionary = settings.at("FileReferences").to();
		let mut scene = AyagamiModel::new_alloc();
		scene.set_meta("basepath", &base_path.to_variant());

		let model_file = file_refs.at("Moc").to_string();
		let model_path = base_path.path_join(&model_file);
		scene.set_meta("moc", &model_path.to_variant());

		// build materials for each texture
		let texture_paths: VarArray = file_refs.at("Textures").to();
		let textures: Array<Gd<Texture2D>> = texture_paths.iter_shared().map(|t_path| {
			let real_path = &base_path.path_join(&t_path.to_string());
			let mut tex: Gd<Texture2D>;
			if ResourceLoader::singleton().exists(real_path) {
				tex = ResourceLoader::singleton().load(real_path).unwrap().cast();
			} else {
				let mut img = Image::load_from_file(real_path);
				img.as_mut().unwrap().generate_mipmaps();
				let img_tex = ImageTexture::create_from_image(img.as_ref()).unwrap();
				tex = img_tex.upcast();
				tex.take_over_path(real_path);
			}

			tex
		}).collect();

		let shaders: Array<Gd<ShaderMaterial>> = array![
			&shader_material("res://addons/ayagami/shaders/mix.gdshader"),
			&shader_material("res://addons/ayagami/shaders/add.gdshader"),
			&shader_material("res://addons/ayagami/shaders/mul.gdshader"),
			&shader_material("res://addons/ayagami/shaders/mask.gdshader")
		];

		let mut meshes = HashMap::new();
		let mut mesh_group = Node2D::new_alloc();
		mesh_group.set_name("Meshes");
		scene.add_child(&mesh_group);
		mesh_group.set_owner(&scene);

		let mut masks = HashMap::new();
		let mut mask_group = Node2D::new_alloc(); // using node2d so texture filter settings can propogate down to masks
		mask_group.set_name("Masks");
		scene.add_child(&mask_group);
		mask_group.set_owner(&scene);

		let mut motion_controller = AnimationPlayer::new_alloc();
		motion_controller.set_name("MotionController");
		scene.add_child(&motion_controller);
		motion_controller.set_owner(&scene);

		let mut m_scene = scene.bind_mut();
		m_scene.load();
		
		let md = m_scene.model.as_mut().unwrap();
		let m = md.model.as_ref();

		// update art meshes to their initial states
		md.driver.drive(m);

		let px_size = m.canvas_properties().scale;
		let canvas_size = m.canvas_properties().dimensions;

		// make all the art meshes
		for uid in md.driver.sorted_artmeshes().into_iter() {
			let artmesh = md.model.artmeshes().get(*uid).unwrap();
			// TODO get mesh state when parameters are at defaults
			// let raw_mesh = md.driver.artmesh_state(artmesh.uid());
			let mut mesh = ArrayMesh::new_gd();
			let id = artmesh.id().to_string_name();
			let uid = artmesh.uid().clone();
			mesh.set_local_to_scene(true);

			let vtx_count = artmesh.vertex_count();
			let am = md.driver.artmesh_state(uid).unwrap();
					
			// must have enough vertices to create a tri
			if vtx_count < 3 {
				continue;
			}

			let mut ary = VarArray::new();
			ary.resize(mesh::ArrayType::MAX.ord() as usize, &Variant::nil());

			// vertices
			{
				let mut vary = PackedVector2Array::new();
				vary.resize(vtx_count as usize);

				for (i, vtx) in am.vertices.iter().enumerate() {
					vary[i] = Vector2 {
						x: vtx.x * px_size,
						y: vtx.y * px_size
					};
				}
				ary.set(
					mesh::ArrayType::VERTEX.ord() as usize,
					&vary
				);
			}
			
			// texture UVs
			{
				let texcoords = md.model.texcoord_buffer().unwrap();
				let offset = artmesh.texcoord_offset();
				let mut vary = PackedVector2Array::new();
				vary.resize(vtx_count as usize);

				for i in 0..vtx_count {
					let uv = texcoords[(offset + i) as usize];
					vary[i as usize] = Vector2 {
						x: uv.x,
						y: uv.y
					};
				}

				ary.set(
					mesh::ArrayType::TEX_UV.ord() as usize,
					&vary
				);
			}

			// indices
			{
				let vary = PackedInt32Array::from_iter(
					artmesh.indices_slice().iter().map(|i| *i as i32)
				);
				
				ary.set(
					mesh::ArrayType::INDEX.ord() as usize,
					&vary
				);
			}

			mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &ary);

			let aabb = mesh.get_aabb();
			mesh.set_custom_aabb(aabb);

			let mut mesh_instance = MeshInstance2D::new_alloc();
			mesh_instance.set_name(&id);
			mesh_instance.set_mesh(&mesh);
			mesh_instance.set_meta("uid", &uid.to_variant());
			mesh_instance.set_self_modulate(Color { r: 1.0, g: 1.0, b: 1.0, a: am.visual.opacity });
			
			let tex_id = artmesh.texture() as usize;
			let tex = textures.get(tex_id).unwrap();
			mesh_instance.set_texture(&tex);

			let mat = match artmesh.blend_mode() {
				BlendMode::Normal => Some(shaders.at(0)),
				BlendMode::Add => Some(shaders.at(1)),
				BlendMode::Multiply => Some(shaders.at(2))
			};
			mesh_instance.set_material(mat.as_ref());

			mesh_group.add_child(&mesh_instance);
			meshes.insert(artmesh.uid(), mesh_instance);
		}
		
		// make masks and attach them to art meshes
		for artmesh in  m.artmeshes() {
			let mask_parts: Vec<u32> = artmesh.clips().into_iter().map(|p| p.uid()).collect();
			if mask_parts.len() > 0 {
				let mask_ids = PackedArray::from_iter(mask_parts.clone().into_iter().map(|c| c.to_string().to_gstring()));
				let hash = GString::from("_").join(&mask_ids).to_string_name();
				let mask_name = hash.clone();

				let mask = masks.entry(hash).or_insert_with(
					|| {
						let mut vp = SubViewport::new_alloc();
						vp.set_name(&mask_name);
						vp.set_transparent_background(true);

						// TODO enable this once godot-rust exposes the enum, it's currently missing in 0.5.4
						// vp.set_default_canvas_item_texture_filter(DefaultCanvasItemTextureFilter::PARENT_NODE);

						// canvas transform can not be set before the node is in the SceneTree
						// accurate viewport size and offsets will be delayed until the model
						// is added and visible for rendering
						vp.set_size(Vector2i { x: 2, y: 2 });

						for part in mask_parts {
							let src_mi = meshes.get(&part).unwrap();
							let mesh = src_mi.get_mesh().unwrap().cast::<ArrayMesh>();
							let mut mi = MeshInstance2D::new_alloc();
							mi.set_name(&src_mi.get_name());
							mi.set_mesh(&mesh);
							mi.set_texture(&src_mi.get_texture().unwrap());
							mi.set_material(&shaders.at(3));
							vp.add_child(&mi);
						}

						mask_group.add_child(&vp);
						
						return vp;
					}
				);

				let node = meshes.get_mut(&artmesh.uid()).unwrap();
				let offset = mask.get_canvas_transform().origin;
				
				let mut tex = ViewportTexture::new_gd();
				tex.set_local_to_scene(true);

				let mut mat = node.get_material().unwrap().duplicate_resource().cast::<ShaderMaterial>();
				mat.set_shader_parameter("tex_mask", &tex.to_variant());
				mat.set_shader_parameter("has_mask", &true.to_variant());
				mat.set_shader_parameter("mesh_offset", &offset.to_variant());
				node.set_material(&mat);

				let mut dependent_meshes: VarArray;
				if !mask.has_meta("meshes") {
					mask.set_meta("meshes", &VarArray::new().to_variant());
				}
				dependent_meshes = mask.get_meta("meshes").to::<VarArray>();
				dependent_meshes.push(&format!("Meshes/{}", node.get_name()).to_node_path());

				mask.set_meta("meshes", &dependent_meshes.to_variant());
			}
		}

		drop(m_scene);

		// make sure all nodes are persisted within the scene
		for child in meshes.values_mut() {
			child.set_owner(&scene);
		}
		for child in masks.values_mut() {
			child.set_owner(&scene);
			for mut m in child.get_children().iter_shared() {
				m.set_owner(&scene);
			}
		}

		scene.bind_mut().set_size(Vector2i {
			x: canvas_size.x as i32,
			y: canvas_size.y as i32,
		});

		let mut animation_library = AnimationLibrary::new_gd();
		let reset = self.create_reset_motion(&scene);
		animation_library.add_animation("RESET", &reset);
		motion_controller.add_animation_library("", &animation_library);

		scene
	}

	#[func]
	pub fn load_motion(&self, file_path: GString) -> Option<Gd<Animation>> {
		let json = FileAccess::get_file_as_string(&file_path);
		let motion: VarDictionary = Json::parse_string(&json).to();

		let meta = motion.at("Meta").to::<VarDictionary>();
		let mut anim = Animation::new_gd();

		let loop_mode = if meta.get("Loop").map_or(false, |v| v.to()) { LoopMode::LINEAR } else { LoopMode:: NONE };
		anim.set_loop_mode(loop_mode);
		
		let fps: f32 = meta.get("FPS").map_or(60.0, |v| v.to());
		anim.set_step(1.0 / fps);

		let mut last_frame: f64 = 0.0;

		// parse motion curves
		let curves: VarArray = motion.at("Curves").to();
		for curve in curves.iter_shared().map(|v| v.to::<VarDictionary>()) {
			let property: GString = curve.at("Id").to();
			let segments: VarArray = curve.at("Segments").to();

			if segments.len() < 2 {
				godot_error!("Invalid Segment, must have at least one point");
				return None;
			}

			let track = anim.add_track(TrackType::BEZIER);

			let target_type = match curve.at("Target").to_string().as_str() {
				"Parameter" => "parameters",
				"PartOpacity" => "parts",
				_ => "",
			};

			anim.track_set_path(track, &format![".:{}/{}", target_type, property].to_node_path());
			anim.track_set_interpolation_loop_wrap(track, false);

			// first key is always the starting time and value
			anim.bezier_track_insert_key(track, segments.at(0).to(), segments.at(1).to());

			let mut last_key = anim.track_get_key_count(track) - 1;
			let mut s_idx = 2;
			loop {
				let seg_type: i32 = segments.at(s_idx).to::<f32>() as i32;

				match seg_type {
					// LINEAR
					0 => {				
						let p0_t: f64 = segments.at(s_idx - 2).to();
						let p0_v: f32 = segments.at(s_idx - 1).to();
						let p1_t: f64 = segments.at(s_idx + 1).to();
						let p1_v: f32 = segments.at(s_idx + 2).to();

						// tangents
						let out_t = Vector2 { x: (p1_t - p0_t) as f32, y: p1_v - p0_v };
						let in_t = out_t * Vector2 { x: -1.0, y: 1.0};

						anim.bezier_track_set_key_out_handle(track, last_key, out_t);

						last_key = anim.bezier_track_insert_key(track, p1_t, p1_v);
						anim.bezier_track_set_key_in_handle(track, last_key, in_t);

						s_idx += 3;
					},
					// CUBIC BEZIER
					1 => {										
						let p0_t: f64 = segments.at(s_idx - 2).to();
						let p0_v: f32 = segments.at(s_idx - 1).to();
						let p1_t: f64 = segments.at(s_idx + 1).to();
						let p1_v: f32 = segments.at(s_idx + 2).to();
						let p2_t: f64 = segments.at(s_idx + 3).to();
						let p2_v: f32 = segments.at(s_idx + 4).to();
						let p3_t: f64 = segments.at(s_idx + 5).to();
						let p3_v: f32 = segments.at(s_idx + 6).to();

						let tangent_len: f32 = (p0_t - p3_t).abs() as f32 * 0.33333;
						let out_t = Vector2 { x: tangent_len, y: p1_v - p0_v };
						let in_t = Vector2 { x: -tangent_len, y: p3_v - p2_v };

						anim.bezier_track_set_key_out_handle(track, last_key, out_t);

						last_key = anim.bezier_track_insert_key(track, p3_t, p3_v);

						anim.bezier_track_set_key_in_handle(track, last_key, in_t);

						s_idx += 7;
					},
					// Stepped
					2 => {
						let p1_t: f64 = segments.at(s_idx + 1).to();
						let p1_v: f32 = segments.at(s_idx + 2).to();

						last_key = anim.bezier_track_insert_key(track, p1_t, p1_v);
						anim.bezier_track_set_key_in_handle(track, last_key, Vector2 { x: 0.0, y: f32::MAX });

						s_idx += 3;
					},
					// Inverse Stepped
					3 => {
						let p0_t: f64 = segments.at(s_idx - 2).to();
						let p0_v: f32 = segments.at(s_idx - 1).to();
						let p1_t: f64 = segments.at(s_idx + 1).to();
						let p1_v: f32 = segments.at(s_idx + 2).to();

						let out_t = Vector2 { x: (p1_t - p0_t) as f32, y: p1_v - p0_v };

						anim.bezier_track_set_key_out_handle(track, last_key, out_t);

						last_key = anim.bezier_track_insert_key(track, p0_t + 0.01, p1_v);
						anim.bezier_track_set_key_in_handle(track, last_key, out_t);

						last_key = anim.bezier_track_insert_key(track, p1_t, p1_v);

						s_idx += 3;
					}
					_ => {
						godot_error!("Invalid Motion Segment Type");
						return None;
					}
				}

				last_frame = last_frame.max(anim.track_get_key_time(track, anim.track_get_key_count(track) - 1));

				if s_idx >= segments.len() {
					break;
				}
			}
		}

		let duration: f64 = meta.get("Duration").map_or(1.0, |v| v.to());
		
		anim.set_length(duration.max(last_frame) as f32);
		anim.set_path(&file_path);

		Some(anim)
	}

	#[func]
	pub fn load_motion_library(&self, model: Gd<AyagamiModel>) -> Gd<AnimationLibrary> {
		let base_path: GString = ProjectSettings::singleton().globalize_path(&model.get_meta("basepath").to::<GString>());
		let mut animation_library = AnimationLibrary::new_gd();

		for entry in glob(&format!("{}/**/*.motion3.json", base_path.to_string())).unwrap() {
			if let Ok(path) = entry {
				let gpath = path.display().to_string().to_gstring();
				let name = gpath.get_file().to_string_name();
				if let Some(animation) = self.load_motion(gpath) {
					animation_library.add_animation(&name, &animation);
				}
			}
		}

		let reset = self.create_reset_motion(&model);
		animation_library.add_animation("RESET", &reset);

		return animation_library;
	}
	
	pub fn create_reset_motion(&self, model: &Gd<AyagamiModel>) -> Gd<Animation> {
		let mut animation = Animation::new_gd();
		animation.set_name("RESET");
		animation.set_length(0.0001);

		let properties = model.get_property_list();

		for p in properties.iter_shared() {
			let name: GString = p.at("name").to();
			if name.begins_with("parameters/") || name.begins_with("parts/") {
				let value = model.get(&name.to_string_name());
				let track = animation.add_track(TrackType::VALUE);
				animation.track_set_path(track, &format!(".:{}", name));
				animation.track_insert_key(track, 0.0, &value);
			}
		}

		animation
	}

	#[func]
	pub fn load_expression(&self, file_path: GString) -> Gd<AyagamiExpression> {
		Gd::default()
	}
}
