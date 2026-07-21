use godot::meta::ClassId;
use godot::prelude::*;
use godot::register::info::{PropertyHint, PropertyHintInfo, PropertyInfo, PropertyUsageFlags};

use crate::expression::BlendMode::{MULTIPLY, OVERRIDE};
use crate::mutator::{IMutator, Parts, Pose};

const ACTIVE_PREFIX: &str = "expressions/";
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

	expression_grouping: Dictionary<StringName, StringName>,
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

	#[func]
	pub fn reset(&mut self) {
		self.weight.clear();
	}

	#[func]
	pub fn reset_group(&mut self, group_name: StringName) {
		// don't allow resetting the default empty group
		if group_name.is_empty() {
			return;
		}

		for (e, _) in self.expression_grouping
			.iter_shared()
			.filter(|(_, group)| group == &group_name) {
			self.weight.erase(&e);
		}
	}

	#[func]
	pub fn get_expression_groups(&self) -> Vec<StringName> {
		self.expression_grouping.values_array().iter_shared().fold(
			Vec::new(),
			|mut acc, v| {
				if !acc.contains(&v) {
					acc.push(v);
				}
				acc
			}
		)
	}

	fn toggle_expression(&mut self, expression_name: StringName, on: bool) {
		if on {
			// make sure only one expression for a group is active at a time
			if let Some(group) = self.expression_grouping.get(&expression_name) {
				self.reset_group(group);
			}
		}
		
		self.weight.set(&expression_name, if on { 1.0 } else { 0.0 });
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
			return self.expression_grouping.get(&name)
				.or(Some("".to_string_name()))
				.map(|v| v.to_variant());
		}

		if parameter.begins_with(ACTIVE_PREFIX) {
			let name = parameter.trim_prefix(ACTIVE_PREFIX).to_string_name();
			return self.weight.get(&name)
				.or(Some(0.0))
				.map(|v| (v > 0.0).to_variant())
		}

		return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
		let mut custom_params: Vec<PropertyInfo> = Vec::new();

		self.expressions.iter_shared().for_each(
			|ex| {
				let expression_name = ex.get_name();
				custom_params.push(PropertyInfo {
					variant_type: VariantType::BOOL,
					class_name: ClassId::none().to_string_name(),
					property_name: format!("{}{}", ACTIVE_PREFIX, expression_name).to_string_name(),
					hint_info: PropertyHintInfo::none(),
					usage: PropertyUsageFlags::EDITOR,
				});
				custom_params.push(PropertyInfo {
					variant_type: VariantType::FLOAT,
					class_name: ClassId::none().to_string_name(),
					property_name: format!("{}{}", WEIGHT_PREFIX, expression_name).to_string_name(),
					hint_info: PropertyHintInfo {
						hint: PropertyHint::NONE,
						hint_string: "0.0,1.0".to_gstring(),
					},
					usage: PropertyUsageFlags::EDITOR,
				});
				custom_params.push(PropertyInfo {
					variant_type: VariantType::STRING,
					class_name: ClassId::none().to_string_name(),
					property_name: format!("{}{}", GROUP_PREFIX, expression_name).to_string_name(),
					hint_info: PropertyHintInfo::none(),
					usage: PropertyUsageFlags::STORAGE | PropertyUsageFlags::EDITOR,
				});
			}	
		);

		custom_params
	}

	fn on_property_get_revert(&self, property: StringName) -> Option<Variant> {
		if property.begins_with(WEIGHT_PREFIX) {
			return Some(0.0.to_variant());
		}
		if property.begins_with(ACTIVE_PREFIX) {
			return Some(false.to_variant());
		}
		if property.begins_with(GROUP_PREFIX) {
			return Some(GString::default().to_variant());
		}
		return None;
	}

	fn on_set(&mut self, property: StringName, value: Variant) -> bool {
		
		// manipulating weight values directly give you full control over how expressions
		// are applied, but bypasses grouping to keep internals simple.
		// A usecase for manipulating weights directly would be to fade between weights
		// using Tweens.  When doing so, it is necessary to replicate grouping behavior
		// in your own code
		if property.begins_with(WEIGHT_PREFIX) {
			let expression = property.trim_prefix(WEIGHT_PREFIX).to_string_name();
			let weight = value.to::<f32>().clamp(0.0, 1.0);
			self.weight.set(&expression, weight);
			return true;
		}

		// active toggles respect group exclusivity and will deactive other expressions
		if property.begins_with(ACTIVE_PREFIX) {
			let expression = property.trim_prefix(ACTIVE_PREFIX).to_string_name();
			self.toggle_expression(expression, value.booleanize());
			return true;
		}

		if property.begins_with(GROUP_PREFIX) {
			let expression = property.trim_prefix(GROUP_PREFIX).to_string_name();
			let group = value.stringify().to_string_name();
			self.expression_grouping.set(&expression, &group);
			return true;
		}

		return false;
	}
}
