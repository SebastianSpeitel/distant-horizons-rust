#[cfg(feature = "gui")]
fn main() {
    distant_horizons::gui::main();
}

#[cfg(not(feature = "gui"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use duckdb::Connection;

    use distant_horizons::{DetailLevel, FullDataSourceV2DTO};

    let db_path = std::env::args()
        .nth(1)
        .ok_or("Missing argument <db_path>")?;
    dbg!(&db_path);

    let conn = Connection::open_in_memory()?;

    conn.execute("INSTALL SQLITE", [])?;
    conn.execute(
        &format!("ATTACH '{db_path}' AS dh (TYPE SQLITE, READONLY)"),
        [],
    )?;
    conn.execute("SET sqlite_all_varchar=true", [])?;
    conn.execute("USE dh", [])?;

    let all_data = FullDataSourceV2DTO::get_all(&conn)?;

    let mut min_detail = DetailLevel::MAX;
    let mut max_detail = DetailLevel::MIN;

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;

    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    for data in all_data {
        let pos = data.section_pos();
        if pos.detail_level() != DetailLevel::Region {
            continue;
        }
        dbg!(pos.to_string());

        // assert_eq!(data.section_pos().detail_level(), data.detail_level + 2);
        assert_eq!(pos.x(), data.pos.x);
        assert_eq!(pos.z(), data.pos.z);

        min_detail = min_detail.min(data.detail_level);
        max_detail = max_detail.max(data.detail_level);

        min_x = min_x.min(pos.center_block_pos_x());
        max_x = max_x.max(pos.center_block_pos_x());

        min_z = min_z.min(pos.center_block_pos_z());
        max_z = max_z.max(pos.center_block_pos_z());
    }

    dbg!(min_detail, max_detail);
    dbg!(min_x, max_x, min_z, max_z);
    Ok(())
}
