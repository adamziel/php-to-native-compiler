use std::collections::{HashMap, HashSet};

use crate::ast::{
    ArrayDimTarget, ArrayElement, AssignmentOp, BinaryOp, CastKind, CatchClause, ConstDeclaration,
    Expr, FunctionDecl, FunctionParameter, IncDecOp, MagicConstantKind, Program, Statement,
    StringPart, SwitchCase, TypeHint, UnaryOp, UnsetTarget,
};
use crate::diagnostic::{Diagnostic, Result, SourceSpan};
use crate::lexer::{lex, StringPart as TokenStringPart, Token, TokenKind};

const KEYWORD_OR_PRECEDENCE: u8 = 1;
const KEYWORD_XOR_PRECEDENCE: u8 = 2;
const KEYWORD_AND_PRECEDENCE: u8 = 3;
const SYMBOL_OR_PRECEDENCE: u8 = 4;
const SYMBOL_AND_PRECEDENCE: u8 = 5;
const BITWISE_OR_PRECEDENCE: u8 = 6;
const BITWISE_XOR_PRECEDENCE: u8 = 7;
const BITWISE_AND_PRECEDENCE: u8 = 8;
const EQUALITY_PRECEDENCE: u8 = 9;
const COMPARISON_PRECEDENCE: u8 = 10;
const CONCAT_PRECEDENCE: u8 = 13;
const SHIFT_PRECEDENCE: u8 = 18;
const ADDITIVE_PRECEDENCE: u8 = 23;
const MULTIPLICATIVE_PRECEDENCE: u8 = 33;
const POWER_PRECEDENCE: u8 = 40;

pub fn parse(source: &str) -> Result<Program> {
    let tokens = lex(source)?;
    Parser {
        tokens,
        index: 0,
        block_depth: 0,
        function_depth: 0,
    }
    .parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    block_depth: usize,
    function_depth: usize,
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program> {
        self.expect_open_tag()?;
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            if matches!(self.peek().kind, TokenKind::OpenTag | TokenKind::CloseTag) {
                self.advance();
                continue;
            }
            if matches!(self.peek().kind, TokenKind::Function) {
                functions.push(self.parse_function_decl()?);
            } else {
                statements.push(self.parse_statement()?);
            }
        }
        validate_function_names(&functions)?;
        for function in &functions {
            validate_goto_labels(&function.body)?;
        }
        validate_goto_labels(&statements)?;
        Ok(Program {
            functions,
            statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.peek().kind {
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::Do => self.parse_do_while(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Foreach => self.parse_foreach(),
            TokenKind::Switch => self.parse_switch(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Goto => self.parse_goto(),
            TokenKind::Const => self.parse_const(),
            TokenKind::LeftBrace => self.parse_compound_block(),
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_increment_statement(),
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::Colon) => {
                self.parse_label()
            }
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("unset")
                    && matches!(self.peek_next().kind, TokenKind::LeftParen) =>
            {
                self.parse_unset_statement()
            }
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::LeftParen) => {
                self.parse_call_statement()
            }
            TokenKind::Variable(_) => self.parse_variable_statement(),
            TokenKind::InlineHtml(_) => self.parse_inline_html(),
            _ if self.peek_starts_expression() => self.parse_expression_statement(),
            _ => Err(Diagnostic::new(
                "expected statement",
                Some(self.peek().span),
            )),
        }
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl> {
        let span = self.expect_function()?;
        let name_token = self.advance().clone();
        let TokenKind::Identifier(name) = name_token.kind else {
            return Err(Diagnostic::new(
                "expected function name",
                Some(name_token.span),
            ));
        };
        let parameters = self.parse_function_parameters()?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_hint()?)
        } else {
            None
        };
        self.function_depth += 1;
        let body = self.parse_block();
        self.function_depth -= 1;
        let body = body?;
        Ok(FunctionDecl {
            name,
            parameters,
            return_type,
            body,
            span,
        })
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FunctionParameter>> {
        self.expect_left_paren()?;
        let mut parameters = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            parameters.push(self.parse_function_parameter()?);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                parameters.push(self.parse_function_parameter()?);
            }
        }
        self.expect_right_paren()?;
        Ok(parameters)
    }

    fn parse_function_parameter(&mut self) -> Result<FunctionParameter> {
        let type_hint = if matches!(self.peek().kind, TokenKind::Null) {
            Some(self.parse_type_hint()?)
        } else {
            None
        };
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected function parameter variable",
                Some(token.span),
            ));
        };
        Ok(FunctionParameter {
            name,
            type_hint,
            span: token.span,
        })
    }

    fn parse_type_hint(&mut self) -> Result<TypeHint> {
        let token = self.advance();
        match token.kind {
            TokenKind::Null => Ok(TypeHint::Null),
            _ => Err(Diagnostic::new("expected type hint", Some(token.span))),
        }
    }

    fn parse_variable_statement(&mut self) -> Result<Statement> {
        let start = self.index;
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected variable", Some(token.span)));
        };
        match self.peek().kind {
            TokenKind::PlusPlus => {
                self.advance();
                self.expect_statement_terminator()?;
                return Ok(Statement::Increment {
                    name,
                    op: IncDecOp::Increment,
                    span: token.span,
                });
            }
            TokenKind::MinusMinus => {
                self.advance();
                self.expect_statement_terminator()?;
                return Ok(Statement::Increment {
                    name,
                    op: IncDecOp::Decrement,
                    span: token.span,
                });
            }
            _ => {}
        }
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            let target = self.parse_array_dim_target(name, token.span)?;
            if !self.peek_is_assignment_op() {
                self.index = start;
                return self.parse_expression_statement();
            }
            let op = self.expect_assignment_op()?;
            let value = self.parse_assignment_value_expr()?;
            self.expect_statement_terminator()?;
            return Ok(Statement::ArrayAssign {
                target,
                op,
                value,
                span: token.span,
            });
        }
        if !self.peek_is_assignment_op() {
            self.index = start;
            return self.parse_expression_statement();
        }
        let op = self.expect_assignment_op()?;
        let value = self.parse_assignment_value_expr()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Assign {
            name,
            op,
            value,
            span: token.span,
        })
    }

    fn parse_array_dim_target(
        &mut self,
        array: String,
        variable_span: SourceSpan,
    ) -> Result<ArrayDimTarget> {
        self.expect_left_bracket()?;
        let index = if matches!(self.peek().kind, TokenKind::RightBracket) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        let right_span = self.expect_right_bracket()?;
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            return Err(Diagnostic::new(
                "nested array-dimension assignment is unsupported",
                Some(self.peek().span),
            ));
        }
        Ok(ArrayDimTarget {
            array,
            index,
            span: combine_spans(variable_span, right_span),
        })
    }

    fn parse_prefix_increment_statement(&mut self) -> Result<Statement> {
        let op_token = self.advance().clone();
        let op = match op_token.kind {
            TokenKind::PlusPlus => IncDecOp::Increment,
            TokenKind::MinusMinus => IncDecOp::Decrement,
            _ => return Err(Diagnostic::new("expected increment", Some(op_token.span))),
        };
        let variable = self.advance().clone();
        let TokenKind::Variable(name) = variable.kind else {
            return Err(Diagnostic::new("expected variable", Some(variable.span)));
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Increment {
            name,
            op,
            span: op_token.span,
        })
    }

    fn parse_echo(&mut self) -> Result<Statement> {
        let span = self.expect_echo()?;
        let mut expressions = vec![self.parse_expr()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            expressions.push(self.parse_expr()?);
        }
        self.expect_statement_terminator()?;
        Ok(Statement::Echo { expressions, span })
    }

    fn parse_print(&mut self) -> Result<Statement> {
        let span = self.expect_print()?;
        let expression = self.parse_expr()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Print { expression, span })
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expression = self.parse_expr()?;
        let span = expression.span();
        if matches!(self.peek().kind, TokenKind::Question) {
            return self.reject_unsupported_ternary_expression();
        }
        if self.peek_is_assignment_op() {
            return Err(Diagnostic::new(
                "expected assignment",
                Some(self.peek().span),
            ));
        }
        self.expect_statement_terminator()?;
        Ok(Statement::Expression { expression, span })
    }

    fn parse_const(&mut self) -> Result<Statement> {
        let span = self.expect_const()?;
        if self.block_depth != 0 {
            return Err(Diagnostic::new(
                "constant declarations must be at global scope",
                Some(span),
            ));
        }

        let mut declarations = vec![self.parse_const_declaration()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            declarations.push(self.parse_const_declaration()?);
        }
        self.expect_const_statement_terminator()?;
        Ok(Statement::Const { declarations, span })
    }

    fn parse_const_declaration(&mut self) -> Result<ConstDeclaration> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected constant name", Some(token.span)));
        };
        self.expect_equal()?;
        let value = self.parse_expr()?;
        if !is_supported_global_const_expr(&value) {
            return Err(Diagnostic::new(
                "constant expression contains invalid operation",
                Some(value.span()),
            ));
        }
        Ok(ConstDeclaration {
            name,
            value,
            span: token.span,
        })
    }

    fn parse_if(&mut self) -> Result<Statement> {
        let span = self.expect_if_like()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        let then_body = self.parse_statement_body()?;
        let else_body = match self.peek().kind {
            TokenKind::Elseif => vec![self.parse_if()?],
            TokenKind::Else => {
                self.advance();
                self.parse_statement_body()?
            }
            _ => Vec::new(),
        };
        Ok(Statement::If {
            condition,
            then_body,
            else_body,
            span,
        })
    }

    fn parse_while(&mut self) -> Result<Statement> {
        let span = self.expect_while()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        let body = self.parse_statement_body()?;
        Ok(Statement::While {
            condition,
            body,
            span,
        })
    }

    fn parse_do_while(&mut self) -> Result<Statement> {
        let span = self.expect_do()?;
        let body = self.parse_statement_body()?;
        self.expect_while()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        self.expect_statement_terminator()?;
        Ok(Statement::DoWhile {
            body,
            condition,
            span,
        })
    }

    fn parse_for(&mut self) -> Result<Statement> {
        let span = self.expect_for()?;
        self.expect_left_paren()?;

        let initializers = if matches!(self.peek().kind, TokenKind::Semicolon) {
            Vec::new()
        } else {
            self.parse_for_clause_list()?
        };
        self.expect_semicolon()?;

        let condition = if matches!(self.peek().kind, TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect_semicolon()?;

        let updates = if matches!(self.peek().kind, TokenKind::RightParen) {
            Vec::new()
        } else {
            self.parse_for_clause_list()?
        };
        self.expect_right_paren()?;
        let body = self.parse_statement_body()?;

        Ok(Statement::For {
            initializers,
            condition,
            updates,
            body,
            span,
        })
    }

    fn parse_foreach(&mut self) -> Result<Statement> {
        let span = self.expect_foreach()?;
        self.expect_left_paren()?;
        let iterable = self.parse_expr()?;
        self.expect_as()?;
        let first = self.parse_foreach_variable()?;
        let (key, value) = if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            (Some(first), self.parse_foreach_variable()?)
        } else {
            (None, first)
        };
        self.expect_right_paren()?;
        let body = self.parse_statement_body()?;
        Ok(Statement::Foreach {
            iterable,
            key,
            value,
            body,
            span,
        })
    }

    fn parse_foreach_variable(&mut self) -> Result<String> {
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            return Err(Diagnostic::new(
                "by-reference foreach is unsupported",
                Some(self.peek().span),
            ));
        }
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            return Err(Diagnostic::new(
                "foreach destructuring is unsupported",
                Some(self.peek().span),
            ));
        }
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Variable(name) => Ok(name),
            _ => Err(Diagnostic::new(
                "expected foreach variable",
                Some(token.span),
            )),
        }
    }

    fn parse_for_clause_list(&mut self) -> Result<Vec<Statement>> {
        let mut clauses = vec![self.parse_for_clause()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            clauses.push(self.parse_for_clause()?);
        }
        Ok(clauses)
    }

    fn parse_for_clause(&mut self) -> Result<Statement> {
        match self.peek().kind {
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_increment_clause(),
            TokenKind::Identifier(_) => self.parse_call_clause(),
            TokenKind::Variable(_) => self.parse_variable_clause(),
            _ => Err(Diagnostic::new(
                "expected for clause",
                Some(self.peek().span),
            )),
        }
    }

    fn parse_variable_clause(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected variable", Some(token.span)));
        };
        match self.peek().kind {
            TokenKind::PlusPlus => {
                self.advance();
                Ok(Statement::Increment {
                    name,
                    op: IncDecOp::Increment,
                    span: token.span,
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                Ok(Statement::Increment {
                    name,
                    op: IncDecOp::Decrement,
                    span: token.span,
                })
            }
            _ => {
                let op = self.expect_assignment_op()?;
                let value = self.parse_assignment_value_expr()?;
                Ok(Statement::Assign {
                    name,
                    op,
                    value,
                    span: token.span,
                })
            }
        }
    }

    fn parse_prefix_increment_clause(&mut self) -> Result<Statement> {
        let op_token = self.advance().clone();
        let op = match op_token.kind {
            TokenKind::PlusPlus => IncDecOp::Increment,
            TokenKind::MinusMinus => IncDecOp::Decrement,
            _ => return Err(Diagnostic::new("expected increment", Some(op_token.span))),
        };
        let variable = self.advance().clone();
        let TokenKind::Variable(name) = variable.kind else {
            return Err(Diagnostic::new("expected variable", Some(variable.span)));
        };
        Ok(Statement::Increment {
            name,
            op,
            span: op_token.span,
        })
    }

    fn parse_call_clause(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected function name", Some(token.span)));
        };
        let (arguments, _) = self.parse_call_arguments()?;
        validate_array_cursor_mutation_call(&name, &arguments, token.span)?;
        Ok(Statement::Call {
            name: name.to_ascii_lowercase(),
            arguments,
            span: token.span,
        })
    }

    fn parse_switch(&mut self) -> Result<Statement> {
        let span = self.expect_switch()?;
        self.expect_left_paren()?;
        let expression = self.parse_expr()?;
        self.expect_right_paren()?;
        self.expect_left_brace()?;

        let mut cases = Vec::new();
        let mut seen_default = false;
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            let case_span = self.peek().span;
            let condition = match self.peek().kind {
                TokenKind::Case => {
                    self.advance();
                    let condition = self.parse_expr()?;
                    self.expect_colon()?;
                    Some(condition)
                }
                TokenKind::Default => {
                    self.advance();
                    if seen_default {
                        return Err(Diagnostic::new(
                            "Switch statements may only contain one default clause",
                            Some(case_span),
                        ));
                    }
                    seen_default = true;
                    self.expect_colon()?;
                    None
                }
                _ => {
                    return Err(Diagnostic::new(
                        "expected switch case or default",
                        Some(self.peek().span),
                    ))
                }
            };

            let mut body = Vec::new();
            while !matches!(
                self.peek().kind,
                TokenKind::Case | TokenKind::Default | TokenKind::RightBrace | TokenKind::Eof
            ) {
                body.push(self.parse_nested_statement()?);
            }
            cases.push(SwitchCase {
                condition,
                body,
                span: case_span,
            });
        }

        self.expect_right_brace()?;
        Ok(Statement::Switch {
            expression,
            cases,
            span,
        })
    }

    fn parse_break(&mut self) -> Result<Statement> {
        let span = self.expect_break()?;
        let level = match self.peek().kind {
            TokenKind::Int(value) if value >= 0 => {
                self.advance();
                value as usize
            }
            _ => 1,
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Break { level, span })
    }

    fn parse_continue(&mut self) -> Result<Statement> {
        let span = self.expect_continue()?;
        let level = match self.peek().kind {
            TokenKind::Int(value) if value >= 0 => {
                self.advance();
                value as usize
            }
            _ => 1,
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Continue { level, span })
    }

    fn parse_return(&mut self) -> Result<Statement> {
        let span = self.expect_return()?;
        let value = if matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof
        ) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Return { value, span })
    }

    fn parse_try(&mut self) -> Result<Statement> {
        let span = self.expect_try()?;
        let body = self.parse_block()?;
        let mut catches = Vec::new();
        while matches!(self.peek().kind, TokenKind::Catch) {
            catches.push(self.parse_catch_clause()?);
        }
        if catches.is_empty() {
            return Err(Diagnostic::new(
                "try without catch or finally is unsupported",
                Some(span),
            ));
        }
        Ok(Statement::Try {
            body,
            catches,
            span,
        })
    }

    fn parse_catch_clause(&mut self) -> Result<CatchClause> {
        let span = self.expect_catch()?;
        self.expect_left_paren()?;
        let type_name = self.parse_catch_type_name()?;
        let variable = if matches!(self.peek().kind, TokenKind::Variable(_)) {
            let token = self.advance().clone();
            let TokenKind::Variable(name) = token.kind else {
                unreachable!("variable match checked above")
            };
            Some(name)
        } else {
            None
        };
        self.expect_right_paren()?;
        let body = self.parse_block()?;
        Ok(CatchClause {
            type_name,
            variable,
            body,
            span,
        })
    }

    fn parse_catch_type_name(&mut self) -> Result<String> {
        let leading_backslash = matches!(self.peek().kind, TokenKind::Backslash);
        if leading_backslash {
            self.advance();
        }
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected catch type name",
                Some(token.span),
            ));
        };
        Ok(name)
    }

    fn parse_goto(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let span = token.span;
        let label = self.advance().clone();
        let TokenKind::Identifier(label) = label.kind else {
            return Err(Diagnostic::new("expected goto label", Some(label.span)));
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Goto { label, span })
    }

    fn parse_label(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected label", Some(token.span)));
        };
        self.expect_colon()?;
        Ok(Statement::Label {
            name,
            span: token.span,
        })
    }

    fn parse_call_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected function name", Some(token.span)));
        };
        let (arguments, _) = self.parse_call_arguments()?;
        validate_array_cursor_mutation_call(&name, &arguments, token.span)?;
        self.expect_statement_terminator()?;
        Ok(Statement::Call {
            name: name.to_ascii_lowercase(),
            arguments,
            span: token.span,
        })
    }

    fn parse_unset_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected unset", Some(token.span)));
        };
        if !name.eq_ignore_ascii_case("unset") {
            return Err(Diagnostic::new("expected unset", Some(token.span)));
        }
        self.expect_left_paren()?;
        let mut targets = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            targets.push(self.parse_unset_target()?);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                targets.push(self.parse_unset_target()?);
            }
        }
        if targets.is_empty() {
            return Err(Diagnostic::new(
                "unset() expects at least one argument",
                Some(token.span),
            ));
        }
        let right_span = self.expect_right_paren()?;
        self.expect_statement_terminator()?;
        Ok(Statement::Unset {
            targets,
            span: combine_spans(token.span, right_span),
        })
    }

    fn parse_unset_target(&mut self) -> Result<UnsetTarget> {
        let target = self.parse_expr()?;
        match target {
            Expr::Variable(name, span) => Ok(UnsetTarget::Variable { name, span }),
            Expr::ArrayAccess { array, index, span } => match *array {
                Expr::Variable(name, variable_span) => Ok(UnsetTarget::ArrayDim(ArrayDimTarget {
                    array: name,
                    index: Some(*index),
                    span: combine_spans(variable_span, span),
                })),
                _ => Err(Diagnostic::new(
                    "unsupported unset target",
                    Some(array.span()),
                )),
            },
            _ => Err(Diagnostic::new(
                "unsupported unset target",
                Some(target.span()),
            )),
        }
    }

    fn parse_inline_html(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::InlineHtml(content) = token.kind else {
            return Err(Diagnostic::new("expected inline HTML", Some(token.span)));
        };
        Ok(Statement::InlineHtml {
            content,
            span: token.span,
        })
    }

    fn parse_compound_block(&mut self) -> Result<Statement> {
        let span = self.peek().span;
        let statements = self.parse_block()?;
        Ok(Statement::Block { statements, span })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        self.expect_left_brace()?;
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            statements.push(self.parse_nested_statement()?);
        }
        self.expect_right_brace()?;
        Ok(statements)
    }

    fn parse_statement_body(&mut self) -> Result<Vec<Statement>> {
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            self.parse_block()
        } else {
            Ok(vec![self.parse_nested_statement()?])
        }
    }

    fn parse_nested_statement(&mut self) -> Result<Statement> {
        self.block_depth += 1;
        let statement = self.parse_statement();
        self.block_depth -= 1;
        statement
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_binary_expr(0)
    }

    fn parse_assignment_value_expr(&mut self) -> Result<Expr> {
        let value = self.parse_binary_expr(SYMBOL_OR_PRECEDENCE)?;
        if self.peek_is_keyword_boolean_operator() {
            return Err(Diagnostic::new(
                "assignment expressions with keyword boolean operators are unsupported",
                Some(self.peek().span),
            ));
        }
        Ok(value)
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr> {
        let mut left = self.parse_unary_expr()?;
        while let Some((op, precedence, right_associative)) = self.peek_binary_op() {
            if precedence < min_precedence {
                break;
            }

            self.advance();
            let next_min_precedence = if right_associative {
                precedence
            } else {
                precedence + 1
            };
            let right = self.parse_binary_expr(next_min_precedence)?;
            let span = combine_spans(left.span(), right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr> {
        match self.peek().kind {
            TokenKind::Plus => {
                let token = self.advance().clone();
                let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Positive,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Minus => {
                let token = self.advance().clone();
                let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Bang => {
                let token = self.advance().clone();
                let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Tilde => {
                let token = self.advance().clone();
                let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::BitwiseNot,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::LeftParen => {
                if let Some((kind, span)) = self.try_parse_cast_prefix()? {
                    let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                    let span = combine_spans(span, expr.span());
                    Ok(Expr::Cast {
                        kind,
                        expr: Box::new(expr),
                        span,
                    })
                } else {
                    self.parse_postfix_expr()
                }
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary_expr()?;
        loop {
            match self.peek().kind {
                TokenKind::LeftBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let right_span = self.expect_right_bracket()?;
                    expr = Expr::ArrayAccess {
                        span: combine_spans(expr.span(), right_span),
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                TokenKind::ObjectOperator => {
                    let start_span = expr.span();
                    self.advance();
                    let method = self.advance().clone();
                    let TokenKind::Identifier(name) = method.kind else {
                        return Err(Diagnostic::new("expected method name", Some(method.span)));
                    };
                    if !matches!(self.peek().kind, TokenKind::LeftParen) {
                        return Err(Diagnostic::new(
                            "object property reads are unsupported",
                            Some(method.span),
                        ));
                    }
                    let (arguments, right_span) = self.parse_call_arguments()?;
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        name: name.to_ascii_lowercase(),
                        arguments,
                        span: combine_spans(start_span, right_span),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::String(value) => Ok(Expr::String(value, token.span)),
            TokenKind::InterpolatedString(parts) => Ok(Expr::InterpolatedString(
                parts.into_iter().map(lower_string_part).collect(),
                token.span,
            )),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::Identifier(name) => {
                let lowercase = name.to_ascii_lowercase();
                if matches!(self.peek().kind, TokenKind::LeftParen) {
                    match lowercase.as_str() {
                        "array" => self.parse_long_array_literal(token.span),
                        "isset" => self.parse_isset_expr(token.span),
                        "empty" => self.parse_empty_expr(token.span),
                        _ => {
                            let (arguments, right_span) = self.parse_call_arguments()?;
                            validate_array_cursor_mutation_call(
                                &lowercase, &arguments, token.span,
                            )?;
                            Ok(Expr::Call {
                                name: lowercase,
                                arguments,
                                span: combine_spans(token.span, right_span),
                            })
                        }
                    }
                } else if let Some(kind) = magic_constant_kind(&name) {
                    Ok(Expr::MagicConstant(kind, token.span))
                } else {
                    Ok(Expr::Constant(name, token.span))
                }
            }
            TokenKind::Backslash => {
                let name_token = self.advance().clone();
                let TokenKind::Identifier(name) = name_token.kind else {
                    return Err(Diagnostic::new(
                        "expected fully qualified constant name",
                        Some(name_token.span),
                    ));
                };
                Ok(Expr::Constant(
                    name,
                    combine_spans(token.span, name_token.span),
                ))
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                let right_span = self.expect_right_paren()?;
                Ok(Expr::Grouped {
                    expr: Box::new(expr),
                    span: combine_spans(token.span, right_span),
                })
            }
            TokenKind::LeftBracket => self.parse_array_literal(token.span),
            _ => Err(Diagnostic::new("expected expression", Some(token.span))),
        }
    }

    fn parse_array_literal(&mut self, left_span: SourceSpan) -> Result<Expr> {
        let (elements, right_span) = self.parse_array_elements(TokenKind::RightBracket)?;
        Ok(Expr::Array {
            elements,
            span: combine_spans(left_span, right_span),
        })
    }

    fn parse_long_array_literal(&mut self, start_span: SourceSpan) -> Result<Expr> {
        self.expect_left_paren()?;
        let (elements, right_span) = self.parse_array_elements(TokenKind::RightParen)?;
        Ok(Expr::Array {
            elements,
            span: combine_spans(start_span, right_span),
        })
    }

    fn parse_array_elements(
        &mut self,
        terminator: TokenKind,
    ) -> Result<(Vec<ArrayElement>, SourceSpan)> {
        let mut elements = Vec::new();
        while !self.at_array_terminator(&terminator) {
            elements.push(self.parse_array_element()?);
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
            if self.at_array_terminator(&terminator) {
                break;
            }
        }
        let right_span = match terminator {
            TokenKind::RightBracket => self.expect_right_bracket()?,
            TokenKind::RightParen => self.expect_right_paren()?,
            _ => unreachable!("array literal terminators are brackets or parentheses"),
        };
        Ok((elements, right_span))
    }

    fn at_array_terminator(&self, terminator: &TokenKind) -> bool {
        matches!(
            (&self.peek().kind, terminator),
            (TokenKind::RightBracket, TokenKind::RightBracket)
                | (TokenKind::RightParen, TokenKind::RightParen)
        )
    }

    fn parse_array_element(&mut self) -> Result<ArrayElement> {
        let first = self.parse_expr()?;
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let value = self.parse_expr()?;
            Ok(ArrayElement {
                key: Some(first),
                value,
            })
        } else {
            Ok(ArrayElement {
                key: None,
                value: first,
            })
        }
    }

    fn parse_isset_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let (targets, right_span) = self.parse_call_arguments()?;
        if targets.is_empty() {
            return Err(Diagnostic::new(
                "isset() expects at least one argument",
                Some(start_span),
            ));
        }
        Ok(Expr::Isset {
            targets,
            span: combine_spans(start_span, right_span),
        })
    }

    fn parse_empty_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let (mut arguments, right_span) = self.parse_call_arguments()?;
        if arguments.len() != 1 {
            return Err(Diagnostic::new(
                "empty() expects exactly one argument",
                Some(start_span),
            ));
        }
        Ok(Expr::Empty {
            target: Box::new(arguments.remove(0)),
            span: combine_spans(start_span, right_span),
        })
    }

    fn parse_call_arguments(&mut self) -> Result<(Vec<Expr>, SourceSpan)> {
        self.expect_left_paren()?;
        let mut arguments = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            arguments.push(self.parse_expr()?);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                arguments.push(self.parse_expr()?);
            }
        }
        let right_span = self.expect_right_paren()?;
        Ok((arguments, right_span))
    }

    fn try_parse_cast_prefix(&mut self) -> Result<Option<(CastKind, SourceSpan)>> {
        let start = self.index;
        let left = self.advance().clone();
        if self.peek_is_real_cast_name()
            && matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::RightParen)
            )
        {
            return Err(Diagnostic::parse_error(
                "The (real) cast has been removed, use (float) instead",
                Some(left.span),
            ));
        }
        if self.peek_is_unset_cast_name()
            && matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::RightParen)
            )
        {
            return Err(Diagnostic::new(
                "The (unset) cast is no longer supported",
                Some(left.span),
            ));
        }
        if self.peek_is_void_cast_name()
            && matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::RightParen)
            )
        {
            return Err(Diagnostic::parse_error(
                "syntax error, unexpected token \"(void)\"",
                Some(left.span),
            ));
        }
        let Some(kind) = self.peek_cast_kind() else {
            self.index = start;
            return Ok(None);
        };
        self.advance();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            self.index = start;
            return Ok(None);
        }
        let right = self.advance().clone();
        Ok(Some((kind, combine_spans(left.span, right.span))))
    }

    fn peek_is_real_cast_name(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("real")
        )
    }

    fn peek_is_unset_cast_name(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("unset")
        )
    }

    fn peek_is_void_cast_name(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("void")
        )
    }

    fn peek_cast_kind(&self) -> Option<CastKind> {
        match self.peek().kind {
            TokenKind::IntType => Some(CastKind::Int),
            TokenKind::IntegerType => Some(CastKind::Integer),
            TokenKind::FloatType => Some(CastKind::Float),
            TokenKind::DoubleType => Some(CastKind::Double),
            TokenKind::StringType => Some(CastKind::String),
            TokenKind::BinaryType => Some(CastKind::Binary),
            TokenKind::BoolType => Some(CastKind::Bool),
            TokenKind::BooleanType => Some(CastKind::Boolean),
            _ => None,
        }
    }

    fn peek_binary_op(&self) -> Option<(BinaryOp, u8, bool)> {
        match self.peek().kind {
            TokenKind::KeywordOr => Some((BinaryOp::Or, KEYWORD_OR_PRECEDENCE, false)),
            TokenKind::KeywordXor => Some((BinaryOp::Xor, KEYWORD_XOR_PRECEDENCE, false)),
            TokenKind::KeywordAnd => Some((BinaryOp::And, KEYWORD_AND_PRECEDENCE, false)),
            TokenKind::OrOr => Some((BinaryOp::Or, SYMBOL_OR_PRECEDENCE, false)),
            TokenKind::AndAnd => Some((BinaryOp::And, SYMBOL_AND_PRECEDENCE, false)),
            TokenKind::Pipe => Some((BinaryOp::BitwiseOr, BITWISE_OR_PRECEDENCE, false)),
            TokenKind::Caret => Some((BinaryOp::BitwiseXor, BITWISE_XOR_PRECEDENCE, false)),
            TokenKind::Ampersand => Some((BinaryOp::BitwiseAnd, BITWISE_AND_PRECEDENCE, false)),
            TokenKind::EqualEqualEqual => Some((BinaryOp::Identical, EQUALITY_PRECEDENCE, false)),
            TokenKind::NotEqualEqual => Some((BinaryOp::NotIdentical, EQUALITY_PRECEDENCE, false)),
            TokenKind::EqualEqual => Some((BinaryOp::Equal, EQUALITY_PRECEDENCE, false)),
            TokenKind::NotEqual => Some((BinaryOp::NotEqual, EQUALITY_PRECEDENCE, false)),
            TokenKind::Spaceship => Some((BinaryOp::Spaceship, EQUALITY_PRECEDENCE, false)),
            TokenKind::Less => Some((BinaryOp::Less, COMPARISON_PRECEDENCE, false)),
            TokenKind::LessEqual => Some((BinaryOp::LessEqual, COMPARISON_PRECEDENCE, false)),
            TokenKind::Greater => Some((BinaryOp::Greater, COMPARISON_PRECEDENCE, false)),
            TokenKind::GreaterEqual => Some((BinaryOp::GreaterEqual, COMPARISON_PRECEDENCE, false)),
            TokenKind::Dot => Some((BinaryOp::Concat, CONCAT_PRECEDENCE, false)),
            TokenKind::ShiftLeft => Some((BinaryOp::ShiftLeft, SHIFT_PRECEDENCE, false)),
            TokenKind::ShiftRight => Some((BinaryOp::ShiftRight, SHIFT_PRECEDENCE, false)),
            TokenKind::Plus => Some((BinaryOp::Add, ADDITIVE_PRECEDENCE, false)),
            TokenKind::Minus => Some((BinaryOp::Subtract, ADDITIVE_PRECEDENCE, false)),
            TokenKind::Asterisk => Some((BinaryOp::Multiply, MULTIPLICATIVE_PRECEDENCE, false)),
            TokenKind::Slash => Some((BinaryOp::Divide, MULTIPLICATIVE_PRECEDENCE, false)),
            TokenKind::Percent => Some((BinaryOp::Modulo, MULTIPLICATIVE_PRECEDENCE, false)),
            TokenKind::AsteriskAsterisk => Some((BinaryOp::Power, POWER_PRECEDENCE, true)),
            _ => None,
        }
    }

    fn peek_is_keyword_boolean_operator(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::KeywordAnd | TokenKind::KeywordOr | TokenKind::KeywordXor
        )
    }

    fn reject_unsupported_ternary_expression(&mut self) -> Result<Statement> {
        let question = self.advance().clone();
        let first_is_short = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            true
        } else {
            self.parse_expr()?;
            self.expect_colon()?;
            false
        };

        self.parse_expr()?;
        if matches!(self.peek().kind, TokenKind::Question) {
            let message = match (first_is_short, self.peek_next_is_colon()) {
                (true, _) => "Unparenthesized `a ?: b ? c : d` is not supported. Use either `(a ?: b) ? c : d` or `a ?: (b ? c : d)`",
                (false, true) => "Unparenthesized `a ? b : c ?: d` is not supported. Use either `(a ? b : c) ?: d` or `a ? b : (c ?: d)`",
                (false, false) => "Unparenthesized `a ? b : c ? d : e` is not supported. Use either `(a ? b : c) ? d : e` or `a ? b : (c ? d : e)`",
            };
            return Err(Diagnostic::new(message, Some(question.span)));
        }

        Err(Diagnostic::new(
            "ternary expressions are unsupported",
            Some(question.span),
        ))
    }

    fn peek_starts_expression(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::String(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Variable(_)
                | TokenKind::Identifier(_)
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Tilde
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Backslash
        )
    }

    fn peek_next_is_colon(&self) -> bool {
        matches!(
            self.tokens.get(self.index + 1).map(|token| &token.kind),
            Some(TokenKind::Colon)
        )
    }

    fn expect_echo(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Echo) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected echo", Some(token.span)))
        }
    }

    fn expect_print(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Print) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected print", Some(token.span)))
        }
    }

    fn expect_const(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Const) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected const", Some(token.span)))
        }
    }

    fn expect_if_like(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::If | TokenKind::Elseif) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected if", Some(token.span)))
        }
    }

    fn expect_while(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::While) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected while", Some(token.span)))
        }
    }

    fn expect_for(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::For) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected for", Some(token.span)))
        }
    }

    fn expect_foreach(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Foreach) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected foreach", Some(token.span)))
        }
    }

    fn expect_as(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::As) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected as", Some(token.span)))
        }
    }

    fn expect_do(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Do) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected do", Some(token.span)))
        }
    }

    fn expect_switch(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Switch) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected switch", Some(token.span)))
        }
    }

    fn expect_break(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Break) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected break", Some(token.span)))
        }
    }

    fn expect_continue(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Continue) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected continue", Some(token.span)))
        }
    }

    fn expect_function(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Function) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected function", Some(token.span)))
        }
    }

    fn expect_return(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Return) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected return", Some(token.span)))
        }
    }

    fn expect_try(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Try) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected try", Some(token.span)))
        }
    }

    fn expect_catch(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Catch) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected catch", Some(token.span)))
        }
    }

    fn expect_open_tag(&mut self) -> Result<()> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::OpenTag) {
            Ok(())
        } else {
            Err(Diagnostic::new("expected <?php open tag", Some(token.span)))
        }
    }

    fn expect_assignment_op(&mut self) -> Result<AssignmentOp> {
        let token = self.advance();
        match token.kind {
            TokenKind::Equal => Ok(AssignmentOp::Assign),
            TokenKind::PlusEqual => Ok(AssignmentOp::AddAssign),
            TokenKind::MinusEqual => Ok(AssignmentOp::SubtractAssign),
            TokenKind::AsteriskEqual => Ok(AssignmentOp::MultiplyAssign),
            TokenKind::AsteriskAsteriskEqual => Ok(AssignmentOp::PowerAssign),
            TokenKind::SlashEqual => Ok(AssignmentOp::DivideAssign),
            TokenKind::PercentEqual => Ok(AssignmentOp::ModuloAssign),
            TokenKind::DotEqual => Ok(AssignmentOp::ConcatAssign),
            TokenKind::AmpersandEqual => Ok(AssignmentOp::BitwiseAndAssign),
            TokenKind::PipeEqual => Ok(AssignmentOp::BitwiseOrAssign),
            TokenKind::CaretEqual => Ok(AssignmentOp::BitwiseXorAssign),
            TokenKind::ShiftLeftEqual => Ok(AssignmentOp::ShiftLeftAssign),
            TokenKind::ShiftRightEqual => Ok(AssignmentOp::ShiftRightAssign),
            _ => Err(Diagnostic::new("expected assignment", Some(token.span))),
        }
    }

    fn peek_is_assignment_op(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Equal
                | TokenKind::PlusEqual
                | TokenKind::MinusEqual
                | TokenKind::AsteriskEqual
                | TokenKind::AsteriskAsteriskEqual
                | TokenKind::SlashEqual
                | TokenKind::PercentEqual
                | TokenKind::DotEqual
                | TokenKind::AmpersandEqual
                | TokenKind::PipeEqual
                | TokenKind::CaretEqual
                | TokenKind::ShiftLeftEqual
                | TokenKind::ShiftRightEqual
        )
    }

    fn expect_equal(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Equal) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected equal", Some(token.span)))
        }
    }

    fn expect_statement_terminator(&mut self) -> Result<()> {
        match self.peek().kind {
            TokenKind::Semicolon => {
                self.advance();
                Ok(())
            }
            TokenKind::CloseTag | TokenKind::Eof => Ok(()),
            _ => Err(syntax_error_unexpected(self.peek(), None)),
        }
    }

    fn expect_const_statement_terminator(&mut self) -> Result<()> {
        match self.peek().kind {
            TokenKind::Semicolon => {
                self.advance();
                Ok(())
            }
            TokenKind::CloseTag | TokenKind::Eof => Ok(()),
            _ => Err(syntax_error_unexpected(self.peek(), Some("\",\" or \";\""))),
        }
    }

    fn expect_semicolon(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Semicolon) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected semicolon", Some(token.span)))
        }
    }

    fn expect_left_paren(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::LeftParen) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new(
                "expected left parenthesis",
                Some(token.span),
            ))
        }
    }

    fn expect_right_paren(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightParen) {
            Ok(token.span)
        } else {
            Err(syntax_error_unexpected(token, Some("\")\"")))
        }
    }

    fn expect_left_bracket(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::LeftBracket) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected left bracket", Some(token.span)))
        }
    }

    fn expect_right_bracket(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightBracket) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected right bracket", Some(token.span)))
        }
    }

    fn expect_colon(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Colon) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected colon", Some(token.span)))
        }
    }

    fn expect_left_brace(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::LeftBrace) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected left brace", Some(token.span)))
        }
    }

    fn expect_right_brace(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::RightBrace) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected right brace", Some(token.span)))
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.index + 1)
            .unwrap_or_else(|| self.peek())
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.index];
        self.index += 1;
        token
    }
}

fn syntax_error_unexpected(token: &Token, expecting: Option<&str>) -> Diagnostic {
    let unexpected = describe_unexpected_token(token);
    let message = match expecting {
        Some(expecting) => format!("syntax error, unexpected {unexpected}, expecting {expecting}"),
        None => format!("syntax error, unexpected {unexpected}"),
    };
    Diagnostic::parse_error(message, Some(token.span))
}

fn describe_unexpected_token(token: &Token) -> String {
    match &token.kind {
        TokenKind::Identifier(name) => format!("identifier \"{name}\""),
        TokenKind::Variable(name) => format!("variable \"${name}\""),
        TokenKind::String(value) => format!("string \"{}\"", escape_token_text(value)),
        TokenKind::InterpolatedString(_) => "encapsed string".to_string(),
        TokenKind::Int(value) => format!("integer \"{value}\""),
        TokenKind::Float(value) => format!("floating-point number \"{value}\""),
        TokenKind::Eof => "end of file".to_string(),
        _ => format!("token \"{}\"", token_text(&token.kind)),
    }
}

fn token_text(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::OpenTag => "<?php",
        TokenKind::CloseTag => "?>",
        TokenKind::InlineHtml(_) => "inline HTML",
        TokenKind::Echo => "echo",
        TokenKind::Print => "print",
        TokenKind::If => "if",
        TokenKind::Elseif => "elseif",
        TokenKind::Else => "else",
        TokenKind::Do => "do",
        TokenKind::While => "while",
        TokenKind::For => "for",
        TokenKind::Foreach => "foreach",
        TokenKind::As => "as",
        TokenKind::Switch => "switch",
        TokenKind::Case => "case",
        TokenKind::Default => "default",
        TokenKind::Break => "break",
        TokenKind::Continue => "continue",
        TokenKind::Return => "return",
        TokenKind::Try => "try",
        TokenKind::Catch => "catch",
        TokenKind::Goto => "goto",
        TokenKind::Const => "const",
        TokenKind::Function => "function",
        TokenKind::Identifier(_) => "identifier",
        TokenKind::String(_) => "string",
        TokenKind::InterpolatedString(_) => "encapsed string",
        TokenKind::Int(_) => "integer",
        TokenKind::Float(_) => "float",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::Null => "null",
        TokenKind::Variable(_) => "variable",
        TokenKind::Equal => "=",
        TokenKind::DoubleArrow => "=>",
        TokenKind::EqualEqual => "==",
        TokenKind::EqualEqualEqual => "===",
        TokenKind::NotEqual => "!=",
        TokenKind::NotEqualEqual => "!==",
        TokenKind::Spaceship => "<=>",
        TokenKind::Less => "<",
        TokenKind::LessEqual => "<=",
        TokenKind::ShiftLeft => "<<",
        TokenKind::ShiftLeftEqual => "<<=",
        TokenKind::Greater => ">",
        TokenKind::GreaterEqual => ">=",
        TokenKind::ShiftRight => ">>",
        TokenKind::ShiftRightEqual => ">>=",
        TokenKind::KeywordAnd => "and",
        TokenKind::KeywordOr => "or",
        TokenKind::KeywordXor => "xor",
        TokenKind::AndAnd => "&&",
        TokenKind::OrOr => "||",
        TokenKind::AmpersandEqual => "&=",
        TokenKind::PipeEqual => "|=",
        TokenKind::CaretEqual => "^=",
        TokenKind::PlusEqual => "+=",
        TokenKind::MinusEqual => "-=",
        TokenKind::PlusPlus => "++",
        TokenKind::MinusMinus => "--",
        TokenKind::ObjectOperator => "->",
        TokenKind::AsteriskEqual => "*=",
        TokenKind::AsteriskAsteriskEqual => "**=",
        TokenKind::SlashEqual => "/=",
        TokenKind::PercentEqual => "%=",
        TokenKind::DotEqual => ".=",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Asterisk => "*",
        TokenKind::AsteriskAsterisk => "**",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::Ampersand => "&",
        TokenKind::Pipe => "|",
        TokenKind::Caret => "^",
        TokenKind::Tilde => "~",
        TokenKind::Bang => "!",
        TokenKind::Backslash => "\\",
        TokenKind::Dot => ".",
        TokenKind::Comma => ",",
        TokenKind::Question => "?",
        TokenKind::Colon => ":",
        TokenKind::Semicolon => ";",
        TokenKind::LeftParen => "(",
        TokenKind::RightParen => ")",
        TokenKind::LeftBracket => "[",
        TokenKind::RightBracket => "]",
        TokenKind::LeftBrace => "{",
        TokenKind::RightBrace => "}",
        TokenKind::IntType => "int",
        TokenKind::IntegerType => "integer",
        TokenKind::FloatType => "float",
        TokenKind::DoubleType => "double",
        TokenKind::StringType => "string",
        TokenKind::BinaryType => "binary",
        TokenKind::BoolType => "bool",
        TokenKind::BooleanType => "boolean",
        TokenKind::Eof => "end of file",
    }
}

fn escape_token_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

#[derive(Debug, Clone)]
struct LabelInfo {
    control_path: Vec<usize>,
}

fn validate_goto_labels(statements: &[Statement]) -> Result<()> {
    let mut labels = HashMap::new();
    let mut control_path = Vec::new();
    collect_labels(statements, &mut labels, &mut control_path)?;
    validate_gotos(statements, &labels, &mut control_path)
}

fn validate_function_names(functions: &[FunctionDecl]) -> Result<()> {
    let mut names = HashSet::new();
    for function in functions {
        let lookup_name = function.name.to_ascii_lowercase();
        if is_modeled_internal_function_name(&lookup_name) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare function {lookup_name}()"),
                Some(function.span),
            ));
        }
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare function {lookup_name}()"),
                Some(function.span),
            ));
        }
    }
    Ok(())
}

fn is_modeled_internal_function_name(name: &str) -> bool {
    matches!(
        name,
        "var_dump"
            | "strlen"
            | "str_rot13"
            | "strcmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "quotemeta"
            | "chunk_split"
            | "strip_tags"
            | "md5"
            | "sha1"
            | "substr"
            | "dirname"
            | "bin2hex"
            | "hex2bin"
            | "quoted_printable_decode"
            | "soundex"
            | "ceil"
            | "floor"
            | "abs"
            | "sqrt"
            | "fdiv"
            | "intdiv"
            | "pi"
            | "getrandmax"
            | "getmypid"
            | "php_sapi_name"
            | "phpversion"
            | "print_r"
            | "bindec"
            | "hexdec"
            | "octdec"
            | "intval"
            | "chr"
            | "ord"
            | "error_reporting"
            | "gettype"
            | "is_array"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_scalar"
            | "is_finite"
            | "is_infinite"
            | "is_nan"
            | "define"
            | "constant"
            | "defined"
            | "function_exists"
            | "isset"
            | "empty"
            | "count"
            | "array_key_exists"
            | "array_pop"
            | "array_push"
            | "array_shift"
            | "current"
            | "end"
            | "key"
            | "next"
            | "prev"
            | "reset"
    )
}

fn is_array_cursor_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "end" | "next" | "prev" | "reset"
    )
}

fn is_direct_variable_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_, _) => true,
        Expr::Grouped { expr, .. } => is_direct_variable_argument(expr),
        _ => false,
    }
}

fn validate_array_cursor_mutation_call(
    name: &str,
    arguments: &[Expr],
    call_span: SourceSpan,
) -> Result<()> {
    if !is_array_cursor_mutation_name(name) || arguments.len() != 1 {
        return Ok(());
    }
    if is_direct_variable_argument(&arguments[0]) {
        return Ok(());
    }
    Err(Diagnostic::new(
        format!(
            "{}() requires a direct variable array argument; temporary array cursor mutation is unsupported",
            name.to_ascii_lowercase()
        ),
        Some(arguments.first().map_or(call_span, Expr::span)),
    ))
}

fn collect_labels(
    statements: &[Statement],
    labels: &mut HashMap<String, LabelInfo>,
    control_path: &mut Vec<usize>,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_labels(then_body, labels, control_path)?;
                collect_labels(else_body, labels, control_path)?;
            }
            Statement::Block { statements, .. } => {
                collect_labels(statements, labels, control_path)?;
            }
            Statement::While { body, span, .. } | Statement::DoWhile { body, span, .. } => {
                collect_control_labels(*span, body, labels, control_path)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                span,
                ..
            } => {
                collect_labels(initializers, labels, control_path)?;
                collect_labels(updates, labels, control_path)?;
                collect_control_labels(*span, body, labels, control_path)?;
            }
            Statement::Foreach { body, span, .. } => {
                collect_control_labels(*span, body, labels, control_path)?;
            }
            Statement::Try { body, catches, .. } => {
                collect_labels(body, labels, control_path)?;
                for catch in catches {
                    collect_labels(&catch.body, labels, control_path)?;
                }
            }
            Statement::Switch { cases, span, .. } => {
                control_path.push(span.byte_start);
                for case in cases {
                    collect_labels(&case.body, labels, control_path)?;
                }
                control_path.pop();
            }
            Statement::Label { name, span } => {
                let previous = labels.insert(
                    name.clone(),
                    LabelInfo {
                        control_path: control_path.clone(),
                    },
                );
                if previous.is_some() {
                    return Err(Diagnostic::new(
                        format!("Label '{name}' already defined"),
                        Some(*span),
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_control_labels(
    span: SourceSpan,
    body: &[Statement],
    labels: &mut HashMap<String, LabelInfo>,
    control_path: &mut Vec<usize>,
) -> Result<()> {
    control_path.push(span.byte_start);
    let result = collect_labels(body, labels, control_path);
    control_path.pop();
    result
}

fn validate_gotos(
    statements: &[Statement],
    labels: &HashMap<String, LabelInfo>,
    control_path: &mut Vec<usize>,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_gotos(then_body, labels, control_path)?;
                validate_gotos(else_body, labels, control_path)?;
            }
            Statement::Block { statements, .. } => {
                validate_gotos(statements, labels, control_path)?;
            }
            Statement::While { body, span, .. } | Statement::DoWhile { body, span, .. } => {
                validate_control_gotos(*span, body, labels, control_path)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                span,
                ..
            } => {
                validate_gotos(initializers, labels, control_path)?;
                validate_gotos(updates, labels, control_path)?;
                validate_control_gotos(*span, body, labels, control_path)?;
            }
            Statement::Foreach { body, span, .. } => {
                validate_control_gotos(*span, body, labels, control_path)?;
            }
            Statement::Try { body, catches, .. } => {
                validate_gotos(body, labels, control_path)?;
                for catch in catches {
                    validate_gotos(&catch.body, labels, control_path)?;
                }
            }
            Statement::Switch { cases, span, .. } => {
                control_path.push(span.byte_start);
                for case in cases {
                    validate_gotos(&case.body, labels, control_path)?;
                }
                control_path.pop();
            }
            Statement::Goto { label, span } => {
                let Some(target) = labels.get(label) else {
                    return Err(Diagnostic::new(
                        format!("'goto' to undefined label '{label}'"),
                        Some(*span),
                    ));
                };
                if !target_control_path_is_reachable(&target.control_path, control_path) {
                    return Err(Diagnostic::new(
                        "'goto' into loop or switch statement is disallowed",
                        Some(*span),
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_control_gotos(
    span: SourceSpan,
    body: &[Statement],
    labels: &HashMap<String, LabelInfo>,
    control_path: &mut Vec<usize>,
) -> Result<()> {
    control_path.push(span.byte_start);
    let result = validate_gotos(body, labels, control_path);
    control_path.pop();
    result
}

fn target_control_path_is_reachable(target: &[usize], current: &[usize]) -> bool {
    target.len() <= current.len()
        && target
            .iter()
            .zip(current)
            .all(|(target, current)| target == current)
}

fn combine_spans(left: SourceSpan, right: SourceSpan) -> SourceSpan {
    SourceSpan::new(left.byte_start, right.byte_end, left.line, left.column)
}

fn lower_string_part(part: TokenStringPart) -> StringPart {
    match part {
        TokenStringPart::Literal(value) => StringPart::Literal(value),
        TokenStringPart::Variable(name) => StringPart::Variable(name),
    }
}

fn is_supported_global_const_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _) => true,
        Expr::Array { elements, .. } => elements.iter().all(|element| {
            element
                .key
                .as_ref()
                .is_none_or(is_supported_global_const_expr)
                && is_supported_global_const_expr(&element.value)
        }),
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Grouped { expr, .. } => {
            is_supported_global_const_expr(expr)
        }
        Expr::Binary { left, right, .. } => {
            is_supported_global_const_expr(left) && is_supported_global_const_expr(right)
        }
        Expr::InterpolatedString(_, _)
        | Expr::Variable(_, _)
        | Expr::Call { .. }
        | Expr::MethodCall { .. }
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. } => false,
    }
}

fn magic_constant_kind(name: &str) -> Option<MagicConstantKind> {
    match name.to_ascii_uppercase().as_str() {
        "__LINE__" => Some(MagicConstantKind::Line),
        "__FILE__" => Some(MagicConstantKind::File),
        "__DIR__" => Some(MagicConstantKind::Dir),
        "__FUNCTION__" => Some(MagicConstantKind::Function),
        "__METHOD__" => Some(MagicConstantKind::Method),
        "__CLASS__" => Some(MagicConstantKind::Class),
        "__TRAIT__" => Some(MagicConstantKind::Trait),
        "__NAMESPACE__" => Some(MagicConstantKind::Namespace),
        _ => None,
    }
}
