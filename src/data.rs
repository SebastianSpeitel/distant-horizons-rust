use duckdb::{Error, Row};

use crate::section_pos::SectionPos;
use crate::DetailLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug)]
pub struct FullDataSourceV2DTO {
    pub detail_level: DetailLevel,
    pub pos: Pos,
    // levelMinY
    min_y: i32,
    // dataChecksum
    checksum: i32,
    // compressedDataByteArray
    data: Vec<u8>,
    // compressedColumnGenStepByteArray
    column_gen_step: Vec<u8>,
    // compressedWorldCompressionModeByteArray
    world_compression_mode: Vec<u8>,
    // compressedMappingByteArray
    mapping: Vec<u8>,
    // dataFormatVersion
    format_version: u8,
    // compressionModeValue
    compression_mode: u8,
    // applyToParent
    apply_to_parent: Option<bool>,
    // applyToChildren
    apply_to_children: Option<bool>,
    // lastModifiedUnixDateTime
    last_modified: i64,
    // createdUnixDateTime
    created: i64,
}

impl FullDataSourceV2DTO {
    pub fn get_all(conn: &duckdb::Connection) -> Result<Vec<Self>, duckdb::Error> {
        const SQL: &str = "SELECT * FROM FullData";
        let mut stmt = conn.prepare_cached(SQL)?;

        let mut rows = stmt.query([])?;
        let Some(first) = rows.next()? else {
            return Ok(Vec::new());
        };
        let mut all = Vec::with_capacity(first.get("count").unwrap_or_default());
        all.push(Self::try_from(first)?);

        while let Some(row) = rows.next()? {
            all.push(Self::try_from(row)?);
        }

        Ok(all)
    }

    #[inline]
    pub const fn block_width(&self) -> i32 {
        self.detail_level.block_width()
    }
}

impl FullDataSourceV2DTO {
    #[inline]
    pub const fn section_pos(&self) -> SectionPos {
        SectionPos::new(self.detail_level, self.pos.x, self.pos.z)
    }
}

// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/sql/repo/FullDataSourceV2Repo.java
impl TryFrom<&Row<'_>> for FullDataSourceV2DTO {
    type Error = Error;

    #[inline]
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let mut detail_level = row.get("DetailLevel")?;
        detail_level = detail_level + SectionPos::SECTION_MINIMUM_DETAIL_LEVEL;
        let x: i32 = row.get("PosX")?;
        let z: i32 = row.get("PosZ")?;

        let min_y: i32 = row.get("MinY")?;
        let checksum: i32 = row.get("DataChecksum")?;

        let format_version: u8 = row.get("DataFormatVersion")?;
        let compression_mode: u8 = row.get("CompressionMode")?;

        let apply_to_parent: Option<bool> = row.get("ApplyToParent")?;
        let apply_to_children: Option<bool> = row.get("ApplyToChildren").unwrap_or_default();

        let last_modified: i64 = row.get("LastModifiedUnixDateTime")?;
        let created: i64 = row.get("CreatedUnixDateTime")?;

        let data: Vec<u8> = row.get("Data")?;
        let column_gen_step: Vec<u8> = row.get("ColumnGenerationStep")?;
        let world_compression_mode: Vec<u8> = row.get("ColumnWorldCompressionMode")?;
        let mapping: Vec<u8> = row.get("Mapping")?;

        Ok(Self {
            detail_level,
            pos: Pos { x, z },
            min_y,
            checksum,
            data,
            column_gen_step,
            world_compression_mode,
            mapping,
            format_version,
            compression_mode,
            apply_to_parent,
            apply_to_children,
            last_modified,
            created,
        })
    }
}
