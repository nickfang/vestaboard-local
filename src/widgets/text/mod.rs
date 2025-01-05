pub mod text;
// this is just so main can use text::get_text instead of text::text::get_text
pub use text::get_text;
pub use text::get_text_from_file;

#[cfg(test)]
pub mod text_tests;
