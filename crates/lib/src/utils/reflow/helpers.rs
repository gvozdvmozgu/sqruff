use smol_str::{SmolStr, ToSmolStr};

use crate::core::parser::segments::base::ErasedSegment;
use crate::core::rules::base::{LintFix, LintResult};
use crate::dialects::SyntaxKind;

/// Return a list of fixes from an iterable of LintResult.
pub fn fixes_from_results(
    results: impl Iterator<Item = LintResult>,
) -> impl Iterator<Item = LintFix> {
    results.flat_map(|result| result.fixes)
}

/// Given a raw segment, deduce the indent of its line.
pub fn deduce_line_indent(raw_segment: &ErasedSegment, root_segment: &ErasedSegment) -> SmolStr {
    let seg_idx = root_segment.get_raw_segments().iter().position(|seg| seg == raw_segment);
    let mut indent_seg = None;
    let raw_segments = root_segment.get_raw_segments();
    if let Some(idx) = seg_idx {
        for seg in raw_segments[..=idx].iter().rev() {
            if seg.is_code() {
                indent_seg = None;
            } else if seg.is_type(SyntaxKind::Whitespace) {
                indent_seg = Some(seg);
            } else if seg.is_type(SyntaxKind::Newline) {
                break;
            }
        }
    }

    indent_seg.map(|seg| seg.raw()).unwrap_or_default().to_smolstr()
}
