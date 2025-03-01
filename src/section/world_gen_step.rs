/// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/api/src/main/java/com/seibel/distanthorizons/api/enums/worldGeneration/EDhApiWorldGenerationStep.java
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum WorldGenStep {
    DownSampled = u8::MAX.wrapping_sub(1),
    Empty = 0,
    StructureStart = 1,
    StructureReference = 2,
    Biomes = 3,
    Noise = 4,
    Surface = 5,
    Carvers = 6,
    LiquidCarvers = 7,
    Features = 8,
    Light = 9,
}

impl TryFrom<u8> for WorldGenStep {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v @ (0..=9 | 254) => Ok(unsafe { std::mem::transmute::<u8, Self>(v) }),
            _ => anyhow::bail!("invalid value: {value}"),
        }
    }
}

impl AsRef<str> for WorldGenStep {
    #[inline]
    fn as_ref(&self) -> &str {
        match *self {
            Self::DownSampled => "down_sampled",
            Self::Empty => "empty",
            Self::StructureStart => "structure_start",
            Self::StructureReference => "structure_reference",
            Self::Biomes => "biomes",
            Self::Noise => "noise",
            Self::Surface => "surface",
            Self::Carvers => "carvers",
            Self::LiquidCarvers => "liquid_carvers",
            Self::Features => "features",
            Self::Light => "light",
        }
    }
}

impl TryFrom<Box<[u8]>> for super::columns::Columns<WorldGenStep> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        use anyhow::Context;

        let mut data_iter = data.into_iter();
        let mut cols = [WorldGenStep::Empty; Self::LEN];
        cols.iter_mut().try_for_each(|col| {
            let next = data_iter.next().context("not enough values")?;
            *col = next.try_into()?;
            anyhow::Ok(())
        })?;
        // anyhow::ensure!(data_iter.next().is_none(), "too many values");
        Ok(Self::new(cols))
    }
}
