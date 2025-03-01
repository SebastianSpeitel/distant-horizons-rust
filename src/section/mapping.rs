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
    pub fn block(&self) -> &'static str {
        self.block
    }

    #[inline]
    #[must_use]
    pub fn map_color(&self) -> Option<(u8, u8, u8)> {
        Some(match self.block {
            b if b.contains("grass_block") => (127, 178, 56),
            "minecraft:cherry_leaves" => (242, 127, 165),
            b if b.contains("grass")
                || b.contains("leave")
                || b.contains("leaf")
                || b.contains("vine")
                || b.contains("cactus")
                || b.contains("fern")
                || b.contains("lily_pad")
                || b.contains("sugar_cane")
                || b.contains("flower")
                || b.contains("seed")
                || b.contains("wheat")
                || b.contains("carrot")
                || b.contains("potato")
                || b.contains("sweet_berry_bush")
                || b.contains("petal")
                || b.contains("sapling") =>
            {
                (0, 124, 0)
            }
            b if (b.contains("dark") && b.contains("oak")) || b.contains("soul") => (102, 76, 51),
            b if (b.contains("sand") && !b.contains("red")) || b.contains("birch") => {
                (247, 233, 163)
            }
            b if b.contains("wool") => (199, 199, 199),
            b if b.contains("tnt") || b.contains("fire") || b.contains("lava") => (255, 0, 0),
            b if b.contains("ice") => (160, 160, 255),
            b if b.contains("iron") => (167, 167, 167),

            b if b.contains("snow") => (255, 255, 255),
            b if b.contains("clay") => (164, 168, 184),
            b if b.contains("dirt")
                || b.contains("farmland")
                || b.contains("granite")
                || b.contains("jungle") =>
            {
                (151, 109, 77)
            }
            b if b.contains("stone")
                || b.contains("andesite")
                || b.contains("ore")
                || b.contains("gravel") =>
            {
                (112, 112, 112)
            }
            b if b.contains("water") || b.contains("seagrass") || b.contains("kelp") => {
                (64, 64, 255)
            }
            b if b.contains("oak") || b.contains("chest") => (143, 119, 72),
            b if b.contains("diorite") || b.contains("quartz") => (255, 252, 245),
            b if b.contains("acacia")
                || (b.contains("red") && b.contains("sand"))
                || b.contains("copper") =>
            {
                (216, 127, 51)
            }
            b if b.contains("mycelium") || b.contains("amethyst") || b.contains("chorus") => {
                (127, 63, 178)
            }
            b if b.contains("podzol") || b.contains("spruce") || b.contains("mangrove") => {
                (129, 86, 49)
            }
            b if b.contains("bamboo") => (229, 229, 51),
            b if b.contains("calcite") || b.contains("cherry") => (209, 177, 161),
            _ => return None,
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
