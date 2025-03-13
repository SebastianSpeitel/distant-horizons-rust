// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/fullData/FullDataPointIdMap.java

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use smol_str::SmolStr;

use crate::block::Block;

const BLOCK_STATE_SEPARATOR_STRING: &str = "_DH-BSW_";

static BIOMES: RwLock<BTreeSet<SmolStr>> = RwLock::new(BTreeSet::new());
static BLOCKS: RwLock<BTreeSet<SmolStr>> = RwLock::new(BTreeSet::new());
static STATE_KEYS: RwLock<BTreeSet<SmolStr>> = RwLock::new(BTreeSet::new());
static STATE_VALUES: RwLock<BTreeSet<SmolStr>> = RwLock::new(BTreeSet::new());

fn intern(value: &str, set: &RwLock<BTreeSet<SmolStr>>) -> SmolStr {
    // value is stack allocated by SmolStr
    if value.len() <= 23 {
        return value.into();
    }
    let read = set.read().unwrap();
    if let Some(value) = read.get("") {
        return value.clone();
    }
    drop(read);
    let mut write = set.write().unwrap();
    let value: SmolStr = value.into();
    write.insert(value.clone());
    value
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entry {
    biome: SmolStr,
    block: SmolStr,
    state: BTreeMap<SmolStr, SmolStr>,
    #[cfg(test)]
    raw: String,
}

impl Entry {
    #[inline]
    pub fn biome(&self) -> &str {
        self.biome.as_str()
    }

    #[inline]
    pub fn in_nether(&self) -> bool {
        matches!(
            self.biome.as_str(),
            "minecraft:nether_wastes"
                | "minecraft:crimson_forest"
                | "minecraft:warped_forest"
                | "minecraft:soul_sand_valley"
                | "minecraft:basalt_deltas"
        )
    }
}

impl Block for Entry {
    #[inline]
    fn full_identifier(&self) -> &str {
        self.block.as_str()
    }

    #[inline]
    fn get_state(&self, state: &str) -> Option<&str> {
        self.state.get(state).map(SmolStr::as_str)
    }
}

impl Default for Entry {
    #[inline]
    fn default() -> Self {
        Self {
            biome: SmolStr::new_static("minecraft:plains"),
            block: SmolStr::new_static("AIR"),
            state: BTreeMap::new(),
            #[cfg(test)]
            raw: format!("minecraft:plains{}AIR", BLOCK_STATE_SEPARATOR_STRING),
        }
    }
}

impl TryFrom<String> for Entry {
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
pub struct Mapping(Box<[Entry]>);

impl core::ops::Index<usize> for Mapping {
    type Output = Entry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::Index<&super::data::DataPoint> for Mapping {
    type Output = Entry;

    #[inline]
    fn index(&self, index: &super::data::DataPoint) -> &Self::Output {
        // this is somehow the bottleneck
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
            let entry = Entry::try_from(entry)?;
            entries.push(entry);
        }

        anyhow::ensure!(data.is_empty(), "data not fully consumed");

        Ok(Self(entries.into_boxed_slice()))
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
