use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_month_argument_given_year_only_date_utc_call_when_checking_types() {
    // Oracle: conformance/es5/es5DateAPIs.ts, target ES2015 and lib ES5;
    // es5DateAPIs(target=es2015).errors.txt expects TS2554: "Expected 2-7 arguments, but got 1."
    // Arrange
    let source = SourceFile::from_path(Path::new("es5DateAPIs.ts"), "Date.UTC(2017);")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    let argument_count_diagnostics: Vec<_> = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == 2554)
        .collect();
    assert_eq!(argument_count_diagnostics.len(), 1);
    assert_eq!(
        argument_count_diagnostics[0].message(),
        "Expected 2-7 arguments, but got 1."
    );
}
