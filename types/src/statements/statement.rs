use crate::{Assignee, ConditionalStatement, Expression, Identifier, Integer, Variable};
use leo_ast::{
    operations::AssignOperation,
    statements::{
        AssertStatement,
        AssignStatement,
        DefinitionStatement,
        ExpressionStatement,
        ForStatement,
        MultipleAssignmentStatement,
        ReturnStatement,
        Statement as AstStatement,
    },
};

use std::fmt;

/// Program statement that defines some action (or expression) to be carried out.
#[derive(Clone, PartialEq, Eq)]
pub enum Statement {
    Return(Vec<Expression>),
    Definition(Variable, Expression),
    Assign(Assignee, Expression),
    MultipleAssign(Vec<Variable>, Expression),
    Conditional(ConditionalStatement),
    For(Identifier, Integer, Integer, Vec<Statement>),
    AssertEq(Expression, Expression),
    Expression(Expression),
}

impl<'ast> From<ReturnStatement<'ast>> for Statement {
    fn from(statement: ReturnStatement<'ast>) -> Self {
        Statement::Return(
            statement
                .expressions
                .into_iter()
                .map(|expression| Expression::from(expression))
                .collect(),
        )
    }
}

impl<'ast> From<DefinitionStatement<'ast>> for Statement {
    fn from(statement: DefinitionStatement<'ast>) -> Self {
        Statement::Definition(
            Variable::from(statement.variable),
            Expression::from(statement.expression),
        )
    }
}

impl<'ast> From<AssignStatement<'ast>> for Statement {
    fn from(statement: AssignStatement<'ast>) -> Self {
        match statement.assign {
            AssignOperation::Assign(ref _assign) => Statement::Assign(
                Assignee::from(statement.assignee),
                Expression::from(statement.expression),
            ),
            operation_assign => {
                // convert assignee into postfix expression
                let converted = Expression::from(statement.assignee.clone());

                match operation_assign {
                    AssignOperation::AddAssign(ref _assign) => Statement::Assign(
                        Assignee::from(statement.assignee),
                        Expression::Add(Box::new(converted), Box::new(Expression::from(statement.expression))),
                    ),
                    AssignOperation::SubAssign(ref _assign) => Statement::Assign(
                        Assignee::from(statement.assignee),
                        Expression::Sub(Box::new(converted), Box::new(Expression::from(statement.expression))),
                    ),
                    AssignOperation::MulAssign(ref _assign) => Statement::Assign(
                        Assignee::from(statement.assignee),
                        Expression::Mul(Box::new(converted), Box::new(Expression::from(statement.expression))),
                    ),
                    AssignOperation::DivAssign(ref _assign) => Statement::Assign(
                        Assignee::from(statement.assignee),
                        Expression::Div(Box::new(converted), Box::new(Expression::from(statement.expression))),
                    ),
                    AssignOperation::PowAssign(ref _assign) => Statement::Assign(
                        Assignee::from(statement.assignee),
                        Expression::Pow(Box::new(converted), Box::new(Expression::from(statement.expression))),
                    ),
                    AssignOperation::Assign(ref _assign) => unimplemented!("cannot assign twice to assign statement"),
                }
            }
        }
    }
}

impl<'ast> From<MultipleAssignmentStatement<'ast>> for Statement {
    fn from(statement: MultipleAssignmentStatement<'ast>) -> Self {
        let variables = statement
            .variables
            .into_iter()
            .map(|typed_variable| Variable::from(typed_variable))
            .collect();

        Statement::MultipleAssign(
            variables,
            Expression::FunctionCall(
                Box::new(Expression::from(statement.function_name)),
                statement.arguments.into_iter().map(|e| Expression::from(e)).collect(),
            ),
        )
    }
}

impl<'ast> From<ForStatement<'ast>> for Statement {
    fn from(statement: ForStatement<'ast>) -> Self {
        let from = match Expression::from(statement.start) {
            Expression::Integer(number) => number,
            Expression::Implicit(string) => Integer::from_implicit(string),
            expression => unimplemented!("Range bounds should be integers, found {}", expression),
        };
        let to = match Expression::from(statement.stop) {
            Expression::Integer(number) => number,
            Expression::Implicit(string) => Integer::from_implicit(string),
            expression => unimplemented!("Range bounds should be integers, found {}", expression),
        };

        Statement::For(
            Identifier::from(statement.index),
            from,
            to,
            statement
                .statements
                .into_iter()
                .map(|statement| Statement::from(statement))
                .collect(),
        )
    }
}

impl<'ast> From<AssertStatement<'ast>> for Statement {
    fn from(statement: AssertStatement<'ast>) -> Self {
        match statement {
            AssertStatement::AssertEq(assert_eq) => {
                Statement::AssertEq(Expression::from(assert_eq.left), Expression::from(assert_eq.right))
            }
        }
    }
}

impl<'ast> From<ExpressionStatement<'ast>> for Statement {
    fn from(statement: ExpressionStatement<'ast>) -> Self {
        Statement::Expression(Expression::from(statement.expression))
    }
}

impl<'ast> From<AstStatement<'ast>> for Statement {
    fn from(statement: AstStatement<'ast>) -> Self {
        match statement {
            AstStatement::Return(statement) => Statement::from(statement),
            AstStatement::Definition(statement) => Statement::from(statement),
            AstStatement::Assign(statement) => Statement::from(statement),
            AstStatement::MultipleAssignment(statement) => Statement::from(statement),
            AstStatement::Conditional(statement) => Statement::Conditional(ConditionalStatement::from(statement)),
            AstStatement::Iteration(statement) => Statement::from(statement),
            AstStatement::Assert(statement) => Statement::from(statement),
            AstStatement::Expression(statement) => Statement::from(statement),
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Statement::Return(ref statements) => {
                write!(f, "return (")?;
                for (i, value) in statements.iter().enumerate() {
                    write!(f, "{}", value)?;
                    if i < statements.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")\n")
            }
            Statement::Definition(ref variable, ref expression) => write!(f, "let {} = {};", variable, expression),
            Statement::Assign(ref variable, ref statement) => write!(f, "{} = {};", variable, statement),
            Statement::MultipleAssign(ref assignees, ref function) => {
                write!(f, "let (")?;
                for (i, id) in assignees.iter().enumerate() {
                    write!(f, "{}", id)?;
                    if i < assignees.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") = {};", function)
            }
            Statement::Conditional(ref statement) => write!(f, "{}", statement),
            Statement::For(ref var, ref start, ref stop, ref list) => {
                write!(f, "for {} in {}..{} {{\n", var, start, stop)?;
                for l in list {
                    write!(f, "\t\t{}\n", l)?;
                }
                write!(f, "\t}}")
            }
            Statement::AssertEq(ref left, ref right) => write!(f, "assert_eq({}, {});", left, right),
            Statement::Expression(ref expression) => write!(f, "{};", expression),
        }
    }
}