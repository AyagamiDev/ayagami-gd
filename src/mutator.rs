use godot::{meta::ClassId, prelude::*, register::info::{PropertyHintInfo, PropertyInfo, PropertyUsageFlags}};

use crate::model::{AyagamiModel, PARAMETER_PREFIX, PART_PREFIX, key_param, key_part};
use ayagami::pose::{Key, Pose};

pub trait IMutator {
	fn apply(&mut self, _pose: &mut Pose);
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiMutator {
	base: Base<Node>
}

#[godot_dyn]
impl IMutator for AyagamiMutator {
	fn apply(&mut self, pose: &mut Pose) {
        let parameters = pose.iter().fold(
            Dictionary::new(), 
            |mut acc: Dictionary<StringName, f32>, (k, v)| {
                match k {
                    Key::Param(key_name) => {
                        acc.set(&format!("{}{}", PARAMETER_PREFIX, key_name).to_string_name(), v.value);
                    }
                    Key::Part(key_name) => {
                        acc.set(&format!("{}{}", PART_PREFIX, key_name).to_string_name(), v.value);
                    }
                }
                acc
            });
		
        <Self>::apply(self, parameters.clone());

        for (k,v) in parameters.iter_shared() {
            if k.begins_with(PARAMETER_PREFIX) {
                pose.set(&key_param(k),v);
            }
            else if k.begins_with(PARAMETER_PREFIX) {
                pose.set(&key_part(k), v);
            }
        }
	}
}

// Script class for making mutators
// because Ayagami Poses are not serializable as Variants to be exposed to the GDScript API
// we instead pass through a Dictionary following the standard property naming pattern on models
// to mutate the pose in the chain
#[godot_api]
impl AyagamiMutator {
	#[func(virtual)]
	fn apply(&mut self, mut _pose: Dictionary<StringName, f32>) {
		
	}
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiOverrideMutator {
	base: Base<Node>,

    #[export]
    pub enabled: bool,
    parameters: Dictionary<StringName, f32>,
    part_opacities: Dictionary<StringName, f32>,
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
	fn apply(&mut self, pose: &mut Pose) {
        if self.enabled {
            for (k, v) in self.parameters.iter_shared() {
                pose.set(&key_param(k), v);
            }
            for (k, v) in self.part_opacities.iter_shared() {
                pose.set(&key_part(k), v);
            }
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
        else if property.begins_with(PART_PREFIX) {
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
                    let maybe_value = model.get(&property);
                    return self.parameters.get(&property)
                        .map(|v| v.to_variant())
                        .or((!maybe_value.is_nil()).then_some(maybe_value));
                }
                else if property.begins_with(PART_PREFIX) {
                    let maybe_value = model.get(&property);
                    return self.part_opacities.get(&property)
                        .map(|v| v.to_variant())
                        .or((!maybe_value.is_nil()).then_some(maybe_value));
        		}
            }
        }

        return None;
	}

	fn on_get_property_list(&mut self) -> Vec<PropertyInfo> {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(model) = parent.clone().try_cast::<AyagamiModel>() {
                return model.bind().pose_map.iter().map(
                    |(_, property)| match property.key.clone() {
                        Key::Param(id) => PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: format!("{}{}", PARAMETER_PREFIX, id).to_string_name(),
                            hint_info: PropertyHintInfo::none(),
                            usage: PropertyUsageFlags::EDITOR
                        },
                        Key::Part(id) => PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: format!("{}{}", PART_PREFIX, id).to_string_name(),
                            hint_info: PropertyHintInfo::none(),
                            usage: PropertyUsageFlags::EDITOR
                        }
                    }
                ).collect();
            }
        }
        return Vec::default();
	}

	fn on_property_get_revert(&self, _property: StringName) -> Option<Variant> {
		return None;
	}
}
