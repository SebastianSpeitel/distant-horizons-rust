/// https://minecraft.wiki/w/Light
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LightLevel {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
    Level8 = 8,
    Level9 = 9,
    Level10 = 10,
    Level11 = 11,
    Level12 = 12,
    Level13 = 13,
    Level14 = 14,
    Level15 = 15,
}

impl LightLevel {
    pub const SUN: Self = Self::Level15;
}

impl TryFrom<u8> for LightLevel {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v @ (0..=15) => Ok(
                // Safety: value is guaranteed to be a valid light level
                unsafe { core::mem::transmute::<u8, Self>(v) },
            ),
            _ => anyhow::bail!("invalid value: {value}"),
        }
    }
}
