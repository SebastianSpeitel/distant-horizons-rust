use bevy::prelude::*;

use crate::Section;

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Decompressed;

const PURPLE: (u8, u8, u8) = (255, 0, 255);

#[derive(Debug, Clone, Copy, Default, Component)]
pub struct Visible;

#[inline]
pub fn update(
    mut commands: Commands,
    sections: Query<(Entity, &Section<'static>), Changed<Section<'static>>>,
) {
    for (entity, section) in sections.iter() {
        if !section.is_decompressed() {
            commands.entity(entity).remove::<Decompressed>();
        }
    }
}

#[inline]
pub fn decompress(
    commands: ParallelCommands,
    mut sections: Query<(Entity, &mut Section<'static>), Without<Decompressed>>,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;
    if sections.is_empty() {
        return;
    }

    let start = Instant::now();

    let stopped = AtomicBool::new(false);

    sections.par_iter_mut().for_each(|(entity, mut section)| {
        if stopped.load(Ordering::Relaxed) {
            return;
        }
        if start.elapsed().as_millis() > 1000 / 65 {
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
    sections: Query<
        (Entity, &Section<'static>),
        (
            Or<(Without<Sprite>, Changed<Section<'static>>)>,
            With<Decompressed>,
        ),
    >,
    mut assets: ResMut<Assets<Image>>,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::channel;
    use std::time::Instant;

    if sections.is_empty() {
        return;
    }

    let start = Instant::now();

    let stopped = AtomicBool::new(false);

    let (tx, rx) = channel();

    sections.par_iter().for_each(|(e, section)| {
        if stopped.load(Ordering::Relaxed) {
            return;
        }
        if start.elapsed().as_millis() > 10 {
            stopped.store(true, Ordering::Relaxed);
            return;
        }
        bevy::log::trace!("Texturing section {e}");
        let Some(img) = build_section_image(section) else {
            return;
        };
        if tx.send((e, img)).is_err() {
            stopped.store(true, Ordering::Relaxed);
        }
    });
    drop(tx);

    let sprites = rx
        .into_iter()
        .map(|(e, img)| {
            let handle = assets.add(img);
            let sprite = Sprite::from_image(handle);
            (e, sprite)
        })
        .collect::<Vec<_>>();

    commands.insert_batch(sprites);

    // for (e, img) in rx {
    //     let handle = assets.add(img);
    //     let mut sprite = Sprite::from_image(handle);
    //     sprite.custom_size.replace(Vec2::new(1., 1.));
    //     let mut entity = commands.entity(e);
    //     entity.insert(sprite);
    // }
}

fn build_section_image(section: &Section) -> Option<Image> {
    // use smol_str::SmolStr;
    // use std::collections::BTreeMap;
    // use std::sync::Mutex;
    // static BLOCK_CACHE: Mutex<BTreeMap<SmolStr, CachedBlock<SmolStr>>> =
    //     Mutex::new(BTreeMap::new());

    use crate::block::Block;
    use bevy::render::render_resource::Extent3d;
    use bevy::render::render_resource::TextureDescriptor;
    let mut img = Image {
        data: Some(vec![0; Section::WIDTH * Section::WIDTH * 4]),
        texture_descriptor: TextureDescriptor {
            size: Extent3d {
                width: Section::WIDTH as u32,
                height: Section::WIDTH as u32,
                depth_or_array_layers: 1,
            },
            ..Image::transparent().texture_descriptor
        },
        ..Image::transparent()
    };
    let img_data = img.data.as_mut()?;

    let cols = section.column_data()?;
    let mapping = section.mapping()?;
    let mut is_nether = None;
    // let mut cache = BLOCK_CACHE.lock().unwrap();

    // let mut cache = BTreeMap::<SmolStr, CachedBlock<SmolStr>>::new();

    for dz in 0..Section::WIDTH as u32 {
        for dx in 0..Section::WIDTH as u32 {
            let col = cols[(dz as usize, dx as usize)].as_ref();
            let slices = col.into_iter().map(|p| {
                let entry = &mapping[p];

                // let entry = cache
                //     .entry(entry.block.clone())
                //     .or_insert_with(|| CachedBlock::new(entry.block.clone()));

                // // if let Some(block) = cache.get(entry.block.as_str()) {
                // //     return (*p, block.to_owned());
                // // }

                // // let block = CachedBlock::new(entry.block.clone());
                // // cache.insert(entry.block.clone(), block.to_owned());

                let block = entry.to_owned();

                (*p, block)
            });

            let mut above_nether_roof = true;
            let mut in_or_above_nether_roof = true;
            for (_dp, b) in slices {
                if b.is_transparent() {
                    in_or_above_nether_roof = above_nether_roof;
                    continue;
                }
                above_nether_roof = false;

                let is_nether = is_nether.get_or_insert_with(|| b.in_nether());

                if *is_nether && (above_nether_roof || in_or_above_nether_roof) {
                    continue;
                }

                let (r, g, b) = b.map_color().unwrap_or(PURPLE);
                // let color = Color::srgb_u8(r, g, b);

                let index = (dz * Section::WIDTH as u32 + dx) as usize;
                let offset = index * 4;
                // let bytes = &mut img.data[offset..offset + 4];

                // bytes[0] = r;
                // bytes[1] = g;
                // bytes[2] = b;
                // bytes[3] = u8::MAX;

                img_data[offset + 0] = r;
                img_data[offset + 1] = g;
                img_data[offset + 2] = b;
                img_data[offset + 3] = u8::MAX;

                // img.set_color_at(dx, dz, color).unwrap();
                break;
            }
        }
    }

    Some(img)
}
