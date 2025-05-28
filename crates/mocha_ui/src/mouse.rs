use bevy::{
    ecs::entity::Entity,
    input::{
        ButtonState,
        mouse::{MouseButton, MouseButtonInput, MouseMotion},
    },
    math::DVec2,
};
use smithay::backend::input::{
    ButtonState as SmithayButtonState, InputBackend, MouseButton as SmithayMouseButton,
    PointerButtonEvent, PointerMotionEvent,
};

pub fn convert_motion<B: InputBackend, E: PointerMotionEvent<B>>(event: E) -> MouseMotion {
    let tuple: (f64, f64) = event.delta().into();
    let delta = DVec2::from(tuple).as_vec2();

    MouseMotion { delta }
}

pub fn convert_button_input<B: InputBackend, E: PointerButtonEvent<B>>(
    event: E,
) -> Option<MouseButtonInput> {
    let button = event
        .button()
        .and_then(convert_button)
        .or_else(|| covert_button_code(event.button_code()))?;

    let state = convert_button_state(event.state());

    Some(MouseButtonInput {
        button,
        state,
        window: Entity::PLACEHOLDER,
    })
}

pub fn convert_button(button: SmithayMouseButton) -> Option<MouseButton> {
    let button = match button {
        SmithayMouseButton::Left => MouseButton::Left,
        SmithayMouseButton::Middle => MouseButton::Middle,
        SmithayMouseButton::Right => MouseButton::Right,
        SmithayMouseButton::Forward => MouseButton::Forward,
        SmithayMouseButton::Back => MouseButton::Back,
        _ => return None,
    };

    Some(button)
}

pub fn convert_button_state(state: SmithayButtonState) -> ButtonState {
    match state {
        SmithayButtonState::Pressed => ButtonState::Pressed,
        SmithayButtonState::Released => ButtonState::Released,
    }
}

pub fn covert_button_code(code: u32) -> Option<MouseButton> {
    code.try_into().map(MouseButton::Other).ok()
}
