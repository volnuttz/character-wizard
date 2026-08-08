//! Character-wizard application library.

pub(crate) mod app_error;
pub mod cli;
pub(crate) mod creation;
pub(crate) mod data_pack;
pub(crate) mod domain;
pub(crate) mod pdf_renderer;
pub(crate) mod rules;
pub(crate) mod share;
pub(crate) mod srd_data;
pub(crate) mod storage;
pub(crate) mod template;

pub(crate) use self::creation as character_wizard_creation;
pub(crate) use self::domain as character_wizard_domain;
pub(crate) use self::domain::Character;
pub(crate) use self::pdf_renderer as character_wizard_pdf_renderer;
pub(crate) use self::srd_data as character_wizard_srd_data;
