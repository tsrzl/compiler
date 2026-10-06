//! Spelling suggestions, matching TypeScript-Go's `core.GetSpellingSuggestion`.

/// Returns the candidate closest to `name` by case-weighted edit distance, if any is close enough.
///
/// Ties keep the lexicographically smallest candidate, so the result does not depend on order.
#[must_use]
pub fn spelling_suggestion<'candidate>(
    name: &str,
    candidates: impl IntoIterator<Item = &'candidate str>,
) -> Option<&'candidate str> {
    spelling_suggestion_by(
        name,
        candidates,
        |candidate| (*candidate).to_owned(),
        Ord::cmp,
    )
}

/// Returns the candidate whose name, given by `name_of`, is closest to `name`, as TypeScript-Go's
/// generic `GetSpellingSuggestion` does. Candidates with an empty name are skipped, and equally
/// close candidates are ordered by `compare`.
pub fn spelling_suggestion_by<T: Copy>(
    name: &str,
    candidates: impl IntoIterator<Item = T>,
    name_of: impl Fn(&T) -> String,
    compare: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Option<T> {
    let name_chars = name.chars().collect::<Vec<_>>();
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let maximum_length_difference = 2.max((name_chars.len() as f64 * 0.34) as usize);
    #[allow(clippy::cast_precision_loss)]
    let mut best_distance = (name_chars.len() as f64 * 0.4).floor() + 0.9;
    let mut best: Option<T> = None;
    for candidate in candidates {
        let candidate_name = name_of(&candidate);
        // TypeScript-Go compares the candidate's byte length with the name's character count.
        let longer = candidate_name.len().max(name_chars.len());
        let shorter = candidate_name.len().min(name_chars.len());
        if candidate_name.is_empty()
            || longer - shorter > maximum_length_difference
            || candidate_name == name
        {
            continue;
        }
        // A user would notice other differences in names under three characters.
        if candidate_name.len() < 3 && !candidate_name.eq_ignore_ascii_case(name) {
            continue;
        }
        let candidate_chars = candidate_name.chars().collect::<Vec<_>>();
        let Some(distance) = levenshtein_with_max(&name_chars, &candidate_chars, best_distance)
        else {
            continue;
        };
        if distance < best_distance {
            best_distance = distance;
            best = Some(candidate);
        } else if best.is_none_or(|best| compare(&candidate, &best).is_lt()) {
            best = Some(candidate);
        }
    }
    best
}

/// Returns the case-weighted edit distance between two strings, or `None` above `max_value`.
fn levenshtein_with_max(first: &[char], second: &[char], max_value: f64) -> Option<f64> {
    let big = max_value + 0.01;
    #[allow(clippy::cast_precision_loss)]
    let mut previous = (0..=second.len())
        .map(|index| index as f64)
        .collect::<Vec<_>>();
    let mut current = vec![0.0; second.len() + 1];
    for (row, &first_char) in first.iter().enumerate() {
        let i = row + 1;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let min_j = ((i as f64 - max_value).ceil().max(1.0)) as usize;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let max_j = ((max_value + i as f64).floor() as usize).min(second.len());
        #[allow(clippy::cast_precision_loss)]
        let mut column_min = i as f64;
        current[0] = column_min;
        for slot in current.iter_mut().take(min_j).skip(1) {
            *slot = big;
        }
        for j in min_j..=max_j {
            let second_char = second[j - 1];
            let substitution = if lowercase(first_char) == lowercase(second_char) {
                previous[j - 1] + 0.1
            } else {
                previous[j - 1] + 2.0
            };
            let distance = if first_char == second_char {
                previous[j - 1]
            } else {
                (previous[j] + 1.0).min((current[j - 1] + 1.0).min(substitution))
            };
            current[j] = distance;
            column_min = column_min.min(distance);
        }
        for slot in current.iter_mut().skip(max_j + 1) {
            *slot = big;
        }
        if column_min > max_value {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let result = previous[second.len()];
    (result <= max_value).then_some(result)
}

fn lowercase(character: char) -> char {
    character.to_lowercase().next().unwrap_or(character)
}
