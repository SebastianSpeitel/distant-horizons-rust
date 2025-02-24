//! Shows how to render a polygonal [`Mesh`], generated from a [`Rectangle`] primitive, in a 2D scene.

use bevy::{
    color::palettes::basic::PURPLE,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

pub fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let square = meshes.add(Mesh::from(Rectangle::from_length(50.0)));
    let mat = materials.add(Color::from(PURPLE));

    dbg!(&square);
    dbg!(&mat);

    commands.spawn((Mesh2d(square), MeshMaterial2d(mat)));

    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::default())),
    //     MeshMaterial2d(materials.add(Color::from(PURPLE))),
    //     Transform::default().with_scale(Vec3::splat(1000.)),
    // ));
}
