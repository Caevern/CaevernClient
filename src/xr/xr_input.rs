use openxr::{Action, ActionSet, ActiveActionSet, Binding, Instance, Path, Session, Vector2f};

use crate::renderer::input_state::InputState;

pub struct XrInput {
    pub action_set: ActionSet,
    pub move_action: Action<Vector2f>,
    pub menu_action: Action<bool>,
}

impl XrInput {
    pub fn new(instance: &Instance) -> openxr::Result<Self> {
        let action_set = instance.create_action_set("gameplay", "Gameplay Inputs", 0)?;
        let move_action = action_set.create_action::<Vector2f>("move", "Player Movement", &[])?;

        let menu_action = action_set.create_action::<bool>("menu", "Menu", &[])?;

        let oculus_profile =
            instance.string_to_path("/interaction_profiles/oculus/touch_controller")?;
        let oculus_left_stick = instance.string_to_path("/user/hand/left/input/thumbstick")?;
        let oculus_menu_button = instance.string_to_path("/user/hand/left/input/menu/click")?;

        instance.suggest_interaction_profile_bindings(
            oculus_profile,
            &[
                Binding::new(&move_action, oculus_left_stick),
                Binding::new(&menu_action, oculus_menu_button),
            ],
        )?;

        let index_profile =
            instance.string_to_path("/interaction_profiles/valve/index_controller")?;
        let index_left_stick = instance.string_to_path("/user/hand/left/input/thumbstick")?;
        let index_b_button = instance.string_to_path("/user/hand/left/input/b/click")?;

        instance.suggest_interaction_profile_bindings(
            index_profile,
            &[
                Binding::new(&move_action, index_left_stick),
                Binding::new(&menu_action, index_b_button),
            ],
        )?;

        let vive_profile = instance.string_to_path("/interaction_profiles/htc/vive_controller")?;
        let vive_left_trackpad = instance.string_to_path("/user/hand/left/input/trackpad")?;
        let vive_menu_button = instance.string_to_path("/user/hand/left/input/system/click")?;

        instance.suggest_interaction_profile_bindings(
            vive_profile,
            &[
                Binding::new(&move_action, vive_left_trackpad),
                Binding::new(&menu_action, vive_menu_button),
            ],
        )?;

        let simple_profile =
            instance.string_to_path("/interaction_profiles/khr/simple_controller")?;
        let simple_menu = instance.string_to_path("/user/hand/left/input/menu/click")?;

        instance.suggest_interaction_profile_bindings(
            simple_profile,
            &[Binding::new(&menu_action, simple_menu)],
        )?;

        Ok(Self {
            action_set,
            move_action,
            menu_action,
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

    let menu_state = xr_input.menu_action.state(session, Path::NULL)?;
    if menu_state.is_active {
        input.menu = menu_state.current_state;
        //println!("menu: menu={}", input.menu);
    }

    Ok(())
}
