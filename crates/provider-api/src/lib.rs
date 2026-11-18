//! The contract between StarVizion and anything that feeds it: a provider announces the fields
//! it offers (name, type, unit, range) and streams their values, and may hand over whole cards
//! to display. This is the API third-party developers write against, so it is versioned and
//! changes only compatibly.
