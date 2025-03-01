/// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/api/src/main/java/com/seibel/distanthorizons/api/enums/config/EDhApiWorldCompressionMode.java
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum WorldCompression {
    MergeSameBlock = 0,
    VisuallyEqual = 1,
}

impl TryFrom<u8> for WorldCompression {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::MergeSameBlock),
            1 => Ok(Self::VisuallyEqual),
            _ => anyhow::bail!("invalid value: {value}"),
        }
    }
}

impl TryFrom<Box<[u8]>> for super::columns::Columns<WorldCompression> {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(data: Box<[u8]>) -> Result<Self, Self::Error> {
        use anyhow::Context;

        let mut data_iter = data.into_iter();
        let mut cols = [WorldCompression::MergeSameBlock; Self::LEN];
        cols.iter_mut().try_for_each(|col| {
            let next = data_iter.next().context("not enough values")?;
            *col = next.try_into()?;
            anyhow::Ok(())
        })?;
        // anyhow::ensure!(data_iter.next().is_none(), "too many values");
        Ok(Self::new(cols))
    }
}
