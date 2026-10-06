use tsrzl::ast::SyntaxKind;
use tsrzl::scanner::{leading_comment_ranges, trailing_comment_ranges};

#[test]
fn should_collect_each_comment_given_leading_comments_when_reading_leading_comment_ranges() {
    // Arrange
    let text = "// one\n/* two */ let value;";

    // Act
    let ranges = leading_comment_ranges(text, 0);

    // Assert
    let spans: Vec<_> = ranges
        .iter()
        .map(|range| &text[range.pos..range.end])
        .collect();
    assert_eq!(spans, ["// one", "/* two */"]);
}

#[test]
fn should_skip_shebang_given_file_start_when_reading_leading_comment_ranges() {
    // Arrange
    let text = "#!/usr/bin/env node\n// note\nrun();";

    // Act
    let ranges = leading_comment_ranges(text, 0);

    // Assert
    assert_eq!(
        ranges.iter().map(|range| range.kind).collect::<Vec<_>>(),
        [SyntaxKind::SingleLineCommentTrivia]
    );
}

#[test]
fn should_record_trailing_new_line_given_line_comment_when_reading_leading_comment_ranges() {
    // Arrange
    let text = "// note\nrun();";

    // Act
    let ranges = leading_comment_ranges(text, 0);

    // Assert
    assert!(ranges[0].has_trailing_new_line);
}

#[test]
fn should_stop_at_line_break_given_trailing_comments_when_reading_trailing_comment_ranges() {
    // Arrange
    let text = "run(); /* after */\n// next line";

    // Act
    let ranges = trailing_comment_ranges(text, 6);

    // Assert
    let spans: Vec<_> = ranges
        .iter()
        .map(|range| &text[range.pos..range.end])
        .collect();
    assert_eq!(spans, ["/* after */"]);
}
