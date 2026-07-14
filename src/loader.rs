use std::collections::HashMap;

use godot::classes::mesh::PrimitiveType;
use godot::classes::viewport::DefaultCanvasItemTextureFilter;
use godot::prelude::*;
use godot::classes::{
	ArrayMesh, FileAccess, Image, ImageTexture, Json, MeshInstance2D, RenderingServer, ResourceLoader, Shader, ShaderMaterial, SubViewport, Texture2D, ViewportTexture, mesh
};

use ayagami::core::{
	ArtMesh, BlendMode, Collection, Item, Model
};
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
				let img = Image::load_from_file(real_path);
				tex = ImageTexture::create_from_image(img.as_ref()).unwrap().upcast();
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

		let mut m_scene = scene.bind_mut();
		m_scene.load();
		
		let md = m_scene.model.as_mut().unwrap();
		let m = md.model.as_ref();

		// update art meshes to their initial states
		md.driver.drive(m);

		let px_size = m.canvas_properties().scale;
		let origin = Vector2 {
			x: m.canvas_properties().center.x,
			y: m.canvas_properties().center.y
		};
		let canvas_size = m.canvas_properties().dimensions;

		// make all the art meshes
		for (_i, uid) in md.driver.sorted_artmeshes().into_iter().enumerate() {
			let artmesh = md.model.artmeshes().get(*uid).unwrap();
			// TODO get mesh state when parameters are at defaults
			// let raw_mesh = md.driver.artmesh_state(artmesh.uid());
			let mut mesh = ArrayMesh::new_gd();
			let id = artmesh.id().to_string_name();
			let uid = artmesh.uid().clone();
			mesh.set_local_to_scene(true);

			let vtx_count = artmesh.vertex_count();
			let am = md.driver.artmesh_state(uid).unwrap();
					
			if vtx_count > 0 {
				let mut ary = VarArray::new();
				ary.resize(mesh::ArrayType::MAX.ord() as usize, &Variant::nil());

				// vertices
				{
					let mut vary = PackedVector2Array::new();
					vary.resize(vtx_count as usize);

					for i in 0..am.vertices.len() {
						vary[i] = Vector2 {
							x: am.vertices[i].x * px_size,
							y: am.vertices[i].y * px_size
						} + origin;
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
			}

			let mut mesh_instance = MeshInstance2D::new_alloc();
			mesh_instance.set_name(&id);
			mesh_instance.set_mesh(&mesh);
			mesh_instance.set_meta("uid", &uid.to_variant());
			mesh_instance.set_visible(artmesh.visible());
			
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

		scene
	}
}
