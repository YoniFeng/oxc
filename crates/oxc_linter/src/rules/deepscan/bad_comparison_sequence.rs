use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Bad comparison sequence")]
#[diagnostic(
    severity(warning),
    help(
        "Comparison result should not be used directly as an operand of another comparison. If you need to compare three or more operands, you should connect each comparison operation with logical AND operator (`&&`)"
    )
)]
struct BadComparisonSequenceDiagnostic(#[label] pub Span);

/// `https://deepscan.io/docs/rules/bad-comparison-sequence`
#[derive(Debug, Default, Clone)]
pub struct BadComparisonSequence;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when the comparison operator is applied two or more times in a row.
    ///
    /// ### Why is this bad?
    /// Because comparison operator is a binary operator, it is impossible to compare three or more operands at once.
    /// If comparison operator is used to compare three or more operands, only the first two operands are compared and the rest is compared with its result of boolean type.
    ///
    /// ### Example
    /// ```javascript
    /// if (a == b == c) {
    ///  console.log("a, b, and c are the same");
    /// }
    /// ```
    BadComparisonSequence,
    correctness
);

impl Rule for BadComparisonSequence {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BinaryExpression(expr) = node.get().kind()
            && is_bad_comparison(expr)
            && has_no_bad_comparison_in_parents(node, ctx)
        {
            ctx.diagnostic(BadComparisonSequenceDiagnostic(expr.span));
        }
    }
}

fn has_no_bad_comparison_in_parents<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> bool {
    let mut current_node = node;
    loop {
        current_node = ctx.parent_node(current_node).unwrap();
        let kind = current_node.get().kind();

        // `a === b === c === d === e` only produce one error, since `(a === b === c) === d === e` will produce two errors.
        // So we should treat Parenthesized Expression as a boundary.
        if matches!(kind, AstKind::Root | AstKind::ParenthesizedExpression(_))
            || kind.is_declaration()
            || kind.is_statement()
        {
            return true;
        }

        if let AstKind::BinaryExpression(expr) = kind && is_bad_comparison(expr) {
            return false;
        }
    }
}

fn is_bad_comparison(expr: &BinaryExpression) -> bool {
    if expr.operator.is_equality()
        && let Expression::BinaryExpression(left_expr) = &expr.left
        && left_expr.operator.is_equality()
    {
        return true
    }

    if expr.operator.is_compare()
        && let Expression::BinaryExpression(left_expr) = &expr.left
        && left_expr.operator.is_compare()
    {
        return true
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("if (a == b > c) { console.log('foo') }", None),
        ("if (a == b < c) { console.log('foo') }", None),
        ("if (a == b >= c) { console.log('foo') }", None),
        ("if (a == b <= c) { console.log('foo') }", None),
        ("if (a === b > c) { console.log('foo') }", None),
        ("if (a === b < c) { console.log('foo') }", None),
        ("if (a === b >= c) { console.log('foo') }", None),
        ("if (a === b <= c) { console.log('foo') }", None),
        ("if (a != b > c) { console.log('foo') }", None),
        ("if (a != b < c) { console.log('foo') }", None),
        ("if (a != b >= c) { console.log('foo') }", None),
        ("if (a != b <= c) { console.log('foo') }", None),
        ("if (a !== b > c) { console.log('foo') }", None),
        ("if (a !== b < c) { console.log('foo') }", None),
        ("if (a !== b >= c) { console.log('foo') }", None),
        ("if (a !== b <= c) { console.log('foo') }", None),
        ("if (a > b == c) { console.log('foo') }", None),
        ("if (a > b === c) { console.log('foo') }", None),
        ("if (a > b != c) { console.log('foo') }", None),
        ("if (a > b !== c) { console.log('foo') }", None),
        ("if (a < b == c) { console.log('foo') }", None),
        ("if (a < b === c) { console.log('foo') }", None),
        ("if (a < b != c) { console.log('foo') }", None),
        ("if (a < b !== c) { console.log('foo') }", None),
        ("if (a >= b == c) { console.log('foo') }", None),
        ("if (a >= b === c) { console.log('foo') }", None),
        ("if (a >= b != c) { console.log('foo') }", None),
        ("if (a >= b !== c) { console.log('foo') }", None),
        ("if (a <= b == c) { console.log('foo') }", None),
        ("if (a <= b === c) { console.log('foo') }", None),
        ("if (a <= b != c) { console.log('foo') }", None),
        ("if (a <= b !== c) { console.log('foo') }", None),
        ("if ((a == b) && (b == c)) { console.log('foo') }", None),
        ("if ((a > b) && (b > c)) { console.log('foo') }", None),
    ];

    let fail = vec![
        ("if (a == b == c) { console.log('foo') }", None),
        ("if (a == b == c == d) { console.log('foo') }", None),
        ("if ((a == b == c) == d) { console.log('foo') }", None),
        ("if ((a == b == c) == d == e == f) { console.log('foo') }", None),
        ("if (a == b === c) { console.log('foo') }", None),
        ("if (a == b != c) { console.log('foo') }", None),
        ("if (a == b !== c) { console.log('foo') }", None),
        ("if (a === b == c) { console.log('foo') }", None),
        ("if (a === b === c) { console.log('foo') }", None),
        ("if (a === b != c) { console.log('foo') }", None),
        ("if (a === b !== c) { console.log('foo') }", None),
        ("if (a != b == c) { console.log('foo') }", None),
        ("if (a != b === c) { console.log('foo') }", None),
        ("if (a != b != c) { console.log('foo') }", None),
        ("if (a != b !== c) { console.log('foo') }", None),
        ("if (a !== b == c) { console.log('foo') }", None),
        ("if (a !== b === c) { console.log('foo') }", None),
        ("if (a !== b != c) { console.log('foo') }", None),
        ("if (a !== b !== c) { console.log('foo') }", None),
        ("if (a > b > c) { console.log('foo') }", None),
        ("if (a > b < c) { console.log('foo') }", None),
        ("if (a > b >= c) { console.log('foo') }", None),
        ("if (a > b <= c) { console.log('foo') }", None),
        ("if (a < b > c) { console.log('foo') }", None),
        ("if (a < b < c) { console.log('foo') }", None),
        ("if (a < b >= c) { console.log('foo') }", None),
        ("if (a < b <= c) { console.log('foo') }", None),
        ("if (a >= b > c) { console.log('foo') }", None),
        ("if (a >= b < c) { console.log('foo') }", None),
        ("if (a >= b >= c) { console.log('foo') }", None),
        ("if (a >= b <= c) { console.log('foo') }", None),
        ("if (a <= b > c) { console.log('foo') }", None),
        ("if (a <= b < c) { console.log('foo') }", None),
        ("if (a <= b >= c) { console.log('foo') }", None),
        ("if (a <= b <= c) { console.log('foo') }", None),
    ];

    Tester::new(BadComparisonSequence::NAME, pass, fail).test_and_snapshot();
}
