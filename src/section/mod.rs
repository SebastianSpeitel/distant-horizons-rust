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

use crate::{
    compression::{Compressed, Compression},
    repo::Query,
};

#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[cfg_attr(feature = "bevy", require(bevy::prelude::Transform))]
pub struct Section<'a> {
    pub pos: Pos,
    // levelMinY
    pub min_y: i32,
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

impl Section<'_> {
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

    pub fn get_all_with_detail_level_from_db(
        db_path: impl AsRef<str>,
        detail_level: crate::DetailLevel,
    ) -> Result<Vec<Self>, duckdb::Error> {
        use crate::repo::Repo;

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

        struct Q;

        impl Query<crate::DetailLevel> for Q {
            fn r#where(&self) -> &str {
                "DetailLevel = ?"
            }

            fn bind_params(&self, stmt: &mut duckdb::Statement, params: crate::DetailLevel) {
                stmt.raw_bind_parameter(1, params as i32).unwrap();
            }
        }

        Self::select_vec_with(
            &conn,
            &Q,
            detail_level - Pos::SECTION_MINIMUM_DETAIL_LEVEL,
            |s| s.into_owned(),
        )
    }

    #[inline]
    pub fn get_all(conn: &duckdb::Connection) -> Result<Vec<Self>, duckdb::Error> {
        use crate::repo::{All, Repo};

        let q = All.ordered("(cast(PosX as INTEGER) ** 2 + cast(PosZ as INTEGER) ** 2) DESC");

        Self::select_vec(conn, &q, |s| s.into_owned())
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
    pub const fn is_compressed(&self) -> bool {
        self.data.is_compressed()
            || self.world_gen_step.is_compressed()
            || self.world_compression.is_compressed()
            || self.mapping.is_compressed()
    }

    #[inline]
    pub const fn is_decompressed(&self) -> bool {
        self.data.is_decompressed()
            && self.world_gen_step.is_decompressed()
            && self.world_compression.is_decompressed()
            && self.mapping.is_decompressed()
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

impl crate::repo::Repo for Section<'_> {
    const TABLE: &'static str = "FullData";
    const INSERT: &'static str = "
            BY NAME (
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

    type Element<'r> = Section<'r>;

    /// <https://gitlab.com/distant-horizons-team/distant-horizons-core/-/blob/main/core/src/main/java/com/seibel/distanthorizons/core/sql/repo/FullDataSourceV2Repo.java>
    #[inline]
    fn from_row<'r>(row: &'r Row) -> duckdb::Result<Self::Element<'r>> {
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

        Ok(Section {
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
    fn bind_insert(stmt: &mut duckdb::Statement, sec: Self::Element<'_>) -> duckdb::Result<()> {
        macro_rules! bind {
            (@param $i:ident = $p:expr) => {
                $i += 1;
                stmt.raw_bind_parameter($i, $p)?;
            };
            ($($p:expr$(,)?)*) => {
                {
                    let mut col_index = 0;
                    $(
                        bind!(@param col_index = $p);
                    )*
                    debug_assert_eq!(stmt.parameter_count(), col_index);
                }
            };
        }

        bind![
            sec.pos.detail_level - Pos::SECTION_MINIMUM_DETAIL_LEVEL,
            sec.pos.x,
            sec.pos.z,
            sec.min_y,
            sec.checksum,
            sec.data,
            sec.world_gen_step,
            sec.world_compression,
            sec.mapping,
            sec.format_version,
            sec.compression,
            sec.apply_to_parent,
            sec.last_modified,
            sec.created
        ];

        Ok(())
    }
}
