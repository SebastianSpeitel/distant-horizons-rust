mod camera;
mod duck;
mod section;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use crate::DetailLevel;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            camera::CameraPlugin,
            duck::DuckPlugin,
            // Wireframe2dPlugin,
        ))
        .add_systems(Startup, attach_distant_horizons)
        .add_systems(Update, (exit, section::update, load))
        .add_systems(FixedUpdate, (section::decompress, section::texturing))
        .run();
}

fn attach_distant_horizons(db: Res<duck::DuckDb>) {
    let dh_path = std::env::var("DH_PATH").unwrap_or_else(|_| "DistantHorizons.sqlite".to_string());
    db.attach_distant_horizons(dh_path).unwrap();
}

fn exit(kb_input: Res<ButtonInput<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if kb_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }

    if kb_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(AppExit::Success);
    }
}

fn load(
    mut commands: Commands,
    kb_input: Res<ButtonInput<KeyCode>>,
    db: Res<duck::DuckDb>,
    mut last_load: Local<i64>,
    old_sections: Query<(Entity, &crate::section::pos::Pos)>,
) {
    use core::mem::take;
    const LOD_LEVEL: DetailLevel = DetailLevel::Chunk16;
    if !kb_input.just_pressed(KeyCode::KeyL) {
        return;
    }

    let mut old_sections: std::collections::BTreeMap<_, _> =
        old_sections.iter().map(|(e, p)| (*p, e)).collect();

    let conn = db.lock();
    let mut modified =
        crate::Section::get_all_with_detail_level_modified_after(&conn, LOD_LEVEL, *last_load)
            .unwrap();
    bevy::log::info!("Found {} modified sections", modified.len());
    drop(conn);

    modified.retain_mut(|s| {
        *last_load = s.last_modified().max(*last_load);
        if let Some(old) = old_sections.remove(&s.pos) {
            let mut old = commands.entity(old);
            old.insert(take(s));
            false
        } else {
            true
        }
    });

    let bundles = modified.into_iter().map(|s| (s.transform_2d(), s.pos, s));
    commands.spawn_batch(bundles);
}

impl crate::Section<'_> {
    fn transform_2d(&self) -> Transform {
        Transform {
            translation: Vec3 {
                x: self.pos.center_x() as f32,
                y: -self.pos.center_z() as f32,
                z: self.min_y as f32,
            },
            scale: Vec3 {
                x: self.width() as f32,
                y: self.width() as f32,
                z: 1.0,
            },
            rotation: Quat::IDENTITY,
        }
    }
}
