use openxr::{Action, ActionSet, ActiveActionSet, Binding, Instance, Path, Session, Vector2f};

use crate::renderer::input_state::InputState;

pub struct XrInput {
    pub action_set: ActionSet,
    pub move_action: Action<Vector2f>,
    pub menu_action_left: Action<bool>,
    pub menu_action_right: Action<bool>,
}

impl XrInput {
    pub fn new(instance: &Instance) -> openxr::Result<Self> {
        let action_set = instance.create_action_set("gameplay", "Gameplay Inputs", 0)?;
        let move_action = action_set.create_action::<Vector2f>("move", "Player Movement", &[])?;

        let menu_action_left = action_set.create_action::<bool>("menu-left", "Menu Left", &[])?;
        let menu_action_right = action_set.create_action::<bool>("menu-right", "Menu Right", &[])?;

        let oculus_profile =
            instance.string_to_path("/interaction_profiles/oculus/touch_controller")?;
        let oculus_left_stick = instance.string_to_path("/user/hand/left/input/thumbstick")?;
        let oculus_left_menu_button = instance.string_to_path("/user/hand/left/input/menu/click")?;

        instance.suggest_interaction_profile_bindings(
            oculus_profile,
            &[
                Binding::new(&move_action, oculus_left_stick),
                Binding::new(&menu_action_left, oculus_left_menu_button),
            ],
        )?;

        let index_profile =
            instance.string_to_path("/interaction_profiles/valve/index_controller")?;
        let index_left_stick = instance.string_to_path("/user/hand/left/input/thumbstick")?;
        let index_left_system_button = instance.string_to_path("/user/hand/left/input/system/click")?;
        let index_right_system_button = instance.string_to_path("/user/hand/right/input/system/click")?;

        instance.suggest_interaction_profile_bindings(
            index_profile,
            &[
                Binding::new(&move_action, index_left_stick),
                Binding::new(&menu_action_left, index_left_system_button),
                Binding::new(&menu_action_right, index_right_system_button),
            ],
        )?;

        let vive_profile = instance.string_to_path("/interaction_profiles/htc/vive_controller")?;
        let vive_left_trackpad = instance.string_to_path("/user/hand/left/input/trackpad")?;
        let vive_left_menu_button = instance.string_to_path("/user/hand/left/input/system/click")?;
        let vive_right_menu_button = instance.string_to_path("/user/hand/right/input/system/click")?;

        instance.suggest_interaction_profile_bindings(
            vive_profile,
            &[
                Binding::new(&move_action, vive_left_trackpad),
                Binding::new(&menu_action_left, vive_left_menu_button),
                Binding::new(&menu_action_right, vive_right_menu_button),
            ],
        )?;

        let simple_profile =
            instance.string_to_path("/interaction_profiles/khr/simple_controller")?;
        let simple_menu_left = instance.string_to_path("/user/hand/left/input/menu/click")?;

        instance.suggest_interaction_profile_bindings(
            simple_profile,
            &[
                Binding::new(&menu_action_left, simple_menu_left),
            ],
        )?;

        Ok(Self {
            action_set,
            move_action,
            menu_action_left,
            menu_action_right
        })
    }

    pub fn attach_to_session<G>(&self, session: &Session<G>) -> openxr::Result<()> {
        session.attach_action_sets(&[&self.action_set])
    }
}

pub fn poll_xr_inputs(
    session: &Session<openxr::Vulkan>,
    xr_input: &XrInput,
    input: &mut InputState,
) -> openxr::Result<()> {
    session.sync_actions(&[ActiveActionSet::new(&xr_input.action_set)])?;

    let move_state = xr_input.move_action.state(session, Path::NULL)?;
    if move_state.is_active {
        let axis = move_state.current_state;
        input.w = axis.y > 0.2;
        input.s = axis.y < -0.2;
        input.d = axis.x > 0.2;
        input.a = axis.x < -0.2;
    }

    let menu_state_left = xr_input.menu_action_left.state(session, Path::NULL)?;
    let menu_state_right = xr_input.menu_action_right.state(session, Path::NULL)?;

    input.menu = false;

    if menu_state_left.is_active {
        if menu_state_left.current_state {
            input.menu = true;
        }
    }
    if menu_state_right.is_active {
        if menu_state_right.current_state {
            input.menu = true;
        }
    }

    Ok(())
}
