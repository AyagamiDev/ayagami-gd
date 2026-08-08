use godot::prelude::*;
use godot::meta::ClassId;
use godot::register::info::{PropertyHintInfo, PropertyInfo, PropertyUsageFlags};

use crate::model::{AyagamiModel, PARAMETER_PREFIX, PART_PREFIX, key_param, key_part, param_to_key};
use ayagami::pose::{Key, Pose, Value};

pub trait IMutator {
	fn apply(&mut self, _pose: &mut Pose);
}

#[derive(GodotClass)]
#[class(tool, no_init, base = RefCounted)]
pub struct AyagamiPose {
    pose: Pose
}

#[godot_api]
impl AyagamiPose {
    #[func]
    fn blend(&mut self, property: StringName, value: f32, #[opt(default = 1.0)] weight: f32) {
        if let Some(k) = param_to_key(property).as_ref() {
            if let Some(p) = self.pose.get_mut_flattened(k) {
                *p = p.blend(&Value::opaque(value), weight)
            }
        }
    }

    #[func]
    fn add(&mut self, property: StringName, value: f32, #[opt(default = 1.0)] weight: f32) {
        if let Some(k) = param_to_key(property).as_ref() {
            if let Some(p) = self.pose.get_mut_flattened(k) {
                *p = p.add(&Value::opaque(value), weight)
            }
        }
    }

    #[func]
    fn multiply(&mut self, property: StringName, value: f32, #[opt(default = 1.0)] weight: f32) {
        if let Some(k) = param_to_key(property).as_ref() {
            if let Some(p) = self.pose.get_mut_flattened(k) {
                *p = p.multiply(&Value::opaque(value), weight)
            }
        }
    }
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiMutator {
	base: Base<Node>,
    #[export(range = (0.0, 1.0))]
    #[init(val = 1.0)]
    weight: f32
}

#[godot_dyn]
impl IMutator for AyagamiMutator {
	fn apply(&mut self, pose: &mut Pose) {
        let data: Gd<AyagamiPose> = Gd::from_object(AyagamiPose { pose: pose.clone() });
        <Self>::apply(self, data.clone());
        pose.blend(&data.bind().pose, self.weight);
	}
}

// Script class for making mutators
// because Ayagami Poses are not serializable as Variants to be exposed to the GDScript API
// we instead pass through a Dictionary following the standard property naming pattern on models
// to mutate the pose in the chain
#[godot_api]
impl AyagamiMutator {
	#[func(virtual)]
	fn apply(&mut self, mut _pose: Gd<AyagamiPose>) {
		
	}
}

#[derive(GodotClass)]
#[class(tool, init, base = Node)]
pub struct AyagamiOverrideMutator {
	base: Base<Node>,

    #[export]
    pub weight: f32,
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
        for (k, v) in self.parameters.iter_shared() {
            if let Some(p) = pose.get_mut_flattened(&key_param(k)) {
                *p = p.blend(&Value::opaque(v), self.weight);
            }
        }
        for (k, v) in self.part_opacities.iter_shared() {
            if let Some(p) = pose.get_mut_flattened(&key_part(k)) {
                *p = p.blend(&Value::opaque(v), self.weight);
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
                if let Some(k) = param_to_key(property).as_ref() {
                    if let Some(pose) = model.bind().pose.as_ref() {
                        if let Some(v) = pose.get_flattened(k) {
                            return Some(v.to_variant());
                        }
                    }
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
