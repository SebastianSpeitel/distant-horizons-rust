use std::{borrow::Cow, convert::Infallible};

use duckdb::{
    Result,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Compression {
    #[default]
    Uncompressed = 0,
    Lz4 = 1,
    #[deprecated]
    Zstd = 2,
    Lzma2 = 3,
}

impl FromSql for Compression {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match u8::column_result(value) {
            Err(e) => Err(e),
            Ok(0) => Ok(Self::Uncompressed),
            Ok(1) => Ok(Self::Lz4),
            Ok(2) => {
                eprintln!("Database contains unsupported zstd-compressed data");
                Ok(Self::Zstd)
            }
            Ok(3) => Ok(Self::Lzma2),
            _ => FromSqlResult::Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for Compression {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok((*self as u8).into())
    }
}

#[derive(Debug)]
pub enum Compressed<'a, T, C = Option<Infallible>> {
    Compressed {
        compressor: C,
        buf: Cow<'a, [u8]>,
    },
    Decompressed {
        compressor: C,
        val: T,
    },
    Cached {
        compressor: C,
        val: T,
        buf: Cow<'a, [u8]>,
    },
}

impl<T, C: Default> Default for Compressed<'_, T, C> {
    fn default() -> Self {
        Self::Compressed {
            compressor: C::default(),
            buf: Cow::Owned(Vec::new()),
        }
    }
}

impl<'a, T, C> Compressed<'a, T, C> {
    #[inline]
    pub const fn is_compressed(&self) -> bool {
        match self {
            Compressed::Compressed { .. } => true,
            Compressed::Cached { .. } => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_decompressed(&self) -> bool {
        match self {
            Compressed::Decompressed { .. } => true,
            Compressed::Cached { .. } => true,
            _ => false,
        }
    }

    #[inline]
    pub fn without_compressor(self) -> Compressed<'a, T> {
        match self {
            Self::Compressed { buf, .. } => Compressed::Compressed {
                compressor: None,
                buf,
            },
            Self::Decompressed { val, .. } => Compressed::Decompressed {
                compressor: None,
                val,
            },
            Self::Cached { val, buf, .. } => Compressed::Cached {
                compressor: None,
                val,
                buf,
            },
        }
    }

    #[inline]
    pub fn with_compressor<C2>(self, compressor: C2) -> Compressed<'a, T, C2> {
        match self {
            Self::Compressed { buf, .. } => Compressed::Compressed { compressor, buf },
            Self::Decompressed { val, .. } => Compressed::Decompressed { compressor, val },
            Self::Cached { val, buf, .. } => Compressed::Cached {
                compressor,
                val,
                buf,
            },
        }
    }

    #[inline]
    pub fn into_owned(self) -> Compressed<'static, T, C> {
        match self {
            Self::Compressed { compressor, buf } => Compressed::Compressed {
                compressor,
                buf: buf.into_owned().into(),
            },
            Self::Decompressed { compressor, val } => Compressed::Decompressed { compressor, val },
            Self::Cached {
                compressor,
                val,
                buf,
            } => Compressed::Cached {
                compressor,
                val,
                buf: buf.into_owned().into(),
            },
        }
    }

    #[inline]
    pub fn decompress_with<E>(&mut self, c: impl FnOnce(&[u8]) -> Result<T, E>) -> Result<&T, E>
    where
        Self: Default,
    {
        use core::mem::take;
        match self {
            Self::Decompressed { val, .. } => Ok(val),
            Self::Cached { val, .. } => Ok(val),
            Self::Compressed { .. } => {
                let Self::Compressed { buf, compressor } = take(self) else {
                    unreachable!();
                };

                let val = c(buf.as_ref())?;

                *self = Self::Cached {
                    compressor,
                    val,
                    buf,
                };

                let Self::Cached { val, .. } = self else {
                    unreachable!();
                };
                Ok(val)
            }
        }
    }

    #[inline]
    pub fn decompress(&mut self) -> Result<&T, anyhow::Error>
    where
        Self: Default,
        C: Decompressor<Error = anyhow::Error>,
        T: TryFrom<Box<[u8]>, Error = anyhow::Error>,
    {
        use core::mem::take;
        match self {
            Self::Decompressed { val, .. } => Ok(val),
            Self::Cached { val, .. } => Ok(val),
            Self::Compressed { .. } => {
                let Self::Compressed {
                    buf,
                    mut compressor,
                } = take(self)
                else {
                    unreachable!();
                };

                let val = compressor.decompress(buf.as_ref())?.try_into()?;

                *self = Self::Cached {
                    compressor,
                    val,
                    buf,
                };

                let Self::Cached { val, .. } = self else {
                    unreachable!();
                };
                Ok(val)
            }
        }
    }

    #[inline]
    pub fn drop_cache(&mut self)
    where
        Self: Default,
    {
        use core::mem::take;
        match self {
            Self::Cached { .. } => {
                let Self::Cached {
                    val, compressor, ..
                } = take(self)
                else {
                    unreachable!();
                };
                *self = Self::Decompressed { compressor, val }
            }
            _ => {}
        }
    }

    /// Returns a mutable reference to the decompressed value, if it is already decompressed.
    /// Invalidates the cache.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut T>
    where
        Self: Default,
    {
        self.drop_cache();
        match self {
            Self::Decompressed { val, .. } => Some(val),
            Self::Cached { val, .. } if cfg!(debug_assertions) => {
                debug_assert!(false, "Cached value should have been dropped");
                Some(val)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Self::Decompressed { val, .. } | Self::Cached { val, .. } => Some(val),
            Self::Compressed { .. } => None,
        }
    }
}

fn t(c: Compressed<Box<crate::section::columns::Columns<bool>>>) {}

impl<T, C> ToSql for Compressed<'_, T, C> {
    #[inline]
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        match self {
            Self::Compressed { buf, .. } | Self::Cached { buf, .. } => Ok(buf.as_ref().into()),
            Self::Decompressed { .. } => Err(duckdb::Error::ToSqlConversionFailure(
                "Value needs to be compressed first".into(),
            )),
        }
    }
}

impl<'a, T, C: Default> TryFrom<ValueRef<'a>> for Compressed<'a, T, C> {
    type Error = FromSqlError;

    #[inline]
    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Blob(buf) | ValueRef::Text(buf) => Ok(Self::Compressed {
                compressor: C::default(),
                buf: buf.into(),
            }),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[inline]
pub fn decompress_lzma(raw: &[u8]) -> Result<Box<[u8]>, anyhow::Error> {
    use std::io::Read;
    use xz2::bufread::XzDecoder;
    let mut decoder = XzDecoder::new(raw);
    let mut output = Vec::new();
    match decoder.read_to_end(&mut output) {
        Ok(b) if b == raw.len() => {}
        Ok(..) => {
            anyhow::bail!("failed to decompress LZMA: incomplete output");
        }
        // Ignore EOF
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {}
        Err(e) => {
            return Err(anyhow::Error::new(e).context("failed to decompress LZMA"));
        }
    }

    debug_assert_eq!(decoder.total_in(), raw.len() as u64);
    debug_assert_eq!(decoder.total_out(), output.len() as u64);

    Ok(output.into_boxed_slice())
}

#[inline]
pub fn compress_lzma(value: Box<[u8]>) -> Box<[u8]> {
    const LZMA_OPTIONS: core::cell::LazyCell<xz2::stream::LzmaOptions> =
        core::cell::LazyCell::new(|| xz2::stream::LzmaOptions::new_preset(9).unwrap());

    let mut encoder = xz2::stream::Stream::new_lzma_encoder(&LZMA_OPTIONS).unwrap();
    let mut output = Vec::new();
    encoder
        .process_vec(&value, &mut output, xz2::stream::Action::Finish)
        .unwrap();

    output.into()
}

pub trait Decompressor {
    type Error;

    fn decompress(&mut self, raw: impl AsRef<[u8]>) -> Result<Box<[u8]>, Self::Error>;
}

impl Decompressor for Compression {
    type Error = anyhow::Error;

    #[inline]
    fn decompress(&mut self, raw: impl AsRef<[u8]>) -> Result<Box<[u8]>, Self::Error> {
        match *self {
            Self::Uncompressed => Ok(raw.as_ref().into()),
            Self::Lzma2 => decompress_lzma(raw.as_ref()).map_err(Into::into),
            _ => todo!(),
        }
    }
}

// pub trait Compressor<T> {
//     type Error;

//     fn decompress(&mut self, raw: impl AsRef<[u8]>) -> Result<T, Self::Error>;

//     fn compress(&mut self, value: T) -> Result<impl AsRef<[u8]>, Self::Error>;
// }

// impl Compressor<Vec<u8>> for Compression {
//     type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

//     #[inline]
//     fn decompress(&mut self, raw: impl AsRef<[u8]>) -> Result<Vec<u8>, Self::Error> {
//         match *self {
//             Compression::Uncompressed => Ok(raw.as_ref().to_vec()),
//             Compression::Lzma2 => {
//                 let mut decoder = xz2::stream::Stream::new_lzma_decoder(2048)?;
//                 let mut output = Vec::new();
//                 decoder.process_vec(raw.as_ref(), &mut output, xz2::stream::Action::Finish)?;

//                 Ok(output)
//             }
//             _ => todo!(),
//         }
//     }

//     #[inline]
//     fn compress(&mut self, value: Vec<u8>) -> Result<impl AsRef<[u8]>, Self::Error> {
//         const LZMA_OPTIONS: core::cell::LazyCell<xz2::stream::LzmaOptions> =
//             core::cell::LazyCell::new(|| xz2::stream::LzmaOptions::new_preset(9).unwrap());

//         match *self {
//             Compression::Uncompressed => Ok(value),
//             Compression::Lzma2 => {
//                 let mut encoder = xz2::stream::Stream::new_lzma_encoder(&LZMA_OPTIONS)?;
//                 let mut output = Vec::new();
//                 encoder.process_vec(&value, &mut output, xz2::stream::Action::Finish)?;

//                 Ok(output)
//             }
//             _ => todo!(),
//         }
//     }
// }
