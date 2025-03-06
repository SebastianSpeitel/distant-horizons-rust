//! Shows how to render a polygonal [`Mesh`], generated from a [`Rectangle`] primitive, in a 2D scene.
mod camera;
mod section;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::Wireframe2dPlugin,
};

use crate::DetailLevel;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            camera::CameraPlugin,
            Wireframe2dPlugin,
        ))
        .add_systems(Update, (exit, section::update, load))
        .add_systems(FixedUpdate, (section::decompress, section::texturing))
        .run();
}

fn exit(kb_input: Res<ButtonInput<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if kb_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }

    if kb_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(AppExit::Success);
    }
}

fn load(mut commands: Commands, kb_input: Res<ButtonInput<KeyCode>>) {
    let dh_path = std::env::var("DH_PATH").unwrap_or_else(|_| "DistantHorizons.sqlite".to_string());

    const LOD_LEVEL: DetailLevel = DetailLevel::Chunk4;
    if kb_input.just_pressed(KeyCode::KeyL) {
        let all_sections =
            crate::Section::get_all_with_detail_level_from_db(dh_path, LOD_LEVEL).unwrap();
        bevy::log::info!("Loaded {} sections", all_sections.len());
        for section in all_sections {
            if section.pos.detail_level != LOD_LEVEL {
                continue;
            }
            let mut transform = Transform::default();
            update_section_transform(&mut transform, &section);
            bevy::log::info!("Spawning section at {:?}", section.pos.center());
            commands.spawn((section, transform));
        }
    }
}

const fn update_section_transform(transform: &mut Transform, section: &crate::Section) {
    let width = section.block_width() as f32;

    transform.translation.x = section.pos.center_x() as f32;
    transform.translation.y = -section.pos.center_z() as f32;
    transform.translation.z = section.min_y as f32;
    transform.scale.x = width;
    transform.scale.y = width;
}
