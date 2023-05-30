use rustpython_parser::ast;
use rustpython_parser::ast::{Expr, Ranged};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::prelude::Constant;

use crate::checkers::ast::Checker;

/// ## What it does
/// Checks that the task variable name matches the `task_id` value for
/// Airflow Operators.
///
/// ## Why is this bad?
/// When initializing an Airflow Operator, for consistency, the variable
/// name should match the `task_id` value. This makes it easier to
/// follow the flow of the DAG.
///
/// ## Example
/// ```python
/// from airflow.operators import PythonOperator
///
///
/// incorrect_name = PythonOperator(task_id="my_task")
/// ```
///
/// Use instead:
/// ```python
/// from airflow.operators import PythonOperator
///
///
/// my_task = PythonOperator(task_id="my_task")
/// ```
#[violation]
pub struct AirflowVariableNameTaskIdMismatch {
    task_id: String,
}

impl Violation for AirflowVariableNameTaskIdMismatch {
    #[derive_message_formats]
    fn message(&self) -> String {
        let AirflowVariableNameTaskIdMismatch { task_id } = self;
        format!("Task variable name should match the `task_id`: \"{task_id}\"")
    }
}

/// AIR001
pub(crate) fn variable_name_task_id(
    checker: &mut Checker,
    targets: &[Expr],
    value: &Expr,
) -> Option<Diagnostic> {
    // If we have more than one target, we can't do anything.
    if targets.len() != 1 {
        return None;
    }

    let target = &targets[0];
    let Expr::Name(ast::ExprName { id, .. }) = target else {
        return None;
    };

    // If the value is not a call, we can't do anything.
    let Expr::Call(ast::ExprCall { func, keywords, .. }) = value else {
        return None;
    };

    // If the function doesn't come from Airflow, we can't do anything.
    if !checker
        .semantic_model()
        .resolve_call_path(func)
        .map_or(false, |call_path| matches!(call_path[0], "airflow"))
    {
        return None;
    }

    // If the call doesn't have a `task_id` keyword argument, we can't do anything.
    let keyword = keywords
        .iter()
        .find(|keyword| keyword.arg.as_ref().map_or(false, |arg| arg == "task_id"))?;

    // If the keyword argument is not a string, we can't do anything.
    let task_id = match &keyword.value {
        Expr::Constant(constant) => match &constant.value {
            Constant::Str(value) => value,
            _ => return None,
        },
        _ => return None,
    };

    // If the target name is the same as the task_id, no violation.
    if id == task_id {
        return None;
    }

    Some(Diagnostic::new(
        AirflowVariableNameTaskIdMismatch {
            task_id: task_id.to_string(),
        },
        target.range(),
    ))
}