use super::formatter::Formatter;

/// This structure compacts an EnCom value with no extra whitespace.
#[derive(Clone, Debug)]
pub(crate) struct CompactFormatter;

impl Formatter for CompactFormatter {}
