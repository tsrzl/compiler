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
