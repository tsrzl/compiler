use std::fmt::Write as _;
use std::path::Path;

use cntryl_stress::{StressContext, black_box, stress, stress_main};
use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[stress(tier = 1, metadata(component = "compiler"))]
fn compile_typed_variables(context: &mut StressContext) {
    let mut source_text = String::new();
    for index in 0..512 {
        writeln!(source_text, "const answer_{index}: number = {index};")
            .expect("writing to a string cannot fail");
    }
    let source = SourceFile::from_path(Path::new("benchmark.ts"), source_text)
        .expect("the benchmark source path has a supported extension");
    let compiler = Compiler::new();

    context
        .benchmark("compile_typed_variables")
        .samples(20)
        .measure_with_setup(
            || source.clone(),
            |source| black_box(compiler.compile(source)),
        );
}

stress_main!();
