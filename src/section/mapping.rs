// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/fullData/FullDataPointIdMap.java

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

const BLOCK_STATE_SEPARATOR_STRING: &str = "_DH-BSW_";

static BIOMES: RwLock<BTreeSet<&'static str>> = RwLock::new(BTreeSet::new());
static BLOCKS: RwLock<BTreeSet<&'static str>> = RwLock::new(BTreeSet::new());
static STATE_KEYS: RwLock<BTreeSet<&'static str>> = RwLock::new(BTreeSet::new());
static STATE_VALUES: RwLock<BTreeSet<&'static str>> = RwLock::new(BTreeSet::new());

fn intern(value: &str, set: &RwLock<BTreeSet<&'static str>>) -> &'static str {
    let read = set.read().unwrap();
    if let Some(value) = read.get(value) {
        return value;
    }
    drop(read);
    let mut write = set.write().unwrap();
    write.insert(Box::leak(value.into()));
    write.get(value).unwrap()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block {
    biome: &'static str,
    block: &'static str,
    state: BTreeMap<&'static str, &'static str>,
    #[cfg(test)]
    raw: String,
}

impl Block {
    #[inline]
    #[must_use]
    pub fn is_air(&self) -> bool {
        self.block == "AIR"
    }

    #[inline]
    #[must_use]
    pub fn is_transparent(&self) -> bool {
        match self.block {
            "AIR" => true,
            "minecraft:torch" => true,
            "minecraft:wall_torch" => true,
            "minecraft:rail" => true,
            "minecraft:powered_rail" => true,
            "minecraft:lever" => true,
            "minecraft:ladder" => true,
            "minecraft:glass" => true,
            "minecraft:repeater" => true,
            "minecraft:iron_bars" => true,
            "minecraft:redstone_wire" => true,
            b if b.starts_with("minecraft:potted_") => true,
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn block(&self) -> &'static str {
        self.block
    }

    /// <https://minecraft.wiki/w/Map_item_format#Color_table>
    #[inline]
    #[must_use]
    pub fn map_color(&self) -> Option<(u8, u8, u8)> {
        type Col = (u8, u8, u8);
        const COLOR_RED: Col = (153, 51, 51);
        const DIRT: Col = (151, 109, 77);
        const PLANT: Col = (0, 124, 0);
        const WOOD: Col = (143, 119, 72);
        const COLOR_ORANGE: Col = (216, 127, 51);
        const COLOR_GREEN: Col = (102, 127, 51);
        const COLOR_BROWN: Col = (102, 76, 51);
        const COLOR_YELLOW: Col = (229, 229, 51);
        const COLOR_BLACK: Col = (25, 25, 25);
        const COLOR_LIGHT_GREEN: Col = (127, 204, 25);
        const COLOR_PINK: Col = (242, 127, 165);
        const COLOR_PURPLE: Col = (127, 63, 178);
        const TERRACOTTA_ORANGE: Col = (159, 82, 36);
        const TERRACOTTA_CYAN: Col = (87, 92, 92);
        const TERRACOTTA_BROWN: Col = (76, 50, 35);
        const TERRACOTTA_YELLOW: Col = (186, 133, 36);
        const TERRACOTTA_RED: Col = (142, 60, 46);
        const TERRACOTTA_LIGHT_GRAY: Col = (135, 107, 98);
        const TERRACOTTA_WHITE: Col = (209, 177, 161);
        const TERRACOTTA_BLUE: Col = (76, 62, 92);
        const NETHER: Col = (112, 2, 0);
        const GOLD: Col = (250, 238, 77);
        const METAL: Col = (167, 167, 167);
        const WOOL: Col = (199, 199, 199);
        const GLOW_LICHEN: Col = (127, 167, 150);
        const STONE: Col = (112, 112, 112);
        const WATER: Col = (64, 64, 255);
        const GRASS: Col = (127, 178, 56);
        const SAND: Col = (247, 233, 163);
        const SNOW: Col = (255, 255, 255);
        const QUARTZ: Col = (255, 252, 245);
        const ICE: Col = (160, 160, 255);
        const PODZOL: Col = (129, 86, 49);
        const FIRE: Col = (255, 0, 0);
        const CLAY: Col = (164, 168, 184);

        Some(match self.block {
            "minecraft:water" => WATER,
            "minecraft:grass_block" => GRASS,
            "minecraft:sand" => SAND,
            "minecraft:stone" => STONE,
            "minecraft:dirt" => DIRT,
            "minecraft:gravel" => STONE,
            "minecraft:red_mushroom_block" => COLOR_RED,
            "minecraft:brown_mushroom_block" => DIRT,
            "minecraft:rose_bush" => PLANT,
            "minecraft:dead_bush" => WOOD,
            "minecraft:lilac" => PLANT,
            "minecraft:oxeye_daisy" => PLANT,
            "minecraft:allium" => PLANT,
            "minecraft:lily_of_the_valley" => PLANT,
            "minecraft:pink_tulip" => PLANT,
            "minecraft:white_tulip" => PLANT,
            "minecraft:red_tulip" => PLANT,
            "minecraft:peony" => PLANT,
            "minecraft:azalea" => PLANT,
            "minecraft:beetroots" => PLANT,
            "minecraft:blue_orchid" => PLANT,
            "minecraft:orange_tulip" => PLANT,
            "minecraft:pumpkin_stem" => PLANT,
            "minecraft:attached_pumpkin_stem" => PLANT,
            "minecraft:melon_stem" => PLANT,
            "minecraft:attached_melon_stem" => PLANT,
            "minecraft:lily_pad" => PLANT,
            "minecraft:vine" => PLANT,
            "minecraft:azure_bluet" => PLANT,
            "minecraft:short_grass" => PLANT,
            "minecraft:tall_grass" => PLANT,
            "minecraft:dandelion" => PLANT,
            "minecraft:cornflower" => PLANT,
            "minecraft:poppy" => PLANT,
            "minecraft:sweet_berry_bush" => PLANT,
            "minecraft:sugar_cane" => PLANT,
            "minecraft:fern" => PLANT,
            "minecraft:large_fern" => PLANT,
            "minecraft:wheat" => PLANT,
            "minecraft:carrots" => PLANT,
            "minecraft:potatoes" => PLANT,
            "minecraft:cactus" => PLANT,
            "minecraft:pink_petals" => PLANT,
            "minecraft:sunflower" => PLANT,
            "minecraft:small_dripleaf" => PLANT,
            "minecraft:big_dripleaf" => PLANT,
            "minecraft:flowering_azalea" => PLANT,
            "minecraft:pumpkin" => COLOR_ORANGE,
            "minecraft:carved_pumpkin" => COLOR_ORANGE,
            "minecraft:jack_o_lantern" => COLOR_ORANGE,
            "minecraft:melon" => COLOR_LIGHT_GREEN,
            "minecraft:moss_carpet" => COLOR_GREEN,
            "minecraft:moss_block" => COLOR_GREEN,
            "minecraft:mud" => TERRACOTTA_CYAN,
            "minecraft:terracotta" => COLOR_ORANGE,
            "minecraft:orange_terracotta" => TERRACOTTA_ORANGE,
            "minecraft:brown_terracotta" => TERRACOTTA_BROWN,
            "minecraft:yellow_terracotta" => TERRACOTTA_YELLOW,
            "minecraft:red_terracotta" => TERRACOTTA_RED,
            "minecraft:light_gray_terracotta" => TERRACOTTA_LIGHT_GRAY,
            "minecraft:white_terracotta" => TERRACOTTA_WHITE,
            "minecraft:blue_terracotta" => TERRACOTTA_BLUE,
            "minecraft:brown_mushroom" => COLOR_BROWN,
            "minecraft:red_mushroom" => COLOR_RED,
            "minecraft:netherrack" => NETHER,
            "minecraft:magma_block" => NETHER,
            "minecraft:nether_brick_slab" => NETHER,
            "minecraft:bee_nest" => COLOR_YELLOW,
            "minecraft:obsidian" => COLOR_BLACK,
            "minecraft:crying_obsidian" => COLOR_BLACK,
            "minecraft:hay_block" => COLOR_YELLOW,
            "minecraft:crafting_table" => WOOD,
            "minecraft:bell" => GOLD,
            "minecraft:gold_block" => GOLD,
            "minecraft:composter" => WOOD,
            "minecraft:lantern" => METAL,
            "minecraft:mud_bricks" => TERRACOTTA_LIGHT_GRAY,
            "minecraft:mud_brick_slab" => TERRACOTTA_LIGHT_GRAY,
            "minecraft:nether_wart" => COLOR_RED,
            "minecraft:cobweb" => WOOL,
            "minecraft:glow_lichen" => GLOW_LICHEN,
            "minecraft:observer" => STONE,
            "minecraft:kelp" => WATER,
            "minecraft:seagrass" => WATER,
            "minecraft:tall_seagrass" => WATER,
            "minecraft:bubble_column" => WATER,
            "minecraft:snow" => SNOW,
            "minecraft:snow_block" => SNOW,
            "minecraft:powder_snow" => SNOW,
            "minecraft:andesite" => STONE,
            "minecraft:diorite" => QUARTZ,
            "minecraft:diorite_wall" => QUARTZ,
            "minecraft:ice" => ICE,
            "minecraft:packed_ice" => ICE,
            "minecraft:blue_ice" => ICE,
            "minecraft:cherry_leaves" => COLOR_PINK,
            "minecraft:cherry_log" => TERRACOTTA_WHITE,
            "minecraft:calcite" => TERRACOTTA_WHITE,
            "minecraft:dirt_path" => DIRT,
            "minecraft:chest" => WOOD,
            "minecraft:mycelium" => COLOR_PURPLE,
            "minecraft:red_sand" => COLOR_ORANGE,
            "minecraft:podzol" => PODZOL,
            "minecraft:mossy_cobblestone" => STONE,
            "minecraft:coarse_dirt" => DIRT,
            "minecraft:campfire" => PODZOL,
            "minecraft:mangrove_roots" => PODZOL,
            "minecraft:muddy_mangrove_roots" => PODZOL,
            "minecraft:granite" => DIRT,
            "minecraft:coal_ore" => STONE,
            "minecraft:iron_ore" => STONE,
            "minecraft:copper_ore" => STONE,
            "minecraft:emerald_ore" => STONE,
            "minecraft:farmland" => DIRT,
            "minecraft:pointed_dripstone" => TERRACOTTA_BROWN,
            "minecraft:dripstone_block" => TERRACOTTA_BROWN,
            "minecraft:lava" => FIRE,
            "minecraft:stone_slab" => STONE,
            "minecraft:rooted_dirt" => DIRT,
            "minecraft:clay" => CLAY,
            "minecraft:sea_pickle" => COLOR_GREEN,
            "minecraft:polished_granite" => DIRT,
            "minecraft:red_sandstone" => COLOR_ORANGE,
            "minecraft:smooth_stone" => STONE,
            "minecraft:smooth_stone_slab" => STONE,
            "minecraft:suspicious_gravel" => STONE,
            "minecraft:sandstone" => SAND,
            "minecraft:smooth_sandstone" => SAND,
            "minecraft:smooth_sandstone_stairs" => SAND,
            "minecraft:smooth_sandstone_slab" => SAND,
            "minecraft:cut_sandstone" => SAND,
            "minecraft:sandstone_slab" => SAND,
            "minecraft:sandstone_stairs" => SAND,
            "minecraft:sandstone_wall" => SAND,
            "minecraft:grindstone" => METAL,
            "minecraft:furnace" => STONE,
            "minecraft:stonecutter" => STONE,
            "minecraft:bamboo" => COLOR_YELLOW,
            // Saplings and leaves before woods
            b if b.ends_with("sapling") => PLANT,
            b if b.ends_with("leaves") => PLANT,
            // All wood types
            b if b.contains("dark_oak") => COLOR_BROWN,
            b if b.contains("oak") => WOOD,
            b if b.contains("acacia") => COLOR_ORANGE,
            b if b.contains("birch") => SAND,
            b if b.contains("spruce") => PODZOL,
            b if b.contains("mangrove") => COLOR_RED,
            b if b.contains("jungle") => DIRT,
            b if b.contains("stone_brick") => STONE,
            b if b.contains("cobblestone") => STONE,
            b if b.contains("copper") => COLOR_ORANGE,
            // Banners before colors
            b if b.ends_with("banner") => WOOD,
            b if b.contains("white") => SNOW,
            b if b.contains("yellow") => COLOR_YELLOW,
            b if b.contains("red") => COLOR_RED,
            b if b.contains("coral") => WATER,
            b => {
                bevy::log::info!("unknown map color for: {b}");
                return None;
            }
        })
    }
}

impl Default for Block {
    #[inline]
    fn default() -> Self {
        Self {
            biome: "minecraft:plains",
            block: "AIR",
            state: BTreeMap::new(),
            #[cfg(test)]
            raw: format!("minecraft:plains{}AIR", BLOCK_STATE_SEPARATOR_STRING),
        }
    }
}

impl TryFrom<String> for Block {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        use anyhow::Context;
        let (biome, block_state) = value
            .split_once(BLOCK_STATE_SEPARATOR_STRING)
            .context("missing separator after biome")?;

        let (block, state_str) = block_state
            .split_once("_STATE_")
            .unwrap_or((block_state, ""));

        let biome = intern(biome, &BIOMES);
        let block = intern(block, &BLOCKS);

        if state_str.is_empty() {
            return Ok(Self {
                biome,
                block,
                state: BTreeMap::new(),
                #[cfg(test)]
                raw: value,
            });
        }

        // minecraft:spruce_leaves_STATE_{distance:1}{persistent:false}{waterlogged:false}
        let mut state = BTreeMap::new();
        for kv in state_str.split('}') {
            if kv.is_empty() {
                continue;
            }
            let Some(("{", kv)) = kv.split_at_checked(1) else {
                anyhow::bail!("missing opening brace in state");
            };
            let (key, value) = kv.split_once(':').context("missing separator in state")?;
            let key = intern(key, &STATE_KEYS);
            let value = intern(value, &STATE_VALUES);
            state.insert(key, value);
        }

        Ok(Self {
            biome,
            block,
            state,
            #[cfg(test)]
            raw: value,
        })
    }
}

#[derive(Debug)]
pub struct Mapping(Vec<Block>);

impl core::ops::Index<usize> for Mapping {
    type Output = Block;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::Index<&super::data::DataPoint> for Mapping {
    type Output = Block;

    #[inline]
    fn index(&self, index: &super::data::DataPoint) -> &Self::Output {
        &self.0[index.id() as usize]
    }
}

impl TryFrom<Box<[u8]>> for Mapping {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        use std::io::Read;
        let mut data = data.as_ref();

        let mut count = [0; 4];
        data.read_exact(&mut count)?;
        let count: usize = u32::from_be_bytes(count).try_into()?;

        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let entry = crate::java::readUTF(&mut data)?;
            let entry = Block::try_from(entry)?;
            entries.push(entry);
        }

        anyhow::ensure!(data.is_empty(), "data not fully consumed");

        Ok(Self(entries))
    }
}

pub fn print_interned_sizes() {
    println!("biomes: {}", BIOMES.read().unwrap().len());
    println!("blocks: {}", BLOCKS.read().unwrap().len());
    println!("state keys: {}", STATE_KEYS.read().unwrap().len());
    println!("state values: {}", STATE_VALUES.read().unwrap().len());

    println!("biomes: {:?}", BIOMES.read().unwrap());
    println!("blocks: {:?}", BLOCKS.read().unwrap());
    println!("state keys: {:?}", STATE_KEYS.read().unwrap());
    println!("state values: {:?}", STATE_VALUES.read().unwrap());
}
