use duckdb::Connection;

use distant_horizons::{DetailLevel, Section};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = std::env::var("DH_PATH").unwrap_or_else(|_| "DistantHorizons.sqlite".to_string());

    #[cfg(feature = "gui")]
    if std::env::args().nth(1).as_deref() == Some("GUI") {
        distant_horizons::gui::main();
        return Ok(());
    }

    dbg!(&db_path);

    let conn = Connection::open_in_memory()?;

    conn.execute("INSTALL SQLITE", [])?;
    conn.execute(
        &format!("ATTACH '{db_path}' AS dh (TYPE SQLITE, READONLY)"),
        [],
    )?;
    conn.execute("SET sqlite_all_varchar=true", [])?;
    conn.execute("USE dh", [])?;

    println!("Connected to database");

    let mut all_sections = Section::get_all(&conn)?;

    let mut min_detail = DetailLevel::MAX;
    let mut max_detail = DetailLevel::MIN;

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;

    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    for section in &mut all_sections {
        if section.pos.detail_level != DetailLevel::Chunk4 {
            continue;
        }

        dbg!(section.pos.to_string());
        dbg!(section.compression());
        if let Err(e) = section.decompress() {
            eprintln!("Failed to decompress section: {e:#?}");
            continue;
        };

        // let cols = section.column_data().unwrap();
        // let mapping = section.mapping().unwrap();
        // let col = &cols[(0, 0)];
        // for point in col {
        //     dbg!(point);
        //     dbg!(&mapping[point]);
        // }

        let pos = section.pos;
        // if pos.detail_level != DetailLevel::Region {
        //     continue;
        // }

        min_detail = min_detail.min(section.pos.detail_level);
        max_detail = max_detail.max(section.pos.detail_level);

        min_x = min_x.min(pos.center_x());
        max_x = max_x.max(pos.center_x());

        min_z = min_z.min(pos.center_z());
        max_z = max_z.max(pos.center_z());
    }

    dbg!(min_detail, max_detail);
    dbg!(min_x, max_x, min_z, max_z);

    distant_horizons::section::mapping::print_interned_sizes();

    // let Some(db_dest) = std::env::args().nth(2) else {
    //     return Ok(());
    // };

    // let conn_dest = Connection::open_in_memory()?;
    // conn_dest.execute("INSTALL SQLITE", [])?;
    // conn_dest.execute(
    //     &format!("ATTACH '{db_dest}' AS dh (TYPE SQLITE, READWRITE)"),
    //     [],
    // )?;
    // conn_dest.execute("SET sqlite_all_varchar=true", [])?;
    // conn_dest.execute("USE dh", [])?;

    // for data in all_data {
    //     data.insert_into(&conn_dest).unwrap_or_else(|e| {
    //         eprintln!("Failed to insert data: {e:?}");
    //     });
    // }

    Ok(())
}
