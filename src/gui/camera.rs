use bevy::prelude::*;

const CAMERA_SPEED: f32 = 500.;
/// How quickly should the camera snap to the desired location.
const CAMERA_DECAY_RATE: f32 = 2.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let camera_transform = Transform {
            // rotation: Quat::from_rotation_z(PI),
            // translation: Vec3::new(0., 49., 0.),
            ..Transform::default()
        };

        app.world_mut().spawn((
            Camera2d,
            camera_transform,
            OrthographicProjection {
                far: 10000.,
                ..OrthographicProjection::default_2d()
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
) {
    if kb_input.pressed(KeyCode::KeyR) {
        camera.translation = Vec3::new(0., 0., 0.);
        return;
    }

    let mut direction = Vec2::ZERO;
    let mut rotation = 0.;
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

    if kb_input.pressed(KeyCode::KeyE) {
        rotation -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyQ) {
        rotation += 1.;
    }

    if kb_input.pressed(KeyCode::NumpadAdd) {
        camera.scale *= 0.9;
    }

    if kb_input.pressed(KeyCode::NumpadSubtract) {
        camera.scale *= 1.1;
    }

    if kb_input.pressed(KeyCode::Numpad1) {
        camera.translation = Vec3::new(-2300., -2000., 0.);
    }

    let delta = direction.normalize_or_zero() * CAMERA_SPEED; // * time.delta_secs();

    let target = camera.translation + delta.extend(0.);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());

    rotation *= time.delta_secs();

    // camera.rotate_local_z(rotation);
}

fn zoom_camera(
    mut camera: Single<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
) {
    for event in scroll.read() {
        camera.scale *= event.y.mul_add(0.1, 1.0);
    }
}
