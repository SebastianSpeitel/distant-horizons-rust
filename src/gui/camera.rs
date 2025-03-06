use bevy::prelude::*;

const CAMERA_SPEED: f32 = 5000.;
/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().spawn((
            Camera2d,
            OrthographicProjection {
                far: 10000.,
                scale: 128.0,
                ..OrthographicProjection::default_2d()
            },
            Camera {
                hdr: true,
                ..Default::default()
            },
        ));
        app.add_systems(Update, (move_camera, zoom_camera));
    }
}

/// Update the camera position by tracking the player.
pub fn move_camera(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    kb_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cam_speed: Local<Vec2>,
) {
    if kb_input.pressed(KeyCode::KeyR) {
        camera.translation = Vec3::new(0., 0., 0.);
        *cam_speed = Vec2::ZERO;
        return;
    }

    let mut direction = Vec2::ZERO;
    if kb_input.pressed(KeyCode::KeyW) {
        direction += camera.up().truncate();
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction -= camera.up().truncate();
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction -= camera.right().truncate();
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction += camera.right().truncate();
    }

    if kb_input.pressed(KeyCode::Numpad1) {
        camera.translation = Vec3::new(-2300., -2000., 0.);
    }

    let accel = direction.normalize_or_zero() * CAMERA_SPEED;

    *cam_speed += accel * 0.5 * time.delta_secs();
    *cam_speed = cam_speed.clamp(accel.min(Vec2::ZERO), accel.max(Vec2::ZERO));

    let target = camera.translation + cam_speed.extend(0.);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());

    // camera.rotate_local_z(rotation);
}

fn zoom_camera(
    mut camera: Single<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    kb_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if kb_input.pressed(KeyCode::KeyR) {
        camera.scale = 1.;
        return;
    }

    // for event in scroll.read() {
    //     camera.scale *= event.y * time.delta_secs() * 0.01;
    // }

    if kb_input.pressed(KeyCode::NumpadAdd) {
        camera.scale *= 0.5f32.powf(time.delta_secs());
    }

    if kb_input.pressed(KeyCode::NumpadSubtract) {
        camera.scale *= 2.0f32.powf(time.delta_secs());
    }
}
