use crate::compiler::ScriptTarget;
use crate::syntax::FunctionParameter;

use super::{EmitContext, emit_expression};

pub(super) fn emit_parameters(
    parameters: &[FunctionParameter],
    wrap: bool,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
    output: &mut String,
) {
    if wrap {
        output.push('(');
    }
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(parameter.name());
        if let Some(initializer) = parameter.initializer() {
            output.push_str(" = ");
            output.push_str(&emit_expression(initializer, context, target, indentation));
        }
    }
    if wrap {
        output.push(')');
    }
}
