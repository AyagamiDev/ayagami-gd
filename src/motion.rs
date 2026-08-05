use godot::{classes::{AnimationPlayer, IAnimationPlayer, notify::NodeNotification}, meta::ClassId, prelude::*, register::info::{PropertyHintInfo, PropertyInfo, PropertyUsageFlags}};

use crate::{model::{AyagamiModel, PARAMETER_PREFIX, PART_PREFIX}, mutator::{IMutator, Parts, Pose}};

#[derive(GodotClass)]
#[class(tool, init, base = AnimationPlayer)]
pub struct AyagamiMotionMutator {
	base: Base<AnimationPlayer>,

    #[export]
    parameters: Pose,
    part_opacities: Parts,
}

#[godot_api]
impl AyagamiMotionMutator {
    #[func]
    pub fn reset(&mut self) {
        self.parameters.clear();
        self.part_opacities.clear();
    }

    fn reset_hook(&mut self, _anim: StringName) {
        self.reset();
    }
}

#[godot_dyn]
impl IMutator for AyagamiMotionMutator {
	fn apply(&mut self, mut pose: Pose, mut parts: Parts) {
        if self.base().get_current_animation() == StringName::default() {
            return;
        }
        
        pose.extend_dictionary(&self.parameters, true);
        parts.extend_dictionary(&self.part_opacities, true);
	}
}

#[godot_api]
impl IAnimationPlayer for AyagamiMotionMutator {
    fn on_notification(&mut self, notification: NodeNotification) {
        if notification == NodeNotification::READY {
            self.signals()
                .animation_started()
                .connect_self(Self::reset_hook);

            self.signals()
                .current_animation_changed()
                .connect_self(Self::reset_hook);

            self.signals()
                .animation_finished()
                .connect_self(Self::reset_hook);
        }
    }

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