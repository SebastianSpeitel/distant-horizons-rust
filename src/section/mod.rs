use std::collections::HashMap;

use anyhow::Context;
use duckdb::Row;

pub mod columns;
pub mod compression;
pub mod data;
pub mod mapping;
pub mod pos;
pub mod world_gen_step;

use columns::Columns;
use pos::Pos;

use crate::compression::{Compressed, Compression};

#[derive(Debug)]
pub struct Section<'a> {
    pub pos: Pos,
    // levelMinY
    min_y: i32,
    // dataChecksum
    checksum: i32,
    // compressedDataByteArray
    data: Compressed<'a, Box<Columns<Box<[data::DataPoint]>>>, Compression>,
    // compressedColumnGenStepByteArray
    world_gen_step: Compressed<'a, Columns<world_gen_step::WorldGenStep>, Compression>,
    // compressedWorldCompressionModeByteArray
    world_compression: Compressed<'a, Columns<compression::WorldCompression>, Compression>,
    // compressedMappingByteArray
    mapping: Compressed<'a, mapping::Mapping, Compression>,
    // dataFormatVersion
    format_version: u8,
    // compressionModeValue
    compression: Compression,
    // applyToParent
    apply_to_parent: Option<bool>,
    // applyToChildren
    apply_to_children: Option<bool>,
    // lastModifiedUnixDateTime
    last_modified: i64,
    // createdUnixDateTime
    created: i64,
}

impl<'a> Section<'a> {
    pub const WIDTH: usize = 64;

    #[inline]
    #[must_use]
    pub const fn compression(&self) -> Compression {
        self.compression
    }

    #[inline]
    #[must_use]
    pub fn column_data(&self) -> Option<&Columns<Box<[data::DataPoint]>>> {
        self.data.as_ref().map(|data| data.as_ref())
    }

    #[inline]
    #[must_use]
    pub fn mapping(&self) -> Option<&mapping::Mapping> {
        self.mapping.as_ref()
    }

    #[inline]
    pub fn get_all_from_db(db_path: impl AsRef<str>) -> Result<Vec<Self>, duckdb::Error> {
        let conn = duckdb::Connection::open_in_memory()?;
        conn.execute("INSTALL SQLITE", [])?;
        conn.execute(
            &format!(
                "ATTACH '{}' AS dh (TYPE SQLITE, READONLY)",
                db_path.as_ref()
            ),
            [],
        )?;
        conn.execute("SET sqlite_all_varchar=true", [])?;
        conn.execute("USE dh", [])?;

        Self::get_all(&conn)
    }

    #[inline]
    pub fn get_all(conn: &duckdb::Connection) -> Result<Vec<Self>, duckdb::Error> {
        const SQL: &str = "SELECT * FROM FullData";
        let mut stmt = conn.prepare_cached(SQL)?;

        let mut rows = stmt.query([])?;

        let Some(first) = rows.next()? else {
            return Ok(Vec::new());
        };
        let mut all = Vec::with_capacity(first.get("count").unwrap_or_default());
        all.push(Section::from_row(first)?.into_owned());

        while let Some(row) = rows.next()? {
            all.push(Section::from_row(row)?.into_owned());
        }

        Ok(all)
    }

    /// https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/sql/repo/FullDataSourceV2Repo.java
    #[inline]
    pub fn from_row(row: &'a Row) -> Result<Self, duckdb::Error> {
        let mut detail_level = row.get("DetailLevel")?;
        detail_level = detail_level + Pos::SECTION_MINIMUM_DETAIL_LEVEL;
        let x: i32 = row.get("PosX")?;
        let z: i32 = row.get("PosZ")?;

        let min_y: i32 = row.get("MinY")?;
        let checksum: i32 = row.get("DataChecksum")?;

        let format_version: u8 = row.get("DataFormatVersion")?;
        let compression = row.get("CompressionMode")?;

        let apply_to_parent: Option<bool> = row.get("ApplyToParent").unwrap_or_default();
        let apply_to_children: Option<bool> = row.get("ApplyToChildren").unwrap_or_default();

        let last_modified: i64 = row.get("LastModifiedUnixDateTime")?;
        let created: i64 = row.get("CreatedUnixDateTime")?;

        let data = row.get_ref("Data")?;
        let data: Compressed<_> = data.try_into()?;
        let world_gen_step: Compressed<_> = row.get_ref("ColumnGenerationStep")?.try_into()?;
        let world_compression: Compressed<_> =
            row.get_ref("ColumnWorldCompressionMode")?.try_into()?;
        let mapping: Compressed<_> = row.get_ref("Mapping")?.try_into()?;

        let data = data.with_compressor(compression);
        let world_gen_step = world_gen_step.with_compressor(compression);
        let world_compression = world_compression.with_compressor(compression);
        let mapping = mapping.with_compressor(compression);

        Ok(Self {
            pos: Pos { detail_level, x, z },
            min_y,
            checksum,
            data,
            world_gen_step,
            world_compression,
            mapping,
            format_version,
            compression,
            apply_to_parent,
            apply_to_children,
            last_modified,
            created,
        })
    }

    #[inline]
    #[must_use]
    pub fn into_owned(self) -> Section<'static> {
        Section {
            pos: self.pos,
            min_y: self.min_y,
            checksum: self.checksum,
            data: self.data.into_owned(),
            world_gen_step: self.world_gen_step.into_owned(),
            world_compression: self.world_compression.into_owned(),
            mapping: self.mapping.into_owned(),
            format_version: self.format_version,
            compression: self.compression,
            apply_to_parent: self.apply_to_parent,
            apply_to_children: self.apply_to_children,
            last_modified: self.last_modified,
            created: self.created,
        }
    }

    #[inline]
    pub fn decompress(&mut self) -> Result<(), anyhow::Error> {
        self.data.decompress().context("decompressing data")?;
        self.world_gen_step
            .decompress()
            .context("decompressing world_gen_steps")?;
        self.world_compression
            .decompress()
            .context("decompressing world compression")?;
        self.mapping.decompress().context("decompressing mapping")?;

        Ok(())
    }

    #[inline]
    pub fn drop_caches(&mut self) {
        self.data.drop_cache();
        self.world_gen_step.drop_cache();
        self.world_compression.drop_cache();
        self.mapping.drop_cache();
    }

    #[inline]
    #[must_use]
    pub const fn block_width(&self) -> i32 {
        self.pos.detail_level.block_width()
    }
}

impl Section<'_> {
    pub fn insert_into(&self, conn: &duckdb::Connection) -> Result<(), duckdb::Error> {
        const SQL: &str = "
            INSERT INTO FullData BY NAME (
                SELECT
                    ? AS DetailLevel,
                    ? AS PosX,
                    ? AS PosZ,
                    ? AS MinY,
                    ? AS DataChecksum,
                    ? AS Data,
                    ? AS ColumnGenerationStep,
                    ? AS ColumnWorldCompressionMode,
                    ? AS Mapping,
                    ? AS DataFormatVersion,
                    ? AS CompressionMode,
                    ? AS ApplyToParent,
                    ? AS LastModifiedUnixDateTime,
                    ? AS CreatedUnixDateTime,

                    -- ? AS ApplyToChildren,
            );
        ";
        let mut stmt = conn.prepare_cached(SQL)?;

        stmt.execute(duckdb::params![
            self.pos.detail_level - Pos::SECTION_MINIMUM_DETAIL_LEVEL,
            self.pos.x,
            self.pos.z,
            self.min_y,
            self.checksum,
            self.data,
            self.world_gen_step,
            self.world_compression,
            self.mapping,
            self.format_version,
            self.compression,
            self.apply_to_parent,
            self.last_modified,
            self.created,
        ])?;
        Ok(())
    }
}
