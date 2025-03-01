use std::io::BufRead;

use anyhow::Result;
use simd_cesu8::mutf8::decode as decode_mutf8;

/// https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/io/DataInput.html#readUTF()
#[inline]
#[allow(non_snake_case)]
pub fn readUTF(mut input: impl BufRead) -> Result<String> {
    let mut len = [0, 0];
    input.read_exact(&mut len)?;
    let len: usize = u16::from_be_bytes(len).into();

    let mut buf = vec![0; len];
    input.read_exact(&mut buf)?;

    let utf = decode_mutf8(&buf[..len])?;

    Ok(utf.into_owned())
}
