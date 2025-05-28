use bevy::app::PluginsState;
use bevy::asset::RenderAssetUsages;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{AccumulatedMouseMotion, MouseButtonInput, MouseMotion};
use bevy::math::{DVec2, VectorSpace};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::surface_target::CustomSurfaceTarget;
use bevy::window::{PrimaryWindow, RawHandleWrapper, WindowResolution};
use smithay::backend::allocator::Format;
use smithay::backend::drm::{DrmDevice, DrmDeviceFd};
use smithay::backend::input::{
    Event, InputBackend, InputEvent, KeyState, KeyboardKeyEvent, PointerButtonEvent,
    PointerMotionEvent,
};
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::session::Session;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::udev::UdevBackend;
use smithay::input::keyboard::{FilterResult, XkbConfig};
use smithay::input::{Seat, SeatState};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{self, EventLoop, Interest, LoopHandle, PostAction};
use smithay::reexports::input::Libinput;
use smithay::reexports::rustix::fs::OFlags;
use smithay::reexports::wayland_server::protocol::{wl_pointer, wl_shm};
use smithay::reexports::wayland_server::{self, DisplayHandle};
use smithay::utils::{DeviceFd, SERIAL_COUNTER};
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::dmabuf::{DmabufFeedbackBuilder, DmabufGlobal, DmabufState};
use smithay::wayland::shm::ShmState;
use smithay::wayland::socket::ListeningSocketSource;
use smithay_drm_extras::drm_scanner::DrmScanner;
use smol_str::SmolStr;
use std::fs::File;
use std::iter;
use std::os::fd::{AsRawFd, OwnedFd};
use std::time::{Duration, Instant};

use self::handlers::ClientState;

const DEVICE_FLAGS: OFlags = OFlags::empty()
    .union(OFlags::CLOEXEC)
    .union(OFlags::NONBLOCK)
    .union(OFlags::NOCTTY)
    .union(OFlags::RDWR);

mod handlers;
mod keyboard;
mod mouse;

#[derive(Debug, Resource)]
struct DrmInfo(CustomSurfaceTarget);

#[derive(Debug, Default, DerefMut, Deref, Resource)]
struct CursorPosition(pub Vec2);

#[derive(Clone, Debug)]
enum UiEvent {
    KeyboardInput(KeyboardInput),
    MouseButtonInput(MouseButtonInput),
    MouseMotion(MouseMotion),
}

struct State {
    app: App,
    loop_handle: LoopHandle<'static, Self>,
    drm_device: DrmDevice,
    drm_scanner: DrmScanner,
    display_handle: DisplayHandle,
    seat: Seat<Self>,
    seat_state: SeatState<Self>,
    ui_events: Vec<UiEvent>,
    compositor_state: CompositorState,
    dmabuf_global: DmabufGlobal,
    dmabuf_state: DmabufState,
    shm_state: ShmState,
}

impl State {
    fn world(&self) -> &World {
        self.app.world()
    }

    fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    fn on_input_event<B: InputBackend>(event: InputEvent<B>, _metadata: &mut (), state: &mut Self) {
        match event {
            InputEvent::Keyboard { event } => {
                let keyboard = state.seat.get_keyboard().unwrap();
                let key_code = event.key_code();
                let button_state = event.state();
                let serial = SERIAL_COUNTER.next_serial();
                let time = event.time_msec();

                keyboard.input(
                    state,
                    key_code,
                    button_state,
                    serial,
                    time,
                    |state, _modifiers, keysym| {
                        let keycode = keysym.raw_code().raw();

                        let key_code = keyboard::convert_keycode(keycode);

                        let text = keysym
                            .modified_sym()
                            .key_char()
                            .map(|character| iter::once(character).collect());

                        let logical_key = text
                            .clone()
                            .map(Key::Character)
                            .unwrap_or_else(|| keyboard::convert_keycode_logical(keycode));

                        let event = KeyboardInput {
                            key_code,
                            logical_key,
                            state: keyboard::convert_state(button_state),
                            text,
                            repeat: false,
                            window: Entity::PLACEHOLDER,
                        };

                        state.ui_events.push(UiEvent::KeyboardInput(event));

                        FilterResult::Intercept(())
                    },
                );
            }
            InputEvent::PointerButton { event } => {
                let Some(event) = mouse::convert_button_input(event) else {
                    return;
                };

                state.ui_events.push(UiEvent::MouseButtonInput(event));
            }
            InputEvent::PointerMotion { event } => {
                let event = mouse::convert_motion(event);

                state.ui_events.push(UiEvent::MouseMotion(event));
            }
            _ => {}
        }
    }

    fn new(
        mut app: App,
        loop_handle: LoopHandle<'static, Self>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display = wayland_server::Display::<Self>::new()?;
        let display_handle = display.handle();

        let (mut session, session_source) = LibSeatSession::new()?;
        let seat_name = session.seat();
        let udev = UdevBackend::new(&seat_name)?;
        let mut input = Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));

        input
            .udev_assign_seat(&seat_name)
            .expect("udev assign seat");

        let backend = LibinputInputBackend::new(input.clone());
        let source = ListeningSocketSource::new_auto()?;

        for (_device_id, path) in udev.device_list() {
            session.open(
                path,
                OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
            )?;
        }

        loop_handle.insert_source(session_source, |event, metadata, state| {
            info!("{event:?} {metadata:?}");
        })?;

        loop_handle.insert_source(udev, |event, metadata, state| {
            info!("{event:?} {metadata:?}")
        })?;

        loop_handle.insert_source(backend, Self::on_input_event)?;

        loop_handle.insert_source(source, |client_stream, metadata, state| {
            info!("{client_stream:?} {metadata:?}");

            state
                .display_handle
                .insert_client(
                    dbg!(client_stream),
                    std::sync::Arc::new(ClientState::default()),
                )
                .expect("new client");
        })?;

        loop_handle.insert_source(
            Generic::new(
                display,
                Interest::READ,
                smithay::reexports::calloop::Mode::Level,
            ),
            |_, display, data| {
                info!("dispatch");

                unsafe {
                    display.get_mut().dispatch_clients(data).unwrap();
                }

                Ok(PostAction::Continue)
            },
        )?;

        let device_fd = File::open("/dev/dri/card1")
            .map(OwnedFd::from)
            .map(DeviceFd::from)
            .map(DrmDeviceFd::new)?;

        // DrmDeviceFd is a ref counted wrapper over the file descriptor.
        let (drm_device, _drm_source) = DrmDevice::new(device_fd.clone(), false)?;
        let mut drm_scanner: DrmScanner = DrmScanner::new();

        drm_scanner.scan_connectors(&drm_device)?;

        // todo: handle no monitors by re-creating things or something, probably?
        let (connector, info) = drm_scanner
            .connectors()
            .iter()
            .find(|(_connector, info)| {
                info.state() == smithay::reexports::drm::control::connector::State::Connected
            })
            .expect("at least one connected connector");

        // todo: enable user to override/pick custom mode
        let mode = info
            .modes()
            .iter()
            .find(|mode| {
                mode.mode_type()
                    .contains(smithay::reexports::drm::control::ModeTypeFlags::PREFERRED)
            })
            .expect("at least one mode");

        let crtc = drm_scanner
            .crtc_for_connector(connector)
            .expect("crtc for the connector");

        let planes = drm_device.planes(&crtc).expect("planes for the crtc");

        drm_device
            .claim_plane(planes.primary[0].handle, crtc)
            .expect("grrr");

        let fd = device_fd.as_raw_fd();
        let plane: u32 = planes.primary[0].handle.into();
        let connector_id: u32 = (*connector).into();
        let size = mode.size();
        let width: u32 = size.0.into();
        let height: u32 = size.1.into();
        let refresh_rate = 99982;

        app.insert_resource(DrmInfo(CustomSurfaceTarget::Drm {
            fd,
            plane,
            connector_id,
            width,
            height,
            refresh_rate,
        }));

        let compositor_state = CompositorState::new::<Self>(&display_handle);

        let mut dmabuf_state = DmabufState::new();
        let default_feedback =
            DmabufFeedbackBuilder::new(drm_device.device_id(), None::<Format>).build()?;

        // And create the dmabuf global.
        let dmabuf_global = dmabuf_state
            .create_global_with_default_feedback::<State>(&display_handle, &default_feedback);

        let shm_state = ShmState::new::<Self>(&display_handle, None::<wl_shm::Format>);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_seat(&seat_name);

        seat.add_pointer();
        seat.add_keyboard(XkbConfig::default(), 250, 25).unwrap();

        app.insert_resource(CursorPosition(Vec2::new(
            width as f32 / 2.0,
            height as f32 / 2.0,
        )));

        Ok(Self {
            app,
            loop_handle,
            compositor_state,
            drm_device,
            drm_scanner,
            display_handle,
            seat,
            seat_state,
            ui_events: Vec::new(),

            dmabuf_global,
            dmabuf_state,
            shm_state,
        })
    }

    fn setup(&mut self) {
        let Self { app, .. } = self;

        while app.plugins_state() != PluginsState::Cleaned {
            bevy::tasks::tick_global_task_pools_on_main_thread();

            app.finish();
            app.cleanup();
        }
    }

    fn forward_events(&mut self) {
        let world = self.app.world_mut();

        for event in self.ui_events.drain(..) {
            match event {
                UiEvent::KeyboardInput(event) => {
                    world.send_event(event);
                }
                UiEvent::MouseButtonInput(event) => {
                    world.send_event(event);
                }
                UiEvent::MouseMotion(event) => {
                    world.send_event(event);
                }
            }
        }
    }

    fn update(&mut self) -> Option<AppExit> {
        self.forward_events();
        self.app.update();
        self.app.should_exit()
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .set_runner(ui_runner)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    exit_sequence,
                    update_cursor_position,
                    update_overlay,
                    move_windows,
                    update_overlay_text,
                )
                    .chain(),
                update_clock,
            ),
        )
        .run();
}

fn update_cursor_position(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cursor_position: ResMut<CursorPosition>,
    mut primary_window: Single<&mut bevy::window::Window, With<PrimaryWindow>>,
) {
    **cursor_position = (**cursor_position + mouse_motion.delta)
        .clamp(Vec2::ZERO, primary_window.resolution.size());

    primary_window.set_cursor_position(Some(**cursor_position));
}

fn exit_sequence(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit_writer: EventWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_writer.write(AppExit::Success);
    }
}

#[derive(Clone, Component, Copy, Debug)]
pub struct Window;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl WindowResizeCorner {
    pub fn from_center(window_center: Vec2, cursor_position: Vec2) -> Self {
        let [is_right, is_bottom] = cursor_position.cmpgt(window_center).into();

        match (is_bottom, is_right) {
            (false, false) => WindowResizeCorner::TopLeft,
            (false, true) => WindowResizeCorner::TopRight,
            (true, false) => WindowResizeCorner::BottomLeft,
            (true, true) => WindowResizeCorner::BottomRight,
        }
    }

    pub fn resize(self, rect: &mut Rect, distance: Vec2, min_window_size: Vec2) {
        match self {
            Self::TopLeft => {
                rect.min += distance;

                rect.min = rect.min.min(rect.max - min_window_size);
            }
            Self::TopRight => {
                rect.min.y += distance.y;
                rect.max.x += distance.x;

                rect.min.y = rect.min.y.min(rect.max.y - min_window_size.y);
                rect.max.x = rect.max.x.max(rect.min.x + min_window_size.x);
            }
            Self::BottomLeft => {
                rect.min.x += distance.x;
                rect.max.y += distance.y;

                rect.min.x = rect.min.x.min(rect.max.x - min_window_size.x);
                rect.max.y = rect.max.y.max(rect.min.y + min_window_size.y);
            }
            Self::BottomRight => {
                rect.max += distance;

                rect.max = rect.max.max(rect.min + min_window_size);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowDragMode {
    Move,
    Resize(WindowResizeCorner),
}

#[derive(Clone, Component, Copy, Debug)]
pub struct WindowDragStart {
    pub cursor_position: Vec2,
    pub window_rect: Rect,
    pub mode: WindowDragMode,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let roboto_regular =
        TextFont::from_font(asset_server.load("fonts/Roboto-Regular.ttf")).with_font_size(24.0);

    let roboto_semibold =
        TextFont::from_font(asset_server.load("fonts/Roboto-SemiBold.ttf")).with_font_size(24.0);

    let roboto_mono_semibold =
        TextFont::from_font(asset_server.load("fonts/RobotoMono-Regular.ttf")).with_font_size(16.0);

    let size = Extent3d {
        width: 3840,
        height: 2160,
        depth_or_array_layers: 1,
    };

    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0xFF, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let video = images.add(image);

    commands.spawn((Camera::default(), Camera3d::default()));

    commands.spawn((
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(16.0)),
            grid_template_rows: vec![RepeatedGridTrack::px(1, 32.0), RepeatedGridTrack::auto(1)],
            ..default()
        },
        children![
            (
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::auto(3),
                    ..default()
                },
                children![
                    (
                        Text::new("Terminal"),
                        TextColor(Color::BLACK.lighter(0.2)),
                        roboto_regular.clone().with_font_size(16.0),
                    ),
                    (
                        Text::new("11/04/2025 17:21:31"),
                        TextColor(Color::BLACK.lighter(0.2)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        roboto_regular.clone().with_font_size(16.0),
                        Clock,
                    ),
                    (
                        Text::new("??"),
                        TextColor(Color::BLACK.lighter(0.2)),
                        roboto_regular.clone().with_font_size(16.0),
                        TextLayout::new_with_justify(JustifyText::Right),
                    ),
                ],
            ),
            (
                BorderRadius::all(Val::Px(16.0)),
                BorderColor(Color::BLACK.lighter(0.05)),
                Node {
                    border: UiRect::all(Val::Px(2.0)),
                    display: Display::Grid,
                    ..default()
                },
                children![(
                    BorderRadius::all(Val::Px(16.0)),
                    Node {
                        width: Val::Px(3440.0 - ((16.0 + 2.0) * 2.0)),
                        height: Val::Px(1440.0 - ((16.0 + 2.0) * 2.0) - 32.0),
                        overflow: Overflow::hidden(),
                        overflow_clip_margin: OverflowClipMargin::border_box(),
                        ..default()
                    },
                    ImageNode::new(asset_server.load("wallpaper.png")),
                )],
            )
        ],
        ZIndex(-1),
    ));

    commands.spawn((
        BackgroundColor(Color::BLACK.lighter(0.001)),
        BorderRadius::all(Val::Px(16.0)),
        BorderColor(Color::BLACK.lighter(0.05)),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            left: Val::Px(16.0),
            top: Val::Px(16.0),
            width: Val::Px(400.0),
            height: Val::Px(300.0),
            border: UiRect::all(Val::Px(2.0)),
            margin: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        Window,
        children![
            (
                BorderColor(Color::BLACK.lighter(0.05)),
                Node {
                    border: UiRect::bottom(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(16.0)),
                    overflow: Overflow::hidden(),
                    ..default()
                },
                children![(
                    Text::new("Terminal"),
                    TextColor(Color::BLACK.lighter(0.2)),
                    roboto_regular.clone().with_font_size(16.0),
                )],
            ),
            (
                Node {
                    display: Display::Grid,
                    padding: UiRect::all(Val::Px(16.0)),
                    overflow: Overflow::hidden(),
                    ..default()
                },
                children![

                    (

                        Text::new(
                            "> id\nuid=1000(mari) gid=1000(mari) groups=1000(mari),992(kvm),998(wheel)\n"
                        ),
                        TextColor(Color::BLACK.lighter(0.2)),
                        roboto_mono_semibold.clone()
                    )
                ],
            )
        ],
    ));

    commands.spawn((
        BackgroundColor(Color::BLACK.lighter(0.001)),
        BorderRadius::all(Val::Px(16.0)),
        BorderColor(Color::BLACK.lighter(0.05)),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            left: Val::Px(500.0),
            top: Val::Px(500.0),
            width: Val::Px(400.0),
            height: Val::Px(300.0),
            border: UiRect::all(Val::Px(2.0)),
            margin: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        Window,
        children![
            (
                BorderColor(Color::BLACK.lighter(0.05)),
                Node {
                    border: UiRect::bottom(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(16.0)),
                    overflow: Overflow::hidden(),
                    ..default()
                },
                children![(
                    Text::new("Terminal"),
                    TextColor(Color::BLACK.lighter(0.2)),
                    roboto_regular.with_font_size(16.0),
                )],
            ),
            (
                Node {
                    display: Display::Grid,
                    padding: UiRect::all(Val::Px(16.0)),
                    overflow: Overflow::hidden(),
                    ..default()
                },
                children![ImageNode::new(video)],
            )
        ],
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            ..default()
        },
        Text::new("16,16 400x300"),
        TextColor(Color::BLACK.lighter(0.2)),
        roboto_semibold,
        ZIndex(i32::MAX),
        Overlay,
    ));
}

#[derive(Clone, Component, Debug)]
pub struct Overlay;

#[derive(Clone, Component, Debug)]
pub struct Clock;

fn update_clock(mut query: Query<&mut Text, With<Clock>>) {
    use time::macros::format_description;

    let fmt = format_description!("[day]/[month]/[year] [hour]:[minute]:[second]");

    let time =
        time::OffsetDateTime::now_local().unwrap_or_else(|_error| time::OffsetDateTime::now_utc());

    let time = time.format(&fmt).unwrap();

    for mut query in query.iter_mut() {
        **query = time.clone();
    }
}

fn update_overlay(
    overlay: Single<(&ComputedNode, &mut Node), With<Overlay>>,
    primary_window: Single<&bevy::window::Window, With<PrimaryWindow>>,
) {
    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let (computed_node, mut node) = overlay.into_inner();

    let Node {
        left: Val::Px(x),
        top: Val::Px(y),
        ..
    } = &mut *node
    else {
        return;
    };

    let size = computed_node.size();

    *x = cursor_position.x - (size.x / 2.0);
    *y = cursor_position.y - size.y;
}

fn update_overlay_text(
    windows: Query<&Node, (Changed<Node>, With<Window>)>,
    mut overlay: Single<&mut Text, With<Overlay>>,
) {
    for node in windows.iter() {
        let Node {
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(width),
            height: Val::Px(height),
            ..
        } = node
        else {
            return;
        };

        ***overlay = format!(
            "{},{} {}x{}",
            x.round(),
            y.round(),
            width.round(),
            height.round()
        );
    }
}

#[derive(Clone, Component, Copy, Debug)]
pub struct Focused;

fn move_windows(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut windows: Query<(Entity, &mut Node, Option<&WindowDragStart>), With<Window>>,
    primary_window: Single<&bevy::window::Window, With<PrimaryWindow>>,
) {
    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let min_window_size = Vec2::new(100.0, 110.0);

    for (entity, mut node, drag_start) in windows.iter_mut() {
        let Node {
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(width),
            height: Val::Px(height),
            ..
        } = node.bypass_change_detection()
        else {
            return;
        };

        let position = Vec2::new(*x, *y);
        let size = Vec2::new(*width, *height);

        let original_rect = Rect::from_corners(position, position + size);
        let mut rect = original_rect;

        if let Some(drag_start) = drag_start {
            let distance = cursor_position - drag_start.cursor_position;

            rect = drag_start.window_rect;

            match drag_start.mode {
                WindowDragMode::Move => {
                    if mouse.just_released(MouseButton::Left) {
                        commands.entity(entity).remove::<WindowDragStart>();
                    }

                    rect.min += distance;
                    rect.max += distance;
                }
                WindowDragMode::Resize(resize_corner) => {
                    if mouse.just_released(MouseButton::Right) {
                        commands.entity(entity).remove::<WindowDragStart>();
                    }

                    resize_corner.resize(&mut rect, distance, min_window_size);
                }
            }
        } else if mouse.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
            let mode = if mouse.just_pressed(MouseButton::Left) {
                WindowDragMode::Move
            } else if mouse.just_pressed(MouseButton::Right) {
                let window_center = rect.center();

                WindowDragMode::Resize(WindowResizeCorner::from_center(
                    window_center,
                    cursor_position,
                ))
            } else {
                return;
            };

            commands.entity(entity).insert(WindowDragStart {
                cursor_position,
                window_rect: rect,
                mode,
            });

            return;
        }

        if original_rect != rect {
            let position = rect.min;
            let size = rect.size();

            *x = position.x;
            *y = position.y;

            *width = size.x;
            *height = size.y;

            node.set_changed();
        }
    }
}

fn ui_runner(app: App) -> AppExit {
    let mut event_loop = EventLoop::try_new().expect("new loop");
    let mut state = State::new(app, event_loop.handle()).unwrap();

    state.setup();

    {
        let world = state.app.world_mut();

        let DrmInfo(target) = world.remove_resource::<DrmInfo>().unwrap();
        let CustomSurfaceTarget::Drm { width, height, .. } = &target else {
            panic!()
        };

        let mut query =
            world.query_filtered::<(Entity, &mut bevy::window::Window), With<PrimaryWindow>>();
        let (entity, mut window) = query.single_mut(world).unwrap();

        window.resolution = WindowResolution::new(*width as f32, *height as f32);
        window.set_cursor_position(Some(Vec2::new(500.0, 500.0)));

        world.entity_mut(entity).insert(target);
    }

    let mut start = Instant::now();
    let delay = Duration::from_millis(1000 / 165);

    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start);

        start = now;

        if elapsed > delay {
            if let Some(app_exit) = state.update() {
                return app_exit;
            }
        }

        event_loop
            .handle()
            .insert_source(
                calloop::timer::Timer::from_deadline(start.checked_add(delay).unwrap()),
                |a, b, c| {
                    info!("tick");

                    calloop::timer::TimeoutAction::Drop
                },
            )
            .unwrap();

        event_loop.dispatch(None, &mut state).unwrap();
    }
}
