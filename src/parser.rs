use std::collections::{HashMap, HashSet};

use crate::ast::{
    AnonymousFunction, ArrayDimTarget, ArrayElement, ArrayElementValue, AssignmentOp,
    AssignmentTarget, BinaryOp, CastKind, CatchClause, ClassDecl, ConstDeclaration, Expr,
    FunctionDecl, FunctionParameter, IncDecOp, ListAssignmentElement, ListAssignmentElementTarget,
    ListAssignmentTarget, MagicConstantKind, MethodDecl, Program, ReferenceTarget, Statement,
    StaticPropertyDecl, StringInterpolationIndex, StringPart, SwitchCase, TypeHint, UnaryOp,
    UnsetTarget,
};
use crate::diagnostic::{Diagnostic, Result, SourceSpan};
use crate::lexer::{
    lex, StringInterpolationIndex as TokenStringInterpolationIndex, StringPart as TokenStringPart,
    Token, TokenKind,
};

const KEYWORD_OR_PRECEDENCE: u8 = 1;
const KEYWORD_XOR_PRECEDENCE: u8 = 2;
const KEYWORD_AND_PRECEDENCE: u8 = 3;
const SYMBOL_OR_PRECEDENCE: u8 = 4;
const COALESCE_PRECEDENCE: u8 = 4;
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
const CLASS_CONSTANT_FETCH_UNSUPPORTED: &str =
    "class constant fetches are unsupported; class constants and enum cases require class metadata";

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

struct ForeachVariable {
    name: String,
    by_ref: bool,
    span: SourceSpan,
}

#[derive(Default)]
struct ClassModifiers {
    is_static: bool,
}

enum ParsedClassMember {
    Method(MethodDecl),
    StaticProperties(Vec<StaticPropertyDecl>),
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program> {
        self.expect_open_tag()?;
        let mut classes = Vec::new();
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Eof) {
            if matches!(self.peek().kind, TokenKind::OpenTag | TokenKind::CloseTag) {
                self.advance();
                continue;
            }
            if self.peek_starts_function_decl() {
                functions.push(self.parse_function_decl()?);
            } else if token_is_identifier_named(self.peek(), "class") {
                classes.push(self.parse_class_decl()?);
            } else {
                statements.push(self.parse_statement()?);
            }
        }
        validate_class_names(&classes)?;
        validate_parent_class_names(&classes)?;
        for class in &classes {
            validate_method_names(class)?;
            for method in &class.methods {
                if method.return_by_ref {
                    validate_by_reference_returns_in_statements(
                        &method.body,
                        &format!("{}::{}", class.name, method.name),
                    )?;
                }
                validate_anonymous_functions_in_statements(&method.body, &functions)?;
                validate_reference_assignment_sources(&method.body, &functions)?;
                validate_goto_labels(&method.body)?;
            }
        }
        validate_function_names(&functions)?;
        validate_by_reference_returns(&functions)?;
        validate_anonymous_functions_in_statements(&statements, &functions)?;
        validate_reference_assignment_sources(&statements, &functions)?;
        for function in &functions {
            validate_anonymous_functions_in_statements(&function.body, &functions)?;
            validate_reference_assignment_sources(&function.body, &functions)?;
            validate_goto_labels(&function.body)?;
        }
        validate_goto_labels(&statements)?;
        Ok(Program {
            classes,
            functions,
            statements,
        })
    }

    fn parse_class_decl(&mut self) -> Result<ClassDecl> {
        let class_token = self.advance().clone();
        let TokenKind::Identifier(keyword) = &class_token.kind else {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        };
        if !keyword.eq_ignore_ascii_case("class") {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        }

        let name_token = self.advance().clone();
        let TokenKind::Identifier(class_name) = name_token.kind else {
            return Err(Diagnostic::new(
                "expected class name",
                Some(name_token.span),
            ));
        };
        let parent_name = if token_is_identifier_named(self.peek(), "extends") {
            self.advance();
            let parent_token = self.advance().clone();
            let TokenKind::Identifier(parent_name) = parent_token.kind else {
                return Err(Diagnostic::new(
                    "expected parent class name",
                    Some(parent_token.span),
                ));
            };
            Some(parent_name)
        } else {
            None
        };
        if token_is_identifier_named(self.peek(), "implements") {
            return Err(Diagnostic::new(
                "interfaces are unsupported",
                Some(self.peek().span),
            ));
        }

        self.expect_left_brace()?;
        let mut static_properties = Vec::new();
        let mut methods = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            match self.parse_class_member()? {
                ParsedClassMember::Method(method) => methods.push(method),
                ParsedClassMember::StaticProperties(properties) => {
                    static_properties.extend(properties);
                }
            }
        }
        self.expect_right_brace()?;
        Ok(ClassDecl {
            name: class_name,
            parent_name,
            static_properties,
            methods,
            span: class_token.span,
        })
    }

    fn parse_class_member(&mut self) -> Result<ParsedClassMember> {
        let modifiers = self.parse_class_modifiers()?;
        if matches!(self.peek().kind, TokenKind::Const) {
            return Err(Diagnostic::new(
                CLASS_CONSTANT_FETCH_UNSUPPORTED,
                Some(self.peek().span),
            ));
        }
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            if modifiers.is_static {
                return Ok(ParsedClassMember::StaticProperties(
                    self.parse_static_property_declarations()?,
                ));
            }
            return Err(Diagnostic::new(
                "class properties are unsupported",
                Some(self.peek().span),
            ));
        }
        if !matches!(self.peek().kind, TokenKind::Function) {
            return Err(Diagnostic::new(
                "unsupported class member",
                Some(self.peek().span),
            ));
        }
        Ok(ParsedClassMember::Method(
            self.parse_method_decl(modifiers)?,
        ))
    }

    fn parse_class_modifiers(&mut self) -> Result<ClassModifiers> {
        let mut modifiers = ClassModifiers::default();
        loop {
            let TokenKind::Identifier(modifier) = &self.peek().kind else {
                break;
            };
            match modifier.to_ascii_lowercase().as_str() {
                "public" => {
                    self.advance();
                }
                "static" => {
                    modifiers.is_static = true;
                    self.advance();
                }
                "private" | "protected" => {
                    return Err(Diagnostic::new(
                        "non-public class members are unsupported",
                        Some(self.peek().span),
                    ));
                }
                "abstract" => {
                    return Err(Diagnostic::new(
                        "abstract class methods are unsupported",
                        Some(self.peek().span),
                    ));
                }
                "final" => {
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(modifiers)
    }

    fn parse_static_property_declarations(&mut self) -> Result<Vec<StaticPropertyDecl>> {
        let mut properties = vec![self.parse_static_property_declaration()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            properties.push(self.parse_static_property_declaration()?);
        }
        self.expect_semicolon()?;
        Ok(properties)
    }

    fn parse_static_property_declaration(&mut self) -> Result<StaticPropertyDecl> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected static property name",
                Some(token.span),
            ));
        };
        let value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            let value = self.parse_expr()?;
            if !is_supported_global_const_expr(&value) {
                return Err(Diagnostic::new(
                    "static property default value must be a supported constant expression",
                    Some(value.span()),
                ));
            }
            Some(value)
        } else {
            None
        };
        Ok(StaticPropertyDecl {
            name,
            value,
            span: token.span,
        })
    }

    fn parse_method_decl(&mut self, modifiers: ClassModifiers) -> Result<MethodDecl> {
        let span = self.expect_function()?;
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        };
        let name_token = self.advance().clone();
        let TokenKind::Identifier(name) = name_token.kind else {
            return Err(Diagnostic::new(
                "expected method name",
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
        Ok(MethodDecl {
            name,
            parameters,
            return_type,
            return_by_ref,
            is_static: modifiers.is_static,
            body,
            span,
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
            TokenKind::Identifier(ref name) if is_unsupported_class_like_declaration(name) => {
                self.reject_unsupported_class_like_declaration()
            }
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::Colon) => {
                self.parse_label()
            }
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("unset")
                    && matches!(self.peek_next().kind, TokenKind::LeftParen) =>
            {
                self.parse_unset_statement()
            }
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("list")
                    && matches!(self.peek_next().kind, TokenKind::LeftParen) =>
            {
                self.parse_expression_statement()
            }
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::DoubleColon) => {
                self.parse_expression_statement()
            }
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::LeftParen) => {
                self.parse_call_statement()
            }
            TokenKind::Variable(_) if matches!(self.peek_next().kind, TokenKind::LeftParen) => {
                self.parse_expression_statement()
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
        let mut return_by_ref_span = None;
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            return_by_ref_span = Some(self.advance().span);
            true
        } else {
            false
        };
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
        let _ = return_by_ref_span;
        self.function_depth += 1;
        let body = self.parse_block();
        self.function_depth -= 1;
        let body = body?;
        Ok(FunctionDecl {
            name,
            parameters,
            return_type,
            return_by_ref,
            body,
            span,
        })
    }

    fn parse_anonymous_function_expr(&mut self, span: SourceSpan) -> Result<Expr> {
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        };
        let parameters = self.parse_function_parameters()?;
        if self.peek_is_identifier("use") {
            return Err(Diagnostic::new(
                "closure use captures are unsupported",
                Some(self.peek().span),
            ));
        }
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
        Ok(Expr::AnonymousFunction(AnonymousFunction {
            parameters,
            return_type,
            return_by_ref,
            body,
            span,
        }))
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
        let type_hint = if self.peek_is_type_hint() {
            Some(self.parse_type_hint()?)
        } else {
            None
        };
        let by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
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
            by_ref,
            span: token.span,
        })
    }

    fn parse_type_hint(&mut self) -> Result<TypeHint> {
        let token = self.advance();
        match token.kind {
            TokenKind::Null => Ok(TypeHint::Null),
            TokenKind::IntType | TokenKind::IntegerType => Ok(TypeHint::Int),
            TokenKind::FloatType | TokenKind::DoubleType => Ok(TypeHint::Float),
            TokenKind::StringType | TokenKind::BinaryType => Ok(TypeHint::String),
            TokenKind::BoolType | TokenKind::BooleanType => Ok(TypeHint::Bool),
            _ => Err(Diagnostic::new("expected type hint", Some(token.span))),
        }
    }

    fn peek_is_type_hint(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Null
                | TokenKind::IntType
                | TokenKind::IntegerType
                | TokenKind::FloatType
                | TokenKind::DoubleType
                | TokenKind::StringType
                | TokenKind::BinaryType
                | TokenKind::BoolType
                | TokenKind::BooleanType
        )
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
            let op_span = self.peek().span;
            let op = self.expect_assignment_op()?;
            if matches!(op, AssignmentOp::CoalesceAssign) {
                validate_coalesce_assignment_target(
                    op,
                    &AssignmentTarget::ArrayDim(target.clone()),
                    op_span,
                )?;
                let value = self.parse_assignment_value_expr()?;
                self.expect_statement_terminator()?;
                return Ok(Statement::ArrayAssign {
                    target,
                    op,
                    value,
                    span: token.span,
                });
            }
            if matches!(op, AssignmentOp::Assign)
                && matches!(self.peek().kind, TokenKind::Ampersand)
            {
                self.advance();
                let source = self.parse_reference_source()?;
                if reference_source_is_variable(&source, &target.array) {
                    return Err(Diagnostic::new(
                        "recursive array references are unsupported",
                        Some(target.span),
                    ));
                }
                self.expect_statement_terminator()?;
                return Ok(Statement::ArrayAssignRef {
                    target,
                    source,
                    span: token.span,
                });
            }
            let value = self.parse_assignment_value_expr()?;
            if matches!(op, AssignmentOp::Assign) {
                validate_recursive_reference_assignment_value(
                    &AssignmentTarget::ArrayDim(target.clone()),
                    &value,
                )?;
            }
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
        if matches!(op, AssignmentOp::Assign) && matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            let source = self.parse_reference_source()?;
            if reference_source_is_array_dim_of(&source, &name) {
                return Err(Diagnostic::new(
                    "self-referential array-element aliases are unsupported",
                    Some(token.span),
                ));
            }
            self.expect_statement_terminator()?;
            return Ok(Statement::AssignRef {
                name,
                source,
                span: token.span,
            });
        }
        let value = self.parse_assignment_value_expr()?;
        if matches!(op, AssignmentOp::Assign) {
            validate_recursive_reference_assignment_value(
                &AssignmentTarget::Variable {
                    name: name.clone(),
                    span: token.span,
                },
                &value,
            )?;
        }
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
        let mut dimensions = Vec::new();
        let mut right_span = variable_span;
        while matches!(self.peek().kind, TokenKind::LeftBracket) {
            self.expect_left_bracket()?;
            let index = if matches!(self.peek().kind, TokenKind::RightBracket) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            right_span = self.expect_right_bracket()?;
            dimensions.push(index);
        }
        Ok(ArrayDimTarget {
            array,
            dimensions,
            span: combine_spans(variable_span, right_span),
        })
    }

    fn parse_reference_target(&mut self) -> Result<ReferenceTarget> {
        reference_target_from_expr(self.parse_expr()?)
    }

    fn parse_reference_source(&mut self) -> Result<Expr> {
        let source = self.parse_postfix_expr()?;
        validate_reference_source_expr(&source)?;
        Ok(source)
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
        let (key, value, value_by_ref) = if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            if first.by_ref {
                return Err(Diagnostic::new(
                    "Key element cannot be a reference",
                    Some(first.span),
                ));
            }
            self.advance();
            let value = self.parse_foreach_variable()?;
            (Some(first.name), value.name, value.by_ref)
        } else {
            (None, first.name, first.by_ref)
        };
        self.expect_right_paren()?;
        let body = self.parse_statement_body()?;
        Ok(Statement::Foreach {
            iterable,
            key,
            value,
            value_by_ref,
            body,
            span,
        })
    }

    fn parse_foreach_variable(&mut self) -> Result<ForeachVariable> {
        let mut by_ref = false;
        let mut span = self.peek().span;
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            by_ref = true;
            span = self.advance().span;
        }
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            return Err(Diagnostic::new(
                "foreach destructuring is unsupported",
                Some(self.peek().span),
            ));
        }
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Variable(name) => Ok(ForeachVariable { name, by_ref, span }),
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
        validate_mutating_array_internal_call(&name, &arguments, token.span)?;
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
        validate_mutating_array_internal_call(&name, &arguments, token.span)?;
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
            Expr::ArrayAccess { .. } => {
                let target = array_dim_target_from_expr(target)?;
                if target.dimensions.iter().any(Option::is_none) {
                    return Err(Diagnostic::new(
                        "append array access is unsupported in unset targets",
                        Some(target.span),
                    ));
                }
                Ok(UnsetTarget::ArrayDim(target))
            }
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
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr> {
        let left = self.parse_binary_expr(0)?;
        if !self.peek_is_expression_assignment_op() {
            reject_append_array_read(&left)?;
            return Ok(left);
        }

        let operator = self.advance().clone();
        let op = match operator.kind {
            TokenKind::Equal => AssignmentOp::Assign,
            TokenKind::QuestionQuestionEqual => AssignmentOp::CoalesceAssign,
            _ => unreachable!("peek_is_expression_assignment_op guards assignment token"),
        };
        let left_span = left.span();
        let target = assignment_target_from_expr(left).map_err(|_| {
            Diagnostic::new(
                "assignment expression target must be a variable, array dimension, or list",
                Some(operator.span),
            )
        })?;
        validate_coalesce_assignment_target(op, &target, operator.span)?;
        if matches!(op, AssignmentOp::Assign) && matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            let source = self.parse_reference_source()?;
            validate_reference_assignment_target_source(&target, &source, operator.span)?;
            let span = combine_spans(left_span, source.span());
            return Ok(Expr::AssignRef {
                target,
                source: Box::new(source),
                span,
            });
        }
        let value = self.parse_assignment_expr()?;
        if matches!(op, AssignmentOp::Assign) {
            validate_recursive_reference_assignment_value(&target, &value)?;
        }
        let span = combine_spans(left_span, value.span());
        Ok(Expr::Assign {
            target,
            op,
            value: Box::new(value),
            span,
        })
    }

    fn parse_assignment_value_expr(&mut self) -> Result<Expr> {
        let left = self.parse_binary_expr(SYMBOL_OR_PRECEDENCE)?;
        let value = if self.peek_is_expression_assignment_op() {
            let operator = self.advance().clone();
            let op = match operator.kind {
                TokenKind::Equal => AssignmentOp::Assign,
                TokenKind::QuestionQuestionEqual => AssignmentOp::CoalesceAssign,
                _ => unreachable!("peek_is_expression_assignment_op guards assignment token"),
            };
            let left_span = left.span();
            let target = assignment_target_from_expr(left).map_err(|_| {
                Diagnostic::new(
                    "assignment expression target must be a variable, array dimension, or list",
                    Some(operator.span),
                )
            })?;
            validate_coalesce_assignment_target(op, &target, operator.span)?;
            if matches!(op, AssignmentOp::Assign)
                && matches!(self.peek().kind, TokenKind::Ampersand)
            {
                self.advance();
                let source = self.parse_reference_source()?;
                validate_reference_assignment_target_source(&target, &source, operator.span)?;
                let span = combine_spans(left_span, source.span());
                return Ok(Expr::AssignRef {
                    target,
                    source: Box::new(source),
                    span,
                });
            }
            let right = self.parse_assignment_expr()?;
            if matches!(op, AssignmentOp::Assign) {
                validate_recursive_reference_assignment_value(&target, &right)?;
            }
            let span = combine_spans(left_span, right.span());
            Expr::Assign {
                target,
                op,
                value: Box::new(right),
                span,
            }
        } else {
            reject_append_array_read(&left)?;
            left
        };
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
            TokenKind::At => {
                let token = self.advance().clone();
                let expr = self.parse_binary_expr(POWER_PRECEDENCE)?;
                let span = combine_spans(token.span, expr.span());
                Ok(Expr::Unary {
                    op: UnaryOp::ErrorSuppress,
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
                    let index = if matches!(self.peek().kind, TokenKind::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expr()?))
                    };
                    let right_span = self.expect_right_bracket()?;
                    expr = Expr::ArrayAccess {
                        span: combine_spans(expr.span(), right_span),
                        array: Box::new(expr),
                        index,
                    };
                }
                TokenKind::ObjectOperator => {
                    let start_span = expr.span();
                    self.advance();
                    let member = self.advance().clone();
                    let TokenKind::Identifier(name) = member.kind else {
                        return Err(Diagnostic::new("expected member name", Some(member.span)));
                    };
                    if !matches!(self.peek().kind, TokenKind::LeftParen) {
                        expr = Expr::PropertyFetch {
                            receiver: Box::new(expr),
                            name,
                            span: combine_spans(start_span, member.span),
                        };
                        continue;
                    }
                    let (arguments, right_span) = self.parse_call_arguments()?;
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        name: name.to_ascii_lowercase(),
                        arguments,
                        span: combine_spans(start_span, right_span),
                    };
                }
                TokenKind::LeftParen => {
                    let start_span = expr.span();
                    let (arguments, right_span) = self.parse_call_arguments()?;
                    expr = Expr::DynamicCall {
                        callee: Box::new(expr),
                        arguments,
                        span: combine_spans(start_span, right_span),
                    };
                }
                TokenKind::DoubleColon => {
                    let scope_span = self.advance().span;
                    return Err(Diagnostic::new(
                        CLASS_CONSTANT_FETCH_UNSUPPORTED,
                        Some(scope_span),
                    ));
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
            TokenKind::Function => self.parse_anonymous_function_expr(token.span),
            TokenKind::New => self.parse_new_object_expr(token.span),
            TokenKind::Identifier(name) => {
                let lowercase = name.to_ascii_lowercase();
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    self.parse_static_member_expr(name, token.span)
                } else if matches!(self.peek().kind, TokenKind::LeftParen) {
                    match lowercase.as_str() {
                        "array" => self.parse_long_array_literal(token.span),
                        "isset" => self.parse_isset_expr(token.span),
                        "empty" => self.parse_empty_expr(token.span),
                        _ => {
                            let (arguments, right_span) = self.parse_call_arguments()?;
                            validate_mutating_array_internal_call(
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
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    return self.parse_static_member_expr(
                        name,
                        combine_spans(token.span, name_token.span),
                    );
                }
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

    fn parse_static_member_expr(
        &mut self,
        class_name: String,
        class_span: SourceSpan,
    ) -> Result<Expr> {
        let scope_span = self.advance().span;
        let member = self.advance().clone();
        if let TokenKind::Variable(member_name) = member.kind {
            return Ok(Expr::StaticPropertyFetch {
                class_name,
                name: member_name,
                span: combine_spans(class_span, member.span),
            });
        }
        let TokenKind::Identifier(member_name) = member.kind else {
            return Err(Diagnostic::new(
                CLASS_CONSTANT_FETCH_UNSUPPORTED,
                Some(scope_span),
            ));
        };
        if !matches!(self.peek().kind, TokenKind::LeftParen) {
            return Err(Diagnostic::new(
                CLASS_CONSTANT_FETCH_UNSUPPORTED,
                Some(scope_span),
            ));
        }
        let (arguments, right_span) = self.parse_call_arguments()?;
        Ok(Expr::Call {
            name: format!("{}::{}", class_name, member_name),
            arguments,
            span: combine_spans(class_span, right_span),
        })
    }

    fn parse_new_object_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let (class_name, class_span) = self.parse_new_object_class_name()?;
        let mut span = combine_spans(start_span, class_span);
        let arguments = if matches!(self.peek().kind, TokenKind::LeftParen) {
            let (arguments, right_span) = self.parse_call_arguments()?;
            span = combine_spans(start_span, right_span);
            arguments
        } else {
            Vec::new()
        };
        Ok(Expr::NewObject {
            class_name,
            arguments,
            span,
        })
    }

    fn parse_new_object_class_name(&mut self) -> Result<(String, SourceSpan)> {
        let leading_backslash = matches!(self.peek().kind, TokenKind::Backslash);
        let start_span = if leading_backslash {
            Some(self.advance().span)
        } else {
            None
        };
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected class name", Some(token.span)));
        };
        let span = start_span
            .map(|span| combine_spans(span, token.span))
            .unwrap_or(token.span);
        Ok((name, span))
    }

    fn reject_unsupported_class_like_declaration(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            unreachable!("caller guards unsupported class-like declaration identifiers")
        };
        Err(Diagnostic::new(
            format!("{} declarations are unsupported", name.to_ascii_lowercase()),
            Some(token.span),
        ))
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
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            let value = ArrayElementValue::Reference(self.parse_reference_target()?);
            if matches!(self.peek().kind, TokenKind::DoubleArrow) {
                return Err(Diagnostic::new(
                    "Key element cannot be a reference",
                    Some(self.peek().span),
                ));
            }
            return Ok(ArrayElement { key: None, value });
        }

        let first = self.parse_expr()?;
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let value = self.parse_array_element_value()?;
            Ok(ArrayElement {
                key: Some(first),
                value,
            })
        } else {
            Ok(ArrayElement {
                key: None,
                value: ArrayElementValue::Value(first),
            })
        }
    }

    fn parse_array_element_value(&mut self) -> Result<ArrayElementValue> {
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            return Ok(ArrayElementValue::Reference(self.parse_reference_target()?));
        }
        Ok(ArrayElementValue::Value(self.parse_expr()?))
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
            TokenKind::QuestionQuestion => Some((BinaryOp::Coalesce, COALESCE_PRECEDENCE, true)),
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
                | TokenKind::Function
                | TokenKind::New
                | TokenKind::Identifier(_)
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Tilde
                | TokenKind::At
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Backslash
        )
    }

    fn peek_is_identifier(&self, expected: &str) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case(expected)
        )
    }

    fn peek_starts_function_decl(&self) -> bool {
        if !matches!(self.peek().kind, TokenKind::Function) {
            return false;
        }
        let mut index = self.index + 1;
        if matches!(
            self.tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Ampersand)
        ) {
            index += 1;
        }
        matches!(
            self.tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Identifier(_))
        ) && matches!(
            self.tokens.get(index + 1).map(|token| &token.kind),
            Some(TokenKind::LeftParen)
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
            TokenKind::QuestionQuestionEqual => Ok(AssignmentOp::CoalesceAssign),
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
                | TokenKind::QuestionQuestionEqual
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

    fn peek_is_expression_assignment_op(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Equal | TokenKind::QuestionQuestionEqual
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
        TokenKind::New => "new",
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
        TokenKind::DoubleColon => "::",
        TokenKind::QuestionQuestion => "??",
        TokenKind::QuestionQuestionEqual => "??=",
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
        TokenKind::At => "@",
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

fn is_unsupported_class_like_declaration(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "class" | "enum" | "interface" | "trait"
    )
}

fn token_is_identifier_named(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case(expected))
}

fn validate_class_names(classes: &[ClassDecl]) -> Result<()> {
    let mut names = HashSet::new();
    for class in classes {
        let lookup_name = class.name.to_ascii_lowercase();
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot declare class {lookup_name}, because the name is already in use"),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn validate_parent_class_names(classes: &[ClassDecl]) -> Result<()> {
    let names = classes
        .iter()
        .map(|class| class.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for class in classes {
        let Some(parent_name) = &class.parent_name else {
            continue;
        };
        if parent_name.eq_ignore_ascii_case("stdClass") {
            continue;
        }
        if !names.contains(&parent_name.to_ascii_lowercase()) {
            return Err(Diagnostic::new(
                format!("Class \"{parent_name}\" not found"),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn validate_method_names(class: &ClassDecl) -> Result<()> {
    let mut names = HashSet::new();
    for method in &class.methods {
        let lookup_name = method.name.to_ascii_lowercase();
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare {}::{}()", class.name, lookup_name),
                Some(method.span),
            ));
        }
    }
    Ok(())
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

fn reference_source_is_variable(source: &Expr, variable: &str) -> bool {
    match source {
        Expr::Variable(name, _) => name == variable,
        Expr::Grouped { expr, .. } => reference_source_is_variable(expr, variable),
        _ => false,
    }
}

fn reference_source_is_array_dim_of(source: &Expr, variable: &str) -> bool {
    match source {
        Expr::ArrayAccess { array, .. } => match array.as_ref() {
            Expr::Variable(name, _) => name == variable,
            Expr::Grouped { expr, .. } => reference_source_is_array_dim_of(expr, variable),
            _ => false,
        },
        Expr::Grouped { expr, .. } => reference_source_is_array_dim_of(expr, variable),
        _ => false,
    }
}

fn validate_recursive_reference_assignment_value(
    target: &AssignmentTarget,
    value: &Expr,
) -> Result<()> {
    let variable = match target {
        AssignmentTarget::Variable { name, .. } => name,
        AssignmentTarget::ArrayDim(target) => &target.array,
        AssignmentTarget::Property { .. } => return Ok(()),
        AssignmentTarget::StaticProperty { .. } => return Ok(()),
        AssignmentTarget::List(_) => return Ok(()),
    };
    if let Some(diagnostic) = expr_array_literal_reference_to_variable(value, variable) {
        if matches!(target, AssignmentTarget::Variable { .. })
            && matches!(diagnostic, RecursiveReferenceDiagnostic::RecursiveArray(_))
        {
            return Ok(());
        }
        return Err(diagnostic.into_diagnostic());
    }
    Ok(())
}

enum RecursiveReferenceDiagnostic {
    RecursiveArray(SourceSpan),
    SameArrayElement(SourceSpan),
}

impl RecursiveReferenceDiagnostic {
    fn into_diagnostic(self) -> Diagnostic {
        match self {
            RecursiveReferenceDiagnostic::RecursiveArray(span) => {
                Diagnostic::new("recursive array references are unsupported", Some(span))
            }
            RecursiveReferenceDiagnostic::SameArrayElement(span) => {
                Diagnostic::new("same-array element references are unsupported", Some(span))
            }
        }
    }
}

fn expr_array_literal_reference_to_variable(
    expr: &Expr,
    variable: &str,
) -> Option<RecursiveReferenceDiagnostic> {
    match expr {
        Expr::Array { elements, .. } => elements
            .iter()
            .find_map(|element| array_element_reference_to_variable(element, variable)),
        Expr::Assign { value, .. }
        | Expr::AssignRef { source: value, .. }
        | Expr::Grouped { expr: value, .. } => {
            expr_array_literal_reference_to_variable(value, variable)
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::AnonymousFunction(_)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::PropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. }
        | Expr::Unary { .. }
        | Expr::Cast { .. }
        | Expr::Binary { .. } => None,
    }
}

fn array_element_reference_to_variable(
    element: &ArrayElement,
    variable: &str,
) -> Option<RecursiveReferenceDiagnostic> {
    element
        .key
        .as_ref()
        .and_then(|key| expr_array_literal_reference_to_variable(key, variable))
        .or_else(|| match &element.value {
            ArrayElementValue::Reference(target) => {
                reference_target_reference_to_variable(target, variable)
            }
            ArrayElementValue::Value(value) => {
                expr_array_literal_reference_to_variable(value, variable)
            }
        })
}

fn reference_target_reference_to_variable(
    target: &ReferenceTarget,
    variable: &str,
) -> Option<RecursiveReferenceDiagnostic> {
    match target {
        ReferenceTarget::Variable { name, span } if name == variable => {
            Some(RecursiveReferenceDiagnostic::RecursiveArray(*span))
        }
        ReferenceTarget::ArrayDim(target) if target.array == variable => {
            Some(RecursiveReferenceDiagnostic::SameArrayElement(target.span))
        }
        _ => None,
    }
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

fn validate_reference_source_expr(source: &Expr) -> Result<()> {
    match source {
        Expr::Variable(_, _) | Expr::Call { .. } => Ok(()),
        Expr::Grouped { expr, .. } => validate_reference_source_expr(expr),
        Expr::ArrayAccess { .. } => validate_variable_root_array_reference_expr(
            source,
            "append reference sources are unsupported",
            "temporary array offset references are unsupported",
        ),
        _ => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(source.span()),
        )),
    }
}

fn validate_variable_root_array_reference_expr(
    expr: &Expr,
    append_message: &str,
    temporary_message: &str,
) -> Result<()> {
    match expr {
        Expr::Variable(_, _) => Ok(()),
        Expr::Grouped { expr, .. } => {
            validate_variable_root_array_reference_expr(expr, append_message, temporary_message)
        }
        Expr::ArrayAccess { array, index, span } => {
            if index.is_none() {
                return Err(Diagnostic::new(append_message, Some(*span)));
            }
            match array.as_ref() {
                Expr::Variable(_, _) => Ok(()),
                Expr::Grouped { expr, .. } => match expr.as_ref() {
                    Expr::Variable(_, _) => Ok(()),
                    Expr::ArrayAccess { .. } => validate_variable_root_array_reference_expr(
                        expr.as_ref(),
                        append_message,
                        temporary_message,
                    ),
                    _ => Err(Diagnostic::new(temporary_message, Some(*span))),
                },
                Expr::ArrayAccess { .. } => validate_variable_root_array_reference_expr(
                    array.as_ref(),
                    append_message,
                    temporary_message,
                ),
                _ => Err(Diagnostic::new(temporary_message, Some(*span))),
            }
        }
        _ => Err(Diagnostic::new(temporary_message, Some(expr.span()))),
    }
}

fn validate_by_reference_returns(functions: &[FunctionDecl]) -> Result<()> {
    for function in functions {
        if function.return_by_ref {
            validate_by_reference_returns_in_statements(&function.body, &function.name)?;
        }
    }
    Ok(())
}

fn validate_anonymous_functions_in_statements(
    statements: &[Statement],
    functions: &[FunctionDecl],
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Assign { value, .. }
            | Statement::AssignRef { source: value, .. }
            | Statement::ArrayAssign { value, .. }
            | Statement::ArrayAssignRef { source: value, .. }
            | Statement::Print {
                expression: value, ..
            }
            | Statement::Expression {
                expression: value, ..
            }
            | Statement::Return {
                value: Some(value), ..
            } => {
                validate_anonymous_functions_in_expr(value, functions)?;
            }
            Statement::Call { arguments, .. }
            | Statement::Echo {
                expressions: arguments,
                ..
            } => {
                for argument in arguments {
                    validate_anonymous_functions_in_expr(argument, functions)?;
                }
            }
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    validate_anonymous_functions_in_expr(&declaration.value, functions)?;
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                validate_anonymous_functions_in_expr(condition, functions)?;
                validate_anonymous_functions_in_statements(then_body, functions)?;
                validate_anonymous_functions_in_statements(else_body, functions)?;
            }
            Statement::Block { statements, .. } => {
                validate_anonymous_functions_in_statements(statements, functions)?;
            }
            Statement::While {
                condition, body, ..
            }
            | Statement::DoWhile {
                condition, body, ..
            } => {
                validate_anonymous_functions_in_expr(condition, functions)?;
                validate_anonymous_functions_in_statements(body, functions)?;
            }
            Statement::For {
                initializers,
                condition,
                updates,
                body,
                ..
            } => {
                validate_anonymous_functions_in_statements(initializers, functions)?;
                if let Some(condition) = condition {
                    validate_anonymous_functions_in_expr(condition, functions)?;
                }
                validate_anonymous_functions_in_statements(updates, functions)?;
                validate_anonymous_functions_in_statements(body, functions)?;
            }
            Statement::Foreach { iterable, body, .. } => {
                validate_anonymous_functions_in_expr(iterable, functions)?;
                validate_anonymous_functions_in_statements(body, functions)?;
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                validate_anonymous_functions_in_expr(expression, functions)?;
                for case in cases {
                    if let Some(condition) = &case.condition {
                        validate_anonymous_functions_in_expr(condition, functions)?;
                    }
                    validate_anonymous_functions_in_statements(&case.body, functions)?;
                }
            }
            Statement::Try { body, catches, .. } => {
                validate_anonymous_functions_in_statements(body, functions)?;
                for catch in catches {
                    validate_anonymous_functions_in_statements(&catch.body, functions)?;
                }
            }
            Statement::Return { value: None, .. }
            | Statement::Increment { .. }
            | Statement::Unset { .. }
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => {}
        }
    }
    Ok(())
}

fn validate_anonymous_functions_in_expr(expr: &Expr, functions: &[FunctionDecl]) -> Result<()> {
    match expr {
        Expr::AnonymousFunction(function) => {
            if function.return_by_ref {
                validate_by_reference_returns_in_statements(&function.body, "{closure}")?;
            }
            validate_anonymous_functions_in_statements(&function.body, functions)?;
            validate_reference_assignment_sources(&function.body, functions)?;
            validate_goto_labels(&function.body)?;
        }
        Expr::Assign { value, .. } => {
            validate_anonymous_functions_in_expr(value, functions)?;
        }
        Expr::AssignRef { source, .. } => {
            validate_anonymous_functions_in_expr(source, functions)?;
        }
        Expr::Call { arguments, .. }
        | Expr::DynamicCall { arguments, .. }
        | Expr::MethodCall { arguments, .. }
        | Expr::NewObject { arguments, .. } => {
            for argument in arguments {
                validate_anonymous_functions_in_expr(argument, functions)?;
            }
            if let Expr::DynamicCall { callee, .. } = expr {
                validate_anonymous_functions_in_expr(callee, functions)?;
            }
            if let Expr::MethodCall { receiver, .. } = expr {
                validate_anonymous_functions_in_expr(receiver, functions)?;
            }
        }
        Expr::PropertyFetch { receiver, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
        }
        Expr::StaticPropertyFetch { .. } => {}
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    validate_anonymous_functions_in_expr(key, functions)?;
                }
                if let ArrayElementValue::Value(value) = &element.value {
                    validate_anonymous_functions_in_expr(value, functions)?;
                }
            }
        }
        Expr::ArrayAccess { array, index, .. } => {
            validate_anonymous_functions_in_expr(array, functions)?;
            if let Some(index) = index {
                validate_anonymous_functions_in_expr(index, functions)?;
            }
        }
        Expr::Isset { targets, .. } => {
            for target in targets {
                validate_anonymous_functions_in_expr(target, functions)?;
            }
        }
        Expr::Empty { target, .. } => {
            validate_anonymous_functions_in_expr(target, functions)?;
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Grouped { expr, .. } => {
            validate_anonymous_functions_in_expr(expr, functions)?;
        }
        Expr::Binary { left, right, .. } => {
            validate_anonymous_functions_in_expr(left, functions)?;
            validate_anonymous_functions_in_expr(right, functions)?;
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _) => {}
    }
    Ok(())
}

fn validate_by_reference_returns_in_statements(
    statements: &[Statement],
    function_name: &str,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Return { value, span } => {
                let Some(value) = value else {
                    return Err(Diagnostic::new(
                        "by-reference return requires a variable or array element",
                        Some(*span),
                    ));
                };
                validate_by_reference_return_value(value, function_name)?;
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_by_reference_returns_in_statements(then_body, function_name)?;
                validate_by_reference_returns_in_statements(else_body, function_name)?;
            }
            Statement::Block { statements, .. } => {
                validate_by_reference_returns_in_statements(statements, function_name)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_by_reference_returns_in_statements(body, function_name)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_by_reference_returns_in_statements(initializers, function_name)?;
                validate_by_reference_returns_in_statements(updates, function_name)?;
                validate_by_reference_returns_in_statements(body, function_name)?;
            }
            Statement::Try { body, catches, .. } => {
                validate_by_reference_returns_in_statements(body, function_name)?;
                for catch in catches {
                    validate_by_reference_returns_in_statements(&catch.body, function_name)?;
                }
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_by_reference_returns_in_statements(&case.body, function_name)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_reference_assignment_sources(
    statements: &[Statement],
    functions: &[FunctionDecl],
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::AssignRef { source, .. } | Statement::ArrayAssignRef { source, .. } => {
                validate_reference_assignment_source_expr(source, functions)?;
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_reference_assignment_sources(then_body, functions)?;
                validate_reference_assignment_sources(else_body, functions)?;
            }
            Statement::Block { statements, .. } => {
                validate_reference_assignment_sources(statements, functions)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_reference_assignment_sources(body, functions)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_reference_assignment_sources(initializers, functions)?;
                validate_reference_assignment_sources(updates, functions)?;
                validate_reference_assignment_sources(body, functions)?;
            }
            Statement::Try { body, catches, .. } => {
                validate_reference_assignment_sources(body, functions)?;
                for catch in catches {
                    validate_reference_assignment_sources(&catch.body, functions)?;
                }
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_reference_assignment_sources(&case.body, functions)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_reference_assignment_source_expr(
    source: &Expr,
    functions: &[FunctionDecl],
) -> Result<()> {
    match source {
        Expr::Grouped { expr, .. } => validate_reference_assignment_source_expr(expr, functions),
        Expr::Call { .. } => Ok(()),
        _ => Ok(()),
    }
}

fn validate_by_reference_return_value(value: &Expr, function_name: &str) -> Result<()> {
    match value {
        Expr::Variable(_, _) => Ok(()),
        Expr::Grouped { expr, .. } => validate_by_reference_return_value(expr, function_name),
        Expr::ArrayAccess {
            array: _,
            index,
            span,
        } => {
            if index.is_none() {
                return Err(Diagnostic::new(
                    "by-reference return requires a variable or array element",
                    Some(*span),
                ));
            }
            validate_variable_root_array_reference_expr(
                value,
                "by-reference return requires a variable or array element",
                "by-reference return requires a variable or array element",
            )
        }
        Expr::Call { name, span, .. } if name.eq_ignore_ascii_case(function_name) => {
            Err(Diagnostic::new(
                "recursive by-reference returns are unsupported",
                Some(*span),
            ))
        }
        Expr::Call { .. } => Ok(()),
        Expr::DynamicCall { span, .. } | Expr::MethodCall { span, .. } => Err(Diagnostic::new(
            "by-reference call-result returns are unsupported",
            Some(*span),
        )),
        _ => Err(Diagnostic::new(
            "by-reference return requires a variable or array element",
            Some(value.span()),
        )),
    }
}

fn is_modeled_internal_function_name(name: &str) -> bool {
    matches!(
        name,
        "_ptn_cow_debug_assert_balanced"
            | "_ptn_cow_debug_assert_counter"
            | "_ptn_cow_debug_counter"
            | "_ptn_cow_debug_reset"
            | "var_dump"
            | "addcslashes"
            | "addslashes"
            | "strlen"
            | "str_rot13"
            | "strcmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "stripcslashes"
            | "stripslashes"
            | "quotemeta"
            | "chunk_split"
            | "strip_tags"
            | "md5"
            | "sha1"
            | "sha1_file"
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
            | "file_exists"
            | "file_put_contents"
            | "intdiv"
            | "pi"
            | "getrandmax"
            | "getmypid"
            | "php_sapi_name"
            | "phpversion"
            | "print_r"
            | "bindec"
            | "hexdec"
            | "in_array"
            | "octdec"
            | "intval"
            | "chr"
            | "ord"
            | "error_reporting"
            | "func_get_arg"
            | "func_get_args"
            | "func_num_args"
            | "gettype"
            | "is_array"
            | "is_null"
            | "is_bool"
            | "is_dir"
            | "is_file"
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
            | "array_count_values"
            | "array_fill_keys"
            | "array_key_exists"
            | "array_merge_recursive"
            | "array_pop"
            | "array_push"
            | "array_replace_recursive"
            | "array_reverse"
            | "array_shift"
            | "array_sum"
            | "array_unshift"
            | "array_values"
            | "current"
            | "end"
            | "key"
            | "next"
            | "prev"
            | "reset"
            | "mkdir"
            | "rmdir"
            | "strtr"
            | "unlink"
    )
}

fn is_array_cursor_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "end" | "next" | "prev" | "reset"
    )
}

fn is_array_by_ref_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_pop" | "array_push" | "array_shift" | "array_unshift"
    )
}

fn is_unsupported_sort_family_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "sort"
            | "rsort"
            | "asort"
            | "arsort"
            | "ksort"
            | "krsort"
            | "natsort"
            | "natcasesort"
            | "usort"
            | "uasort"
            | "uksort"
            | "shuffle"
            | "array_multisort"
    )
}

fn is_direct_variable_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_, _) => true,
        Expr::Grouped { expr, .. } => is_direct_variable_argument(expr),
        _ => false,
    }
}

fn validate_mutating_array_internal_call(
    name: &str,
    arguments: &[Expr],
    call_span: SourceSpan,
) -> Result<()> {
    if is_unsupported_sort_family_mutation_name(name) {
        let normalized = name.to_ascii_lowercase();
        if arguments.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "{normalized}() mutates array arguments by reference; sort-family array mutation semantics are unsupported"
                ),
                Some(call_span),
            ));
        }
        if is_direct_variable_argument(&arguments[0]) {
            return Err(Diagnostic::new(
                format!(
                    "{normalized}() mutates array arguments by reference; sort-family array mutation semantics are unsupported"
                ),
                Some(arguments.first().map_or(call_span, Expr::span)),
            ));
        }
        return Err(Diagnostic::new(
            format!(
                "{normalized}() requires a direct variable array argument; sort-family array mutation targets are unsupported"
            ),
            Some(arguments.first().map_or(call_span, Expr::span)),
        ));
    }
    if arguments.is_empty() {
        return Ok(());
    }
    if is_array_cursor_mutation_name(name) && arguments.len() == 1 {
        if is_direct_variable_argument(&arguments[0]) {
            return Ok(());
        }
        return Err(Diagnostic::new(
            format!(
                "{}() requires a direct variable array argument; temporary array cursor mutation is unsupported",
                name.to_ascii_lowercase()
            ),
            Some(arguments.first().map_or(call_span, Expr::span)),
        ));
    }

    if !is_array_by_ref_mutation_name(name) {
        return Ok(());
    }

    if is_direct_variable_argument(&arguments[0]) {
        return Ok(());
    }
    Err(Diagnostic::new(
        format!(
            "{}() requires a direct variable array argument; non-variable array mutation targets are unsupported",
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

fn array_dim_target_from_expr(expr: Expr) -> Result<ArrayDimTarget> {
    let span = expr.span();
    let mut dimensions = Vec::new();
    let mut current = expr;
    loop {
        match current {
            Expr::ArrayAccess { array, index, .. } => {
                dimensions.push(index.map(|index| *index));
                current = *array;
            }
            Expr::Variable(array, variable_span) => {
                dimensions.reverse();
                return Ok(ArrayDimTarget {
                    array,
                    dimensions,
                    span: combine_spans(variable_span, span),
                });
            }
            _ => {
                return Err(Diagnostic::new(
                    "unsupported unset target",
                    Some(current.span()),
                ));
            }
        }
    }
}

fn reference_target_from_expr(expr: Expr) -> Result<ReferenceTarget> {
    let span = expr.span();
    match expr {
        Expr::Variable(name, span) => Ok(ReferenceTarget::Variable { name, span }),
        Expr::Grouped { expr, .. } => reference_target_from_expr(*expr),
        array_expr @ Expr::ArrayAccess { .. } => Ok(ReferenceTarget::ArrayDim(
            reference_array_dim_target_from_expr(array_expr, span)?,
        )),
        other => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(other.span()),
        )),
    }
}

fn reference_array_dim_target_from_expr(
    expr: Expr,
    target_span: SourceSpan,
) -> Result<ArrayDimTarget> {
    let mut dimensions = Vec::new();
    let mut current = expr;
    loop {
        match current {
            Expr::ArrayAccess { array, index, .. } => {
                dimensions.push(index.map(|index| *index));
                current = *array;
            }
            Expr::Grouped { expr, .. } => {
                current = *expr;
            }
            Expr::Variable(array, _) => {
                dimensions.reverse();
                return Ok(ArrayDimTarget {
                    array,
                    dimensions,
                    span: target_span,
                });
            }
            _ => {
                return Err(Diagnostic::new(
                    "temporary array offset references are unsupported",
                    Some(target_span),
                ));
            }
        }
    }
}

fn assignment_target_from_expr(expr: Expr) -> Result<AssignmentTarget> {
    match expr {
        Expr::Variable(name, span) => Ok(AssignmentTarget::Variable { name, span }),
        Expr::ArrayAccess { .. } => Ok(AssignmentTarget::ArrayDim(array_dim_target_from_expr(
            expr,
        )?)),
        Expr::PropertyFetch {
            receiver,
            name,
            span,
        } => Ok(AssignmentTarget::Property {
            receiver,
            name,
            span,
        }),
        Expr::StaticPropertyFetch {
            class_name,
            name,
            span,
        } => Ok(AssignmentTarget::StaticProperty {
            class_name,
            name,
            span,
        }),
        Expr::Array { elements, span } => Ok(AssignmentTarget::List(
            list_assignment_target_from_array_elements(elements, span)?,
        )),
        Expr::Call {
            name,
            arguments,
            span,
        } if name.eq_ignore_ascii_case("list") => {
            let elements = arguments
                .into_iter()
                .map(|argument| {
                    Ok(ListAssignmentElement {
                        key: None,
                        target: ListAssignmentElementTarget::Value(Box::new(
                            assignment_target_from_expr(argument)?,
                        )),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(AssignmentTarget::List(ListAssignmentTarget {
                elements,
                span,
            }))
        }
        Expr::Grouped { expr, .. } => assignment_target_from_expr(*expr),
        other => Err(Diagnostic::new(
            "unsupported assignment target",
            Some(other.span()),
        )),
    }
}

fn validate_coalesce_assignment_target(
    op: AssignmentOp,
    target: &AssignmentTarget,
    span: SourceSpan,
) -> Result<()> {
    if !matches!(op, AssignmentOp::CoalesceAssign) {
        return Ok(());
    }

    match target {
        AssignmentTarget::Variable { .. } => Ok(()),
        AssignmentTarget::ArrayDim(target) => {
            if target.dimensions.iter().any(Option::is_none) {
                return Err(Diagnostic::new(
                    "null coalescing assignment cannot use append array access",
                    Some(span),
                ));
            }
            Ok(())
        }
        AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::StaticProperty { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables and array/string offsets",
            Some(span),
        )),
        AssignmentTarget::List(_) => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables and array/string offsets",
            Some(span),
        )),
    }
}

fn validate_reference_assignment_target_source(
    target: &AssignmentTarget,
    source: &Expr,
    span: SourceSpan,
) -> Result<()> {
    match target {
        AssignmentTarget::Variable { name, span } => {
            if reference_source_is_array_dim_of(source, name) {
                return Err(Diagnostic::new(
                    "self-referential array-element aliases are unsupported",
                    Some(*span),
                ));
            }
        }
        AssignmentTarget::ArrayDim(target) => {
            if reference_source_is_variable(source, &target.array) {
                return Err(Diagnostic::new(
                    "recursive array references are unsupported",
                    Some(target.span),
                ));
            }
        }
        AssignmentTarget::Property { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::StaticProperty { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::List(_) => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
    }
    Ok(())
}

fn list_assignment_target_from_array_elements(
    elements: Vec<ArrayElement>,
    span: SourceSpan,
) -> Result<ListAssignmentTarget> {
    let mut lowered = Vec::with_capacity(elements.len());
    for element in elements {
        let target = match element.value {
            ArrayElementValue::Value(value) => {
                ListAssignmentElementTarget::Value(Box::new(assignment_target_from_expr(value)?))
            }
            ArrayElementValue::Reference(target) => ListAssignmentElementTarget::Reference(target),
        };
        lowered.push(ListAssignmentElement {
            key: element.key,
            target,
        });
    }
    Ok(ListAssignmentTarget {
        elements: lowered,
        span,
    })
}

fn reject_append_array_read(expr: &Expr) -> Result<()> {
    match expr {
        Expr::ArrayAccess { array, index, span } => {
            if index.is_none() {
                return Err(Diagnostic::new(
                    "append array access is only valid as an assignment target",
                    Some(*span),
                ));
            }
            reject_append_array_read(array)?;
            if let Some(index) = index {
                reject_append_array_read(index)?;
            }
        }
        Expr::Assign { value, .. } => reject_append_array_read(value)?,
        Expr::AssignRef { source, .. } => reject_append_array_read(source)?,
        Expr::Call { arguments, .. } => {
            for argument in arguments {
                reject_append_array_read(argument)?;
            }
        }
        Expr::DynamicCall {
            callee, arguments, ..
        } => {
            reject_append_array_read(callee)?;
            for argument in arguments {
                reject_append_array_read(argument)?;
            }
        }
        Expr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            reject_append_array_read(receiver)?;
            for argument in arguments {
                reject_append_array_read(argument)?;
            }
        }
        Expr::NewObject { arguments, .. } => {
            for argument in arguments {
                reject_append_array_read(argument)?;
            }
        }
        Expr::PropertyFetch { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        Expr::StaticPropertyFetch { .. } => {}
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    reject_append_array_read(key)?;
                }
                if let ArrayElementValue::Value(value) = &element.value {
                    reject_append_array_read(value)?;
                }
            }
        }
        Expr::Isset { targets, .. } => {
            for target in targets {
                reject_append_array_read(target)?;
            }
        }
        Expr::Empty { target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::Cast { expr: target, .. }
        | Expr::Grouped { expr: target, .. } => reject_append_array_read(target)?,
        Expr::Binary { left, right, .. } => {
            reject_append_array_read(left)?;
            reject_append_array_read(right)?;
        }
        Expr::InterpolatedString(_, _)
        | Expr::AnonymousFunction(_)
        | Expr::String(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _) => {}
    }
    Ok(())
}

fn combine_spans(left: SourceSpan, right: SourceSpan) -> SourceSpan {
    SourceSpan::new(left.byte_start, right.byte_end, left.line, left.column)
}

fn lower_string_part(part: TokenStringPart) -> StringPart {
    match part {
        TokenStringPart::Literal(value) => StringPart::Literal(value),
        TokenStringPart::Variable(name) => StringPart::Variable(name),
        TokenStringPart::ArrayAccess { array, indices } => StringPart::ArrayAccess {
            array,
            indices: indices
                .into_iter()
                .map(lower_string_interpolation_index)
                .collect(),
        },
    }
}

fn lower_string_interpolation_index(
    index: TokenStringInterpolationIndex,
) -> StringInterpolationIndex {
    match index {
        TokenStringInterpolationIndex::String(value) => StringInterpolationIndex::String(value),
        TokenStringInterpolationIndex::Int(value) => StringInterpolationIndex::Int(value),
        TokenStringInterpolationIndex::Variable(name) => StringInterpolationIndex::Variable(name),
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
                && match &element.value {
                    ArrayElementValue::Value(value) => is_supported_global_const_expr(value),
                    ArrayElementValue::Reference(_) => false,
                }
        }),
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Grouped { expr, .. } => {
            is_supported_global_const_expr(expr)
        }
        Expr::Binary { left, right, .. } => {
            is_supported_global_const_expr(left) && is_supported_global_const_expr(right)
        }
        Expr::InterpolatedString(_, _)
        | Expr::Variable(_, _)
        | Expr::Assign { .. }
        | Expr::AssignRef { .. }
        | Expr::AnonymousFunction(_)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::PropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
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
