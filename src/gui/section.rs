use bevy::prelude::*;

use crate::Section;

#[derive(Debug, Component)]
pub struct Decompressed;

const PURPLE: (u8, u8, u8) = (255, 0, 255);

#[inline]
pub fn update(
    mut commands: Commands,
    mut sections: Query<(Entity, &Section<'static>), Changed<Section<'static>>>,
) {
    for (entity, section) in sections.iter_mut() {
        bevy::log::debug!("Updating section {entity}");
        let mut entity = commands.entity(entity);

        // let wire_color = if section.is_decompressed() {
        //     Wireframe2dColor {
        //         color: Color::linear_rgb(0.0, 1.0, 0.0),
        //     }
        // } else {
        //     Wireframe2dColor {
        //         color: Color::linear_rgb(1.0, 0.0, 0.0),
        //     }
        // };

        entity.insert((
            section.pos.detail_level,
            section.compression(),
            // unit_rect.get(),
            // Wireframe2d,
            // wire_color,
        ));
    }
}

#[inline]
pub fn decompress(
    commands: ParallelCommands,
    mut sections: Query<(Entity, &mut Section<'static>), Without<Decompressed>>,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;
    let start = Instant::now();

    let stopped = AtomicBool::new(false);

    sections.par_iter_mut().for_each(|(entity, mut section)| {
        if stopped.load(Ordering::Relaxed) {
            return;
        }
        if start.elapsed().as_millis() > 10 {
            stopped.store(true, Ordering::Relaxed);
            return;
        }
        bevy::log::debug!("Decompressing section {entity}");
        if let Err(e) = section.decompress() {
            bevy::log::error!("Failed to decompress section: {e:?}");
        }
        debug_assert!(section.is_decompressed());
        commands.command_scope(|mut c| {
            c.entity(entity).insert(Decompressed);
        });
    });
}

pub fn texturing(
    mut commands: Commands,
    sections: Query<(Entity, &Section<'static>), (Without<Sprite>, With<Decompressed>)>,
    mut assets: ResMut<Assets<Image>>,
) {
    for (e, section) in sections.iter().take(32) {
        bevy::log::debug!("Texturing section {e}");
        let img = build_section_image(section);
        let handle = assets.add(img);
        let mut sprite = Sprite::from_image(handle);
        // sprite.custom_size.replace(Vec2 {
        //     x: Section::WIDTH as f32,
        //     y: Section::WIDTH as f32,
        // });
        sprite.custom_size.replace(Vec2::new(1., 1.));
        let mut entity = commands.entity(e);
        entity.insert(sprite);
        // entity.remove::<Mesh2d>();
    }
}

fn build_section_image(section: &Section) -> Image {
    use bevy::render::render_resource::Extent3d;
    let extent = Extent3d {
        width: Section::WIDTH as u32,
        height: Section::WIDTH as u32,
        ..Default::default()
    };
    let mut img = Image::transparent();
    img.resize(extent);

    let cols = section.column_data().unwrap();
    let mapping = section.mapping().unwrap();

    for dz in 0..Section::WIDTH as u32 {
        for dx in 0..Section::WIDTH as u32 {
            let col = cols[(dz as usize, dx as usize)].as_ref();
            let slices = col.into_iter().map(|p| {
                let block = &mapping[p];
                (*p, block)
            });

            for (_dp, b) in slices {
                if b.is_transparent() {
                    continue;
                }

                let (r, g, b) = b.map_color().unwrap_or(PURPLE);
                let material = Color::srgb_u8(r, g, b);

                img.set_color_at(dx, dz, material).unwrap();
                break;
            }
        }
    }

    img
}
