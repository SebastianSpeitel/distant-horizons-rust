use std::sync::{Arc, Mutex};

use anyhow::Result;
use bevy::prelude::*;
use duckdb::Connection;

#[derive(Debug, Clone, Copy, Default)]
pub struct DuckPlugin;

impl Plugin for DuckPlugin {
    #[inline]
    fn build(&self, app: &mut App) {
        app.init_resource::<DuckDb>();
    }
}

#[derive(Debug, Clone, Resource)]
pub struct DuckDb {
    conn: Arc<Mutex<Connection>>,
}

impl Default for DuckDb {
    #[inline]
    fn default() -> Self {
        Self {
            conn: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
        }
    }
}

impl DuckDb {
    #[inline]
    pub fn lock(&self) -> impl std::ops::DerefMut<Target = Connection> {
        self.conn.lock().unwrap()
    }

    #[inline]
    pub fn attach_distant_horizons(&self, dh_path: impl AsRef<str>) -> Result<()> {
        let conn = self.lock();
        conn.execute("INSTALL SQLITE", [])?;
        conn.execute(
            &format!(
                "ATTACH '{}' AS dh (TYPE SQLITE, READONLY)",
                dh_path.as_ref()
            ),
            [],
        )?;
        conn.execute("SET sqlite_all_varchar=true", [])?;
        conn.execute("USE dh", [])?;

        Ok(())
    }
}
