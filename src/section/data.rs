// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/dataObjects/fullData/FullDataPointIdMap.java
// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/util/FullDataPointUtil.java
use anyhow::Context;

use crate::light::LightLevel;

use super::columns::Columns;

#[derive(Clone, Copy)]
pub struct DataPoint(u64);

impl DataPoint {
    const ID_WIDTH: usize = 32;
    const HEIGHT_WIDTH: usize = 12;
    const MIN_Y_WIDTH: usize = 12;
    const SKY_LIGHT_WIDTH: usize = 4;
    const BLOCK_LIGHT_WIDTH: usize = 4;

    const ID_OFFSET: usize = 0;
    const HEIGHT_OFFSET: usize = Self::ID_OFFSET + Self::ID_WIDTH;
    const MIN_Y_OFFSET: usize = Self::HEIGHT_OFFSET + Self::HEIGHT_WIDTH;
    const SKY_LIGHT_OFFSET: usize = Self::MIN_Y_OFFSET + Self::MIN_Y_WIDTH;
    const BLOCK_LIGHT_OFFSET: usize = Self::SKY_LIGHT_OFFSET + Self::SKY_LIGHT_WIDTH;

    const ID_MASK: u64 = (1 << Self::ID_WIDTH) - 1;
    const HEIGHT_MASK: u64 = (1 << Self::HEIGHT_WIDTH) - 1;
    const MIN_Y_MASK: u64 = (1 << Self::MIN_Y_WIDTH) - 1;
    const SKY_LIGHT_MASK: u64 = (1 << Self::SKY_LIGHT_WIDTH) - 1;
    const BLOCK_LIGHT_MASK: u64 = (1 << Self::BLOCK_LIGHT_WIDTH) - 1;

    #[inline]
    #[must_use]
    pub const fn id(&self) -> u32 {
        (self.0 & Self::ID_MASK) as u32
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> u16 {
        ((self.0 >> Self::HEIGHT_OFFSET) & Self::HEIGHT_MASK) as u16
    }

    #[inline]
    #[must_use]
    pub const fn min_y(&self) -> u16 {
        ((self.0 >> Self::MIN_Y_OFFSET) & Self::MIN_Y_MASK) as u16
    }

    #[inline]
    #[must_use]
    pub fn sky_light(&self) -> LightLevel {
        let level = ((self.0 >> Self::SKY_LIGHT_OFFSET) & Self::SKY_LIGHT_MASK) as u8;
        assert!(level <= 15);
        LightLevel::try_from(level).unwrap_or(LightLevel::Level15)
    }

    #[inline]
    #[must_use]
    pub fn block_light(&self) -> LightLevel {
        let level = ((self.0 >> Self::BLOCK_LIGHT_OFFSET) & Self::BLOCK_LIGHT_MASK) as u8;
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

impl TryFrom<&[u8]> for DataPoint {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let v = u64::from_be_bytes(value.try_into().context("invalid datapoint")?);
        Ok(Self(v))
    }
}

impl TryFrom<&[u8]> for Columns<Box<[super::data::DataPoint]>> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(mut data: &[u8]) -> Result<Self, Self::Error> {
        use super::data::DataPoint;
        use std::io::{BufRead, Read};
        use std::mem::MaybeUninit;
        let mut cols: [MaybeUninit<_>; Self::LEN] = std::array::from_fn(|_| MaybeUninit::uninit());
        for col in &mut cols {
            let mut len = [0, 0];
            data.read_exact(&mut len).context("reading u16")?;
            let len: usize = u16::from_be_bytes(len).into();

            anyhow::ensure!(data.len() >= len * 8, "not enough data for column");

            let points = data
                .chunks_exact(8)
                .take(len)
                .map(DataPoint::try_from)
                .collect::<Result<Box<[_]>, _>>()?;

            anyhow::ensure!(points.len() == len, "not enough points in column");

            col.write(points);

            data.consume(len * 8);
        }

        // if !data.is_empty() {
        //     anyhow::bail!("{} bytes left after reading all columns", data.len());
        // }

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
