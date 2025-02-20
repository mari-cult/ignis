use self::state::SmithayAppRunnerState;
use bevy::app::PluginsState;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use external_image::ExternalImagePlugin;
use smithay::reexports::calloop;

pub mod convert;
pub mod external_image;
pub mod state;
pub mod util;

pub(crate) use self::state::SmithayAppRunnerState as State;

pub(crate) type EventLoop = calloop::EventLoop<'static, State>;
pub(crate) type LoopHandle = calloop::LoopHandle<'static, State>;

pub struct SmithayPlugin;

impl Plugin for SmithayPlugin {
    fn build(&self, app: &mut App) {
        let event_loop = match EventLoop::try_new() {
            Ok(event_loop) => event_loop,
            Err(error) => {
                error!("failed to create event loop: {error}");

                return;
            }
        };

        let loop_handle = event_loop.handle();

        app.add_plugins(ExternalImagePlugin)
            .insert_non_send_resource(loop_handle)
            .insert_non_send_resource(event_loop)
            .set_runner(smithay_runner);
    }
}

pub fn smithay_runner(mut app: App) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let mut event_loop = app
        .world_mut()
        .remove_non_send_resource::<EventLoop>()
        .unwrap();

    app.world_mut()
        .insert_non_send_resource(event_loop.handle());

    let mut runner_state = match State::try_new(&mut event_loop, app) {
        Ok(runner_state) => runner_state,
        Err(error) => {
            error!("{error}");

            return AppExit::error();
        }
    };

    //runner_state.start_xwayland();

    runner_state.run(&mut event_loop)
}
