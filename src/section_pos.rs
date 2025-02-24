/// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/pos/DhSectionPos.java
use crate::DetailLevel;

pub const DETAIL_LEVEL_WIDTH: usize = 8;
pub const X_POS_WIDTH: usize = 28;
pub const Z_POS_WIDTH: usize = 28;
pub const X_POS_MISSING_WIDTH: usize = 32 - 28;
pub const Z_POS_MISSING_WIDTH: usize = 32 - 28;

pub const DETAIL_LEVEL_OFFSET: usize = 0;
pub const POS_X_OFFSET: usize = DETAIL_LEVEL_OFFSET + DETAIL_LEVEL_WIDTH;
pub const POS_Z_OFFSET: usize = POS_X_OFFSET + X_POS_WIDTH;

pub const DETAIL_LEVEL_MASK: i64 = u8::MAX as i64;
pub const POS_X_MASK: i32 = (1 << X_POS_WIDTH) - 1;
pub const POS_Z_MASK: i32 = (1 << Z_POS_WIDTH) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionPos(i64);

impl SectionPos {
    pub const SECTION_MINIMUM_DETAIL_LEVEL: DetailLevel = DetailLevel::Chunk4;

    #[inline]
    pub const fn new(section_detail_level: DetailLevel, x: i32, z: i32) -> Self {
        assert!(x.unsigned_abs() < 1 << X_POS_WIDTH);
        assert!(z.unsigned_abs() < 1 << Z_POS_WIDTH);

        let mut data = 0;
        data |= section_detail_level as i64 & DETAIL_LEVEL_MASK;
        data |= ((x & POS_X_MASK) as i64) << POS_X_OFFSET;
        data |= ((z & POS_Z_MASK) as i64) << POS_Z_OFFSET;

        Self(data)
    }

    #[inline]
    pub const fn detail_level(self) -> DetailLevel {
        let dl = ((self.0 >> DETAIL_LEVEL_OFFSET) & DETAIL_LEVEL_MASK) as u8;
        assert!(dl <= DetailLevel::MAX as u8);
        unsafe { core::mem::transmute(dl) }
    }

    #[inline]
    pub const fn x(self) -> i32 {
        let x = ((self.0 >> POS_X_OFFSET) & POS_X_MASK as i64) as i32;
        // if at least one of the first 4 bits is neither 0 for positive nor 1 for negative, this won't work
        (x << X_POS_MISSING_WIDTH) >> X_POS_MISSING_WIDTH
    }

    #[inline]
    pub const fn z(self) -> i32 {
        let z = ((self.0 >> POS_Z_OFFSET) & POS_Z_MASK as i64) as i32;
        (z << Z_POS_MISSING_WIDTH) >> Z_POS_MISSING_WIDTH
    }

    #[inline]
    pub const fn min_corner_block_x(self) -> i32 {
        match self.detail_level() {
            DetailLevel::Block2 => self.center_block_pos_x(),
            _ => self.center_block_pos_x() - self.block_width() / 2,
        }
    }

    #[inline]
    pub const fn min_corner_block_z(self) -> i32 {
        match self.detail_level() {
            DetailLevel::Block2 => self.center_block_pos_z(),
            _ => self.center_block_pos_z() - self.block_width() / 2,
        }
    }

    #[inline]
    pub const fn block_width(self) -> i32 {
        self.detail_level().block_width()
    }

    #[inline]
    pub const fn center_block_pos_x(self) -> i32 {
        let x = self.x();

        match self.detail_level() {
            DetailLevel::Block => x,
            DetailLevel::Block2 => x * 2,
            d => (x << d as u8) + (1 << (d as u8 - 1)),
        }
    }

    #[inline]
    pub const fn center_block_pos_z(self) -> i32 {
        let z = self.z();

        match self.detail_level() {
            DetailLevel::Block => z,
            DetailLevel::Block2 => z * 2,
            d => (z << d as u8) + (1 << (d as u8 - 1)),
        }
    }
}

impl core::fmt::Display for SectionPos {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.debug_struct("Pos")
                .field("x", &self.x())
                .field("z", &self.z())
                .field("detail_level", &self.detail_level())
                .finish()
        } else {
            write!(f, "{:?}*{},{}", self.detail_level(), self.x(), self.z())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let pos = SectionPos::new(DetailLevel::Block, 0, 0);
        assert_eq!(pos.detail_level(), DetailLevel::Block);
        assert_eq!(pos.x(), 0);
        assert_eq!(pos.z(), 0);
        assert_eq!(pos.min_corner_block_x(), 0);
        assert_eq!(pos.min_corner_block_z(), 0);
        assert_eq!(pos.block_width(), 1);
        assert_eq!(pos.center_block_pos_x(), 0);
        assert_eq!(pos.center_block_pos_z(), 0);
    }

    #[test]
    fn maximum() {
        let x = POS_X_MASK >> 1 as i32;
        let z = POS_Z_MASK >> 1 as i32;

        let pos = SectionPos::new(DetailLevel::Block, x, z);
        assert_eq!(pos.x(), x);
        assert_eq!(pos.z(), z);
    }

    #[test]
    fn negative() {
        let x = -2;
        let z = -2;

        let pos = SectionPos::new(DetailLevel::Block, x, z);
        assert_eq!(pos.x(), x);
        assert_eq!(pos.z(), z);
    }

    #[test]
    #[should_panic]
    fn too_small() {
        let x = i32::MIN;
        let z = i32::MIN;

        let _pos = SectionPos::new(DetailLevel::Block, x, z);
    }

    #[test]
    #[should_panic]
    fn too_large() {
        let x = i32::MAX;
        let z = i32::MAX;

        let _pos = SectionPos::new(DetailLevel::Block, x, z);
    }
}
