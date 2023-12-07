use std::collections::HashSet;

use itertools::{chain, enumerate, Itertools};

use crate::core::parser::{
    context::ParseContext, helpers::trim_non_code_segments, match_algorithms::prune_options,
    match_result::MatchResult, matchable::Matchable, segments::base::Segment, types::ParseMode,
};

fn parse_mode_match_result(
    matched_segments: Vec<Box<dyn Segment>>,
    mut unmatched_segments: Vec<Box<dyn Segment>>,
    tail: Vec<Box<dyn Segment>>,
    parse_mode: ParseMode,
) -> MatchResult {
    if let ParseMode::Strict = parse_mode {
        let mut unmatched = unmatched_segments;
        unmatched.extend(tail);
        return MatchResult::new(matched_segments, unmatched);
    }

    if unmatched_segments.is_empty() || unmatched_segments.iter().all(|s| !s.is_code()) {
        let mut unmatched = unmatched_segments;
        unmatched.extend(tail);
        return MatchResult::new(matched_segments, unmatched);
    }

    let trim_idx = unmatched_segments
        .iter()
        .position(|s| s.is_code())
        .unwrap_or(0);

    // Create an unmatched segment
    let expected = if let Some(first_tail_segment) = tail.get(0) {
        format!("Nothing else before {first_tail_segment:?}")
    } else {
        "Nothing else".to_string()
    };

    let unmatched_seg = unimplemented!();
    // let unmatched_seg = UnparsableSegment::new(&unmatched_segments[trim_idx..], expected);
    let mut matched = matched_segments;
    matched.extend_from_slice(&unmatched_segments[..trim_idx]);
    matched.push(unmatched_seg);

    MatchResult::new(matched, tail)
}

#[derive(Debug, Clone)]
pub struct AnyNumberOf {
    elements: Vec<Box<dyn Matchable>>,
    max_times: Option<usize>,
    min_times: usize,
    allow_gaps: bool,
}

impl PartialEq for AnyNumberOf {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl AnyNumberOf {
    pub fn new(elements: Vec<Box<dyn Matchable>>) -> Self {
        Self {
            elements,
            max_times: None,
            min_times: 1,
            allow_gaps: true,
        }
    }

    fn _longest_trimmed_match(
        &self,
        segments: &[Box<dyn Segment>],
        matchers: Vec<Box<dyn Matchable>>,
        parse_context: &mut ParseContext,
        trim_noncode: bool,
    ) -> (MatchResult, Option<Box<dyn Matchable>>) {
        // Have we been passed an empty list?
        if segments.is_empty() {
            return (MatchResult::from_empty(), None);
        }
        // If presented with no options, return no match
        else if matchers.is_empty() {
            return (MatchResult::from_unmatched(segments), None);
        }

        let available_options = prune_options(&matchers, segments, parse_context);

        if available_options.is_empty() {
            return (MatchResult::from_unmatched(segments), None);
        }

        let mut best_match_length = 0;
        let mut best_match = None;

        for (idx, matcher) in enumerate(available_options) {
            let match_result = matcher.match_segments(segments.to_vec(), parse_context);

            // No match. Skip this one.
            if !match_result.has_match() {
                continue;
            }

            if match_result.is_complete() {
                unimplemented!()
            } else if match_result.has_match() {
                if match_result.trimmed_matched_length() > best_match_length {
                    best_match_length = match_result.trimmed_matched_length();
                    best_match = ((match_result, matcher)).into();
                }
            }
        }

        // If we get here, then there wasn't a complete match. If we
        // has a best_match, return that.
        if best_match_length > 0 {
            let (match_result, matchable) = best_match.unwrap();
            return (match_result, matchable.into());
        }

        unimplemented!()
    }

    // Match the forward segments against the available elements once.
    // This serves as the main body of OneOf, but also a building block
    // for AnyNumberOf.
    pub fn match_once(
        &self,
        segments: &[Box<dyn Segment>],
        parse_context: &mut ParseContext,
    ) -> (MatchResult, Option<Box<dyn Matchable>>) {
        let name = std::any::type_name::<Self>();

        parse_context.deeper_match(name, false, &[], None, |ctx| {
            self._longest_trimmed_match(segments, self.elements.clone(), ctx, false)
        })
    }
}

impl Matchable for AnyNumberOf {
    fn is_optional(&self) -> bool {
        todo!()
    }

    fn simple(
        &self,
        parse_context: &ParseContext,
        crumbs: Option<Vec<&str>>,
    ) -> Option<(HashSet<String>, HashSet<String>)> {
        todo!()
    }

    fn match_segments(
        &self,
        segments: Vec<Box<dyn Segment>>,
        parse_context: &mut ParseContext,
    ) -> MatchResult {
        let mut matched_segments = MatchResult::from_empty();
        let mut unmatched_segments = segments;
        let tail = Vec::new();

        // Keep track of the number of times each option has been matched.
        let mut n_matches = 0;
        // let option_counter = {elem.cache_key(): 0 for elem in self._elements}
        loop {
            if self.max_times.is_some() && Some(n_matches) >= self.max_times {
                // We've matched as many times as we can
                unimplemented!()
            }

            // Is there anything left to match?
            if unmatched_segments.is_empty() {
                unimplemented!()
            }

            let pre_seg = if n_matches > 0 && self.allow_gaps {
                let segments = std::mem::take(&mut unmatched_segments);
                let (pre_seg, mid_seg, post_seg) = trim_non_code_segments(&segments);

                unmatched_segments = chain(mid_seg, post_seg).cloned().collect_vec();

                pre_seg.to_vec()
            } else {
                Vec::new()
            };

            let (match_result, matched_option) =
                self.match_once(&unmatched_segments, parse_context);

            // Increment counter for matched option.
            if let Some(_matched_option) = matched_option {
                // TODO:
                // if matched_option.cache_key() in option_counter:
            }

            if match_result.has_match() {
                matched_segments
                    .matched_segments
                    .extend(chain(pre_seg, match_result.matched_segments));
                unmatched_segments = match_result.unmatched_segments;
                n_matches += 1;
            } else {
                // If we get here, then we've not managed to match. And the next
                // unmatched segments are meaningful, i.e. they're not what we're
                // looking for.
                return if n_matches >= self.min_times {
                    parse_mode_match_result(
                        matched_segments.matched_segments,
                        chain(pre_seg, unmatched_segments).collect_vec(),
                        tail,
                        ParseMode::Strict,
                    )
                } else {
                    // We didn't meet the hurdle
                    parse_mode_match_result(
                        vec![],
                        chain(matched_segments.matched_segments, pre_seg)
                            .chain(unmatched_segments)
                            .collect_vec(),
                        tail,
                        ParseMode::Strict,
                    )
                };
            }
        }
    }

    fn cache_key(&self) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        core::{
            dialects::init::{dialect_selector, get_default_dialect},
            parser::{
                context::ParseContext,
                matchable::Matchable,
                parsers::StringParser,
                segments::{keyword::KeywordSegment, test_functions::generate_test_segments_func},
            },
        },
        helpers::ToMatchable,
        traits::Boxed,
    };

    use super::AnyNumberOf;

    #[test]
    fn test__parser__grammar_anyof_modes() {
        let cases: [(&[&str], &[(&str, &str)]); 3] = [
            (&["a"], &[("a", "kw")]),
            (&["b"], &[]),
            (
                &["b", "a"],
                &[("a", "kw"), (" ", "whitespace"), ("b", "kw")],
            ),
        ];

        let segments = generate_test_segments_func(vec!["a", " ", "b", " ", "c", "d", " ", "d"]);
        let mut parse_cx = ParseContext::new(dialect_selector(get_default_dialect()).unwrap());

        for (sequence, output_tuple) in cases {
            let elements = sequence
                .iter()
                .map(|it| {
                    StringParser::new(
                        it,
                        |it| {
                            KeywordSegment::new(
                                it.get_raw().unwrap(),
                                it.get_position_marker().unwrap(),
                            )
                            .boxed()
                        },
                        None,
                        false,
                        None,
                    )
                    .to_matchable()
                })
                .collect_vec();

            let seq = AnyNumberOf::new(elements);

            let match_result = seq.match_segments(segments.clone(), &mut parse_cx);
            let matched_segments = match_result.matched_segments;

            let result = matched_segments
                .into_iter()
                .map(|segment| (segment.get_raw().unwrap(), segment.get_type()))
                .collect_vec();

            let are_equal = result
                .iter()
                .map(|(s, str_ref)| (s.as_str(), str_ref))
                .zip(output_tuple.iter())
                .all(|((s1, str_ref1), (s2, str_ref2))| s1 == *s2 && str_ref1 == str_ref2);

            assert!(are_equal);
        }
    }
}
