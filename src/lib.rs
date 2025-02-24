mod data;
mod detail_level;
#[cfg(feature = "gui")]
pub mod gui;
mod section_pos;

pub use data::FullDataSourceV2DTO;
pub use detail_level::DetailLevel;
