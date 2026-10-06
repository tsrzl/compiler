use tsrzl::spelling::spelling_suggestion;

#[test]
fn should_suggest_closest_candidate_given_misspelled_keyword_when_finding_suggestion() {
    // Arrange
    let candidates = ["function", "interface", "namespace"];

    // Act
    let actual = spelling_suggestion("funtion", candidates);

    // Assert
    assert_eq!(actual, Some("function"));
}

#[test]
fn should_ignore_short_candidates_given_case_sensitive_difference_when_finding_suggestion() {
    // Arrange
    let candidates = ["if", "in"];

    // Act
    let actual = spelling_suggestion("is", candidates);

    // Assert
    assert_eq!(actual, None);
}

#[test]
fn should_break_ties_by_comparison_given_equally_close_candidates_when_suggesting_by_key() {
    // Arrange
    let candidates = [(2, "colour"), (1, "colors")];

    // Act
    let suggestion = tsrzl::spelling::spelling_suggestion_by(
        "color",
        candidates,
        |(_, name)| (*name).to_owned(),
        |left, right| left.0.cmp(&right.0),
    );

    // Assert
    assert_eq!(suggestion, Some((1, "colors")));
}
