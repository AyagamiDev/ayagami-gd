use godot::meta::ClassId;
use godot::prelude::*;
use godot::register::info::{PropertyHint, PropertyHintInfo, PropertyInfo, PropertyUsageFlags};

use crate::expression::BlendMode::{MULTIPLY, OVERRIDE};
use crate::mutator::{IMutator, Parts, Pose};

const WEIGHT_PREFIX: &str = "weight/";
pub const GROUP_PREFIX: &str = "expression_groups/";

#[derive(GodotConvert, Var, Export, Default, Clone)]
#[godot(via = GString)]
pub enum BlendMode {
	#[default]
	ADD,
	MULTIPLY,
	OVERRIDE,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct AyagamiExpressionTrack {
	#[export]
	pub property_name: StringName,
	#[export]
	pub blend_mode: BlendMode,
	#[export]
	pub amount: f32
}

#[derive(GodotClass)]
#[class(tool, init, base = Resource)]
pub struct AyagamiExpression {
	#[export]
	pub tracks: Array<Gd<AyagamiExpressionTrack>>
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiExpressionMutator {
	base: Base<Node>,

	#[export]
	pub expressions: Array<Gd<AyagamiExpression>>,

	expression_belonging: Dictionary<StringName, StringName>,
	weight: Dictionary<StringName, f32>,
}

#[godot_dyn]
impl IMutator for AyagamiExpressionMutator {
	fn apply(&mut self, mut pose: Pose, mut _parts: Parts) {
		for ex in self.expressions.iter_shared() {
			let e = ex.get_name().to_string_name();
			let weight = self.weight.get(&e).unwrap_or_default();
			for track in ex.bind().tracks.iter_shared() {
				let t = track.bind();
				if let Some(p) = pose.get(&t.property_name) {
					let target = match t.blend_mode {
						OVERRIDE => t.amount,
						MULTIPLY => p * t.amount,
						_ => p + t.amount
					};
					pose.set(
						&t.property_name, 
						p.lerp(target, weight)
					);
				}
			}
		}
	}
}

#[godot_api]
impl AyagamiExpressionMutator {
	#[func]
	pub fn is_activated(&self, expression: StringName) -> bool {
		return self.weight.get(&expression).unwrap_or(0.0) > 0.0;
	}
}

#[godot_api]
impl INode for AyagamiExpressionMutator {
	fn on_get(&self, parameter: StringName) -> Option<Variant> {
		if parameter.begins_with(WEIGHT_PREFIX) {
			let name = parameter.trim_prefix(WEIGHT_PREFIX).to_string_name();
			return self.weight.get(&name)
				.or(Some(0.0))
				.map(|v| v.to_variant());
		}

		if parameter.begins_with(GROUP_PREFIX) {
			let name = parameter.trim_prefix(GROUP_PREFIX).to_string_name();
			return self.expression_belonging.get(&name)
				.or(Some("".to_string_name()))
				.map(|v| v.to_variant());
		}

		return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
		let mut custom_params: Vec<PropertyInfo> = Vec::new();

		self.expressions.iter_shared().for_each(
			|ex| {
				let expression_name = ex.get_name();
				let expression = PropertyInfo {
					variant_type: VariantType::FLOAT,
					class_name: ClassId::none().to_string_name(),
					property_name: format!("{}{}", WEIGHT_PREFIX, expression_name).to_string_name(),
					hint_info: PropertyHintInfo {
						hint: PropertyHint::NONE,
						hint_string: "0.0,1.0".to_gstring(),
					},
					usage: PropertyUsageFlags::EDITOR,
				};
				custom_params.push(expression);

				let expression_group = PropertyInfo {
					variant_type: VariantType::STRING,
					class_name: ClassId::none().to_string_name(),
					property_name: format!("{}{}", GROUP_PREFIX, expression_name).to_string_name(),
					hint_info: PropertyHintInfo {
						hint: PropertyHint::NONE,
						hint_string: "".to_gstring(),
					},
					usage: PropertyUsageFlags::STORAGE | PropertyUsageFlags::EDITOR,
				};
				custom_params.push(expression_group);
			}	
		);

		custom_params
	}

	fn on_property_get_revert(&self, property: StringName) -> Option<Variant> {
		if property.begins_with(WEIGHT_PREFIX) {
			return Some(0.0.to_variant());
		}
		return None;
	}

	fn on_set(&mut self, property: StringName, value: Variant) -> bool {
		if property.begins_with(WEIGHT_PREFIX) {
			let expression = property.trim_prefix(WEIGHT_PREFIX).to_string_name();
			let weight = value.to::<f32>().clamp(0.0, 1.0);

			if let Some(group) = self.expression_belonging.get(&expression) {				
				// the default group should not be included
				if !group.is_empty() {
					for e in self.expression_belonging.iter_shared()
						.filter_map(
							|(name, group2)| (group == group2 && name != expression).then_some(name)
						) {
						if self.weight.get_or_insert(&e, 0.0) > 0.0 {
							self.weight.set(&e, 1.0 - weight);
						}
					}
				}
			}

			self.weight.set(&expression, weight);
			return true;
		}

		if property.begins_with(GROUP_PREFIX) {
			let expression = property.trim_prefix(GROUP_PREFIX).to_string_name();
			let group = value.stringify().to_string_name();
			self.expression_belonging.set(&expression, &group);
			return true;
		}

		return false;
	}
}
