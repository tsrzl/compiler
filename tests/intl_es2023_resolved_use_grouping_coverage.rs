use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_boolean_assignment_given_es2023_resolved_use_grouping_result_when_checking_types()
{
    // TypeScript 7.0.2 case: conformance/es2023/intlNumberFormatES2023.ts types the result as
    // `false | keyof Intl.NumberFormatOptionsUseGroupingRegistry`, which includes string values.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("intlNumberFormatES2023.ts"),
        "const useGrouping: boolean = new Intl.NumberFormat('en-GB').resolvedOptions().useGrouping;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2022));

    // Act
    let result = compiler.compile(source);

    // Assert
    let diagnostic_codes: Vec<_> = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert!(
        diagnostic_codes.contains(&2322),
        "the ES2023 resolved useGrouping result may be a string; got diagnostics {diagnostic_codes:?}"
    );
}
