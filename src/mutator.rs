use godot::{meta::ClassId, prelude::*, register::info::{PropertyHintInfo, PropertyInfo, PropertyUsageFlags}};

use crate::model::{AyagamiModel, PARAMETER_PREFIX, PART_PREFIX};

pub type Pose = Dictionary<StringName, f32>;
pub type Parts = Dictionary<StringName, f32>;

pub trait IMutator {
	fn apply(&mut self, _pose: Pose, _parts: Parts);
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiMutator {
	base: Base<Node>
}

#[godot_dyn]
impl IMutator for AyagamiMutator {
	fn apply(&mut self, pose: Pose, parts: Parts) {
		<Self>::apply(self, pose, parts);
	}
}

#[godot_api]
pub impl AyagamiMutator {
	#[func(virtual)]
	fn apply(&mut self, _pose: Pose, _parts: Parts) {
		
	}
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiOverrideMutator {
	base: Base<Node>,

    #[export]
    pub enabled: bool,
    parameters: Pose,
    part_opacities: Parts,
}

#[godot_api]
impl AyagamiOverrideMutator {
    #[func]
    pub fn reset(&mut self) {
        self.parameters.clear();
        self.part_opacities.clear();
    }
}

#[godot_dyn]
impl IMutator for AyagamiOverrideMutator {
	fn apply(&mut self, mut pose: Pose, mut parts: Parts) {
        if self.enabled {
		    pose.extend_dictionary(&self.parameters, true);
            parts.extend_dictionary(&self.part_opacities, true);
        }
	}
}

#[godot_api]
impl INode for AyagamiOverrideMutator {
    fn on_set(&mut self, parameter: StringName, value: Variant) -> bool {
		// check if attempting to set a value on the internal ayagami driver
		if parameter.begins_with(PARAMETER_PREFIX) {
            self.parameters.set(&parameter, value.to::<f32>());
            return true;
		}

		if parameter.begins_with(PART_PREFIX) {
            self.part_opacities.set(&parameter, value.to::<f32>());
            return true;
		}
		
		return false;
	}

	fn on_get(&self, parameter: StringName) -> Option<Variant> {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(_) = parent.clone().try_cast::<AyagamiModel>() {
                let value = parent.clone().get(&parameter);
                let maybe_value = (!value.is_nil()).then_some(value);
                if parameter.begins_with(PARAMETER_PREFIX) {
                    return self.parameters.get(&parameter)
                        .map(|v| v.to_variant())
                        .or(maybe_value);
                }

                if parameter.begins_with(PART_PREFIX) {
                    return self.part_opacities.get(&parameter)
                        .map(|v| v.to_variant())
                        .or(maybe_value);
        		}
            }
        }

        return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(_) = parent.clone().try_cast::<AyagamiModel>() {
                return parent.clone().get_property_list().iter_shared().fold(
                    Vec::new(),
                    |mut acc, property| {
                        let name = property.at("name").stringify().to_string_name();
                        if name.begins_with(PARAMETER_PREFIX) || name.begins_with(PART_PREFIX) {
                            acc.push(PropertyInfo {
                                variant_type: VariantType::FLOAT,
                                class_name: ClassId::none().to_string_name(),
                                property_name: name,
                                hint_info: PropertyHintInfo::none(),
                                usage: PropertyUsageFlags::EDITOR
                            });
                        }
                        acc
                    }
                );
            }
        }
        return Vec::default();
	}

	fn on_property_get_revert(&self, _property: StringName) -> Option<Variant> {
		return None;
	}
}
