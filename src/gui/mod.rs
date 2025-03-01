//! Shows how to render a polygonal [`Mesh`], generated from a [`Rectangle`] primitive, in a 2D scene.
mod camera;

use std::collections::BTreeMap;

use bevy::{
    color::palettes::basic::{PURPLE, RED},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use crate::Section;

pub fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            camera::CameraPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, exit)
        .run();
}

const DH_PATH: &str = "DistantHorizons.sqlite";
// const DH_PATH: &str = "DistantHorizons_enderium.sqlite";

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    use crate::DetailLevel;

    const LOD_LEVEL: DetailLevel = DetailLevel::Chunk4;
    let section_width = LOD_LEVEL.block_width();
    let column_width = LOD_LEVEL.block_width() / 64;

    // let section = meshes.add(Mesh::from(Cuboid::from_size(Vec3 {
    //     x: DetailLevel::Chunk4.block_width() as f32,
    //     y: 1.,
    //     z: DetailLevel::Chunk4.block_width() as f32,
    // })));
    let section = meshes.add(Mesh::from(Rectangle::from_size(Vec2::new(
        section_width as f32,
        section_width as f32,
    ))));
    let column = meshes.add(Mesh::from(Rectangle::from_size(Vec2::new(
        column_width as f32,
        column_width as f32,
    ))));

    let purple = materials.add(Color::from(PURPLE));
    let red = materials.add(Color::from(RED));

    let mut color_materials: std::collections::BTreeMap<(u8, u8, u8), Handle<ColorMaterial>> =
        BTreeMap::new();

    let light_levels: [_; 15] = std::array::from_fn(|i| {
        let i = i as f32;
        let i = (i / 20.) + 0.25;
        let c = Color::linear_rgb(i, i, i);
        materials.add(c)
    });

    dbg!(&section);
    dbg!(&purple);

    let all_data = crate::Section::get_all_from_db(DH_PATH).unwrap();

    const CENTER: Vec2 = Vec2::new(-2100., -2500.);

    let mut get_color_material = |color: (u8, u8, u8)| -> Handle<ColorMaterial> {
        if let Some(material) = color_materials.get(&color) {
            return material.clone();
        }

        let material = materials.add(Color::srgb_u8(color.0, color.1, color.2));
        color_materials.insert(color, material.clone());
        material
    };

    for mut data in all_data {
        if data.pos.detail_level != LOD_LEVEL {
            continue;
        }

        let center = data.pos.center();
        let center = Vec2::new(center.0 as f32, -center.1 as f32);
        // if center.distance_squared(CENTER) > 500f32.powi(2) {
        //     continue;
        // }
        if center.distance_squared(CENTER) > 10000f32.powi(2) {
            continue;
        }
        // if data.pos.center_x() < -2500
        //     || data.pos.center_z() < 1700
        //     || data.pos.center_x() > -800
        //     || data.pos.center_z() > 2700
        // {
        //     continue;
        // }

        let pos = data.pos;
        let section_pos = Vec3 {
            x: pos.min_x() as f32,
            y: -pos.min_z() as f32,
            z: -1.,
        };

        if let Err(e) = data.decompress() {
            eprintln!("Failed to decompress section: {e:?}");

            commands.spawn((
                Mesh2d(section.clone()),
                MeshMaterial2d(red.clone()),
                Transform::from_translation(section_pos),
            ));

            continue;
        };

        commands.spawn((
            Mesh2d(section.clone()),
            MeshMaterial2d(purple.clone()),
            Transform::from_translation(section_pos),
        ));

        data.drop_caches();

        let cols = data.column_data().unwrap();
        let mapping = data.mapping().unwrap();

        for dz in (0..Section::WIDTH as i32).into_iter() {
            for dx in (0..Section::WIDTH as i32).into_iter() {
                let col = &cols[(dz as usize, dx as usize)];
                let block_pos = Vec3 {
                    x: (pos.min_x() + (dx * column_width)) as f32,
                    y: -(pos.min_z() + (dz * column_width)) as f32,
                    z: 1.,
                };

                let slices = col.into_iter().map(|p| {
                    let block = &mapping[p];
                    (p, block)
                });

                for (_dp, b) in slices {
                    if b.is_air() || b.block() == "minecraft:torch" {
                        continue;
                    }

                    let color = b.map_color();
                    let material = color
                        .map(|c| get_color_material(c))
                        .unwrap_or_else(|| purple.clone());

                    commands.spawn((
                        Mesh2d(column.clone()),
                        MeshMaterial2d(material),
                        Transform::from_translation(block_pos),
                    ));

                    break;
                }
            }
        }
    }
}

fn exit(kb_input: Res<ButtonInput<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if kb_input.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }

    if kb_input.just_pressed(KeyCode::KeyQ) {
        app_exit_events.send(AppExit::Success);
    }
}
