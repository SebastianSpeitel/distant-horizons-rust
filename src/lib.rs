pub mod block;
mod compression;
mod detail_level;
#[cfg(feature = "gui")]
pub mod gui;
mod java;
mod light;
mod repo;
pub mod section;

pub use detail_level::DetailLevel;
pub use section::Section;

pub mod minecraft {
    use std::env::var;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    macro_rules! from_env {
        ($var:literal) => {
            match var($var) {
                Ok(v) => Some(v.into()),
                Err(_) => option_env!($var).map(Into::into),
            }
        };
    }

    pub static MINECRAFT_PATH: LazyLock<Option<PathBuf>> =
        LazyLock::new(|| from_env!("MINECRAFT_PATH"));

    pub static MINECRAFT_WORLD: LazyLock<Option<String>> =
        LazyLock::new(|| from_env!("MINECRAFT_WORLD"));
}
