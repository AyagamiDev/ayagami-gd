use godot::{classes::{AnimationPlayer, IAnimationPlayer, notify::NodeNotification}, meta::ClassId, prelude::*, register::info::{PropertyHintInfo, PropertyInfo, PropertyUsageFlags}};

use crate::{model::{AyagamiModel, PARAMETER_PREFIX, PART_PREFIX, key_param, key_part}, mutator::IMutator};
use ayagami::pose::{Key, Pose};

#[derive(GodotClass)]
#[class(tool, init, base = AnimationPlayer)]
pub struct AyagamiMotionMutator {
	base: Base<AnimationPlayer>,

    // internal pose based on parent node's pose map
    // behavior may become unstable if this node is moved in the scene tree
    // after it has been initialized
    pose: Option<Pose>,
}

#[godot_api]
impl AyagamiMotionMutator {
    #[func]
    pub fn reset(&mut self) {
        if let Some(parent) = self.base().get_parent() {
            if let Ok(model) = parent.try_cast::<AyagamiModel>() {
                self.pose = Some(Pose::with_map(model.bind().pose_map.clone()));
            }
        }
        else {
            self.pose = None
        };
    }

    fn reset_hook(&mut self, _anim: StringName) {
        self.reset();
    }

    fn loaded_hook(&mut self) {
        self.reset();
    }
}

#[godot_dyn]
impl IMutator for AyagamiMotionMutator {
	fn apply(&mut self, pose: &mut Pose) {
        if self.base().get_current_animation() != StringName::default() {
            if let Some(p) = &self.pose {
                pose.update(p);
            }
        }
	}
}

#[godot_api]
impl IAnimationPlayer for AyagamiMotionMutator {
    fn on_notification(&mut self, notification: NodeNotification) {
        if notification == NodeNotification::READY {
            // if the parent is already initialized, it's safe to load the pose model from it
            self.reset();
            
            self.signals()
                .animation_started()
                .connect_self(Self::reset_hook);

            self.signals()
                .current_animation_changed()
                .connect_self(Self::reset_hook);

            self.signals()
                .animation_finished()
                .connect_self(Self::reset_hook);

            // if the model isn't already loaded, wait for the signal to be able to reset to an accurate pose map
            if let Some(parent) = self.base().get_parent() {
                if let Ok(model) = parent.try_cast::<AyagamiModel>() {
                    model.signals()
                        .loaded()
                        .connect_other(self, AyagamiMotionMutator::loaded_hook);
                }
            }
        }
    }

    fn on_set(&mut self, property: StringName, value: Variant) -> bool {
        if let Some(pose) = self.pose.as_mut() {
            // check if attempting to set a value on the internal ayagami driver
            if property.begins_with(PARAMETER_PREFIX) {
                let key = key_param(property);
                if let Ok(v) = value.try_to::<f32>() {
                    pose.set_or_add(key, v);
                    return true;
                }
            }
            else if property.begins_with(PART_PREFIX) {
                let key = key_part(property);
                if let Ok(v) = value.try_to::<f32>() {
                    pose.set_or_add(key, v);
                    return true;
                }
            }
        }
		
		return false;
	}

	fn on_get(&self, property: StringName) -> Option<Variant> {
        if let Some(parent) = self.base().get_parent() {
            if let Some(pose) = &self.pose {
                if property.begins_with(PARAMETER_PREFIX) {
                    let key = key_param(property.clone());
                    let maybe_value = parent.get(&property);
                    return pose.get(&key)
                        .map(|v| v.value.to_variant())
                        .or((!maybe_value.is_nil()).then_some(maybe_value));
                }

                if property.begins_with(PART_PREFIX) {
                    let key = key_part(property.clone());
                    let maybe_value = parent.get(&property);
                    return pose.get(&key)
                        .map(|v| v.value.to_variant())
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
                    |(k, _)| match k {
                        Key::Param(property) => PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: format!("{}{}", PARAMETER_PREFIX, property).to_string_name(),
                            hint_info: PropertyHintInfo::none(),
                            usage: PropertyUsageFlags::EDITOR
                        },
                        Key::Part(property) => PropertyInfo {
                            variant_type: VariantType::FLOAT,
                            class_name: ClassId::none().to_string_name(),
                            property_name: format!("{}{}", PART_PREFIX, property).to_string_name(),
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