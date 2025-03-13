// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/fullData/FullDataPointIdMap.java
// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/util/FullDataPointUtil.java
use anyhow::Context;

use crate::light::LightLevel;

use super::columns::Columns;

#[derive(Clone, Copy)]
pub struct DataPoint {
    id: u32,
    meta: u32,
}

impl DataPoint {
    // const ID_WIDTH: usize = 32;
    const HEIGHT_WIDTH: usize = 12;
    const MIN_Y_WIDTH: usize = 12;
    const SKY_LIGHT_WIDTH: usize = 4;
    const BLOCK_LIGHT_WIDTH: usize = 4;

    // const ID_OFFSET: usize = 0;
    // const HEIGHT_OFFSET: usize = Self::ID_OFFSET + Self::ID_WIDTH;
    const HEIGHT_OFFSET: usize = 0;
    const MIN_Y_OFFSET: usize = Self::HEIGHT_OFFSET + Self::HEIGHT_WIDTH;
    const SKY_LIGHT_OFFSET: usize = Self::MIN_Y_OFFSET + Self::MIN_Y_WIDTH;
    const BLOCK_LIGHT_OFFSET: usize = Self::SKY_LIGHT_OFFSET + Self::SKY_LIGHT_WIDTH;

    // const ID_MASK: u64 = (1 << Self::ID_WIDTH) - 1;
    const HEIGHT_MASK: u32 = (1 << Self::HEIGHT_WIDTH) - 1;
    const MIN_Y_MASK: u32 = (1 << Self::MIN_Y_WIDTH) - 1;
    const SKY_LIGHT_MASK: u32 = (1 << Self::SKY_LIGHT_WIDTH) - 1;
    const BLOCK_LIGHT_MASK: u32 = (1 << Self::BLOCK_LIGHT_WIDTH) - 1;

    #[inline]
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> u16 {
        ((self.meta >> Self::HEIGHT_OFFSET) & Self::HEIGHT_MASK) as u16
    }

    #[inline]
    #[must_use]
    pub const fn min_y(&self) -> u16 {
        ((self.meta >> Self::MIN_Y_OFFSET) & Self::MIN_Y_MASK) as u16
    }

    #[inline]
    #[must_use]
    pub fn sky_light(&self) -> LightLevel {
        let level = ((self.meta >> Self::SKY_LIGHT_OFFSET) & Self::SKY_LIGHT_MASK) as u8;
        assert!(level <= 15);
        LightLevel::try_from(level).unwrap_or(LightLevel::Level15)
    }

    #[inline]
    #[must_use]
    pub fn block_light(&self) -> LightLevel {
        let level = ((self.meta >> Self::BLOCK_LIGHT_OFFSET) & Self::BLOCK_LIGHT_MASK) as u8;
        assert!(level <= 15);
        LightLevel::try_from(level).unwrap_or(LightLevel::Level15)
    }
}

impl core::fmt::Debug for DataPoint {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DataPoint")
            .field("id", &self.id())
            .field("height", &self.height())
            .field("min_y", &self.min_y())
            .field("sky_light", &self.sky_light())
            .field("block_light", &self.block_light())
            .finish()
    }
}

impl From<[u8; 8]> for DataPoint {
    #[inline]
    fn from(value: [u8; 8]) -> Self {
        let meta = u32::from_be_bytes(value[..4].try_into().unwrap());
        let id = u32::from_be_bytes(value[4..8].try_into().unwrap());
        Self { id, meta }
    }
}

// impl TryFrom<&[u8]> for DataPoint {
//     type Error = anyhow::Error;

//     #[inline]
//     fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
//         Ok(Self::from(value.try_into()?))
//     }
// }

impl TryFrom<&[u8]> for Columns<Box<[DataPoint]>> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(mut data: &[u8]) -> Result<Self, Self::Error> {
        use std::io::Read;
        use std::mem::MaybeUninit;
        let mut cols = std::array::from_fn(|_| MaybeUninit::uninit());
        for col in &mut cols {
            let mut len = [0, 0];
            data.read_exact(&mut len).context("reading u16")?;
            let len: usize = u16::from_be_bytes(len).into();
            anyhow::ensure!(data.len() >= len * 8, "not enough data for column");

            let mut points = Vec::with_capacity(len);
            for p in points.spare_capacity_mut() {
                let mut point = [0; 8];
                data.read_exact(&mut point).context("reading datapoint")?;
                p.write(point.into());
            }
            // Safety: we just wrote len elements
            unsafe { points.set_len(len) }
            col.write(points.into_boxed_slice());
        }

        // Safety: All elements have been initialized
        let cols = unsafe { cols.map(|col| col.assume_init()) };

        Ok(Self::new(cols))
    }
}

impl TryFrom<Box<[u8]>> for Columns<Box<[super::data::DataPoint]>> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        Self::try_from(data.as_ref())
    }
}

impl TryFrom<Box<[u8]>> for Box<Columns<Box<[super::data::DataPoint]>>> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        Columns::try_from(data.as_ref()).map(Self::new)
    }
}
