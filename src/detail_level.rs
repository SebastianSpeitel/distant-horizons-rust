#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[repr(u8)]
pub enum DetailLevel {
    #[default]
    Block = 0,
    Block2 = 1,
    Block4 = 2,
    Block8 = 3,
    Chunk = 4,
    Chunk2 = 5,
    Chunk4 = 6,
    Chunk8 = 7,
    Chunk16 = 8,
    Region = 9,
    Region2 = 10,
    Region4 = 11,
    Region8 = 12,
    Region16 = 13,
    Region32 = 14,
    Region64 = 15,
    Region128 = 16,
    Region256 = 17,
    Region512 = 18,
}

impl DetailLevel {
    pub const MIN: Self = Self::Block;
    pub const MAX: Self = Self::Region512;

    #[inline]
    pub const fn try_new(level: u8) -> Result<Self, ()> {
        if level > Self::MAX as u8 {
            Err(())
        } else {
            // Safety: level is guaranteed to be a valid variant discriminant
            Ok(unsafe { core::mem::transmute::<u8, Self>(level) })
        }
    }

    /// # Safety
    ///
    /// level must be a valid variant discriminant
    #[inline]
    #[must_use]
    pub const unsafe fn new_unchecked(level: u8) -> Self {
        debug_assert!(level <= Self::MAX as u8);
        // Safety: the caller guarantees that level is a valid variant discriminant
        unsafe { core::mem::transmute(level) }
    }

    #[inline]
    #[must_use]
    pub const fn block_width(self) -> i32 {
        1 << (self as u8)
    }
}

impl duckdb::ToSql for DetailLevel {
    #[inline]
    fn to_sql(&self) -> duckdb::Result<duckdb::types::ToSqlOutput<'_>> {
        use duckdb::types::{ToSqlOutput, Value};
        Ok(ToSqlOutput::Owned(Value::from(*self as u8)))
    }
}

impl duckdb::types::FromSql for DetailLevel {
    #[inline]
    fn column_result(value: duckdb::types::ValueRef<'_>) -> duckdb::types::FromSqlResult<Self> {
        let value = u8::column_result(value)?;
        Self::try_new(value).map_err(|_| duckdb::types::FromSqlError::InvalidType)
    }
}

impl core::ops::Add for DetailLevel {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let result = self as u8 + rhs as u8;
        assert!(!(result > Self::MAX as u8), "DetailLevel overflow");
        unsafe { core::mem::transmute(result) }
    }
}

impl core::ops::Sub for DetailLevel {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let result = (self as u8).saturating_sub(rhs as u8);
        unsafe { core::mem::transmute(result) }
    }
}
