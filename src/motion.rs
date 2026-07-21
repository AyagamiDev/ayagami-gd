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

	fn on_property_get_revert(&self, property: StringName) -> Option<Variant> {
		return None;
	}
}