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
    fn on_set(&mut self, property: StringName, value: Variant) -> bool {
		// check if attempting to set a value on the internal ayagami driver
		if property.begins_with(PARAMETER_PREFIX) {
            if let Ok(v) = value.try_to::<f32>() {
                self.parameters.set(&property, v);
                return true;
            }
		}

		if property.begins_with(PART_PREFIX) {
            if let Ok(v) = value.try_to::<f32>() {
                self.part_opacities.set(&property, v);
                return true;
            }
		}
		
		return false;
	}

	fn on_get(&self, property: StringName) -> Option<Variant> {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(model) = parent.clone().try_cast::<AyagamiModel>() {
                if property.begins_with(PARAMETER_PREFIX) {
                    let maybe_value = model.bind().parameters.get(&property).map(|v| v.to_variant());
                    return self.parameters.get(&property)
                        .map(|v| v.to_variant())
                        .or(maybe_value);
                }

                if property.begins_with(PART_PREFIX) {
                    let maybe_value = model.bind().part_opacities.get(&property).map(|v| v.to_variant());
                    return self.part_opacities.get(&property)
                        .map(|v| v.to_variant())
                        .or(maybe_value);
        		}
            }
        }

        return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(model) = parent.clone().try_cast::<AyagamiModel>() {
                let parameters = model.bind().get_parameters();
                let parameters_iter = parameters.iter_shared().map(
                    |property| {
                        let name = format!("{}{}", PARAMETER_PREFIX, property).to_string_name();
                        PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: name,
                            hint_info: PropertyHintInfo::none(),
                            usage: PropertyUsageFlags::EDITOR
                        }
                    }
                );

                let parts = model.bind().get_parts();
                let parts_iter = parts.iter_shared().map(
                    |part| {
                        let name = format!("{}{}", PART_PREFIX, part).to_string_name();
                        PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: name,
                            hint_info: PropertyHintInfo::none(),
                            usage: PropertyUsageFlags::EDITOR
                        }
                    }
                );

                return parameters_iter.chain(parts_iter).collect();
            }
        }
        return Vec::default();
	}

	fn on_property_get_revert(&self, _property: StringName) -> Option<Variant> {
		return None;
	}
}
