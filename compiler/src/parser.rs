use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, ClassDecl, ClassMember, ClassMethodDecl, ClassPropertyDecl,
    ClassVisibility, Expr, FunctionDecl, FunctionParam, Program, Span, Stmt, UnaryOp,
};
use crate::error::{CompileResult, Diagnostic, Phase};
use crate::lexer::{tokenize, Token, TokenKind};

pub fn parse_source(source: &str) -> CompileResult<Program> {
    let tokens = tokenize(source)?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_program(mut self) -> CompileResult<Program> {
        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::Eof)) {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> CompileResult<Stmt> {
        match &self.peek().kind {
            TokenKind::Function => self.parse_function(),
            TokenKind::Class => self.parse_class(),
            TokenKind::Namespace => self.parse_unsupported_namespace(),
            TokenKind::Use => self.parse_unsupported_use(),
            TokenKind::Declare => self.parse_unsupported_declare(),
            TokenKind::Eval => self.parse_unsupported_eval(),
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Foreach => self.parse_unsupported_foreach(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Global => self.parse_global(),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("foreach") => {
                self.parse_unsupported_foreach()
            }
            kind if include_require_name(kind).is_some() => {
                self.parse_unsupported_include_or_require()
            }
            _ => self.parse_assignment_or_expression_statement(),
        }
    }

    fn parse_function(&mut self) -> CompileResult<Stmt> {
        let start = self
            .consume_keyword(TokenKind::Function, "expected 'function'")?
            .span;
        Ok(Stmt::Function(self.parse_function_after_keyword(start)?))
    }

    fn parse_function_after_keyword(&mut self, start: Span) -> CompileResult<FunctionDecl> {
        if self.check(|kind| matches!(kind, TokenKind::Ampersand)) {
            let span = self.advance().span;
            return Err(self.error_at(
                span,
                "unsupported reference return: returning functions by reference is not implemented",
            ));
        }
        let name = self.consume_identifier("expected function name")?;
        self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;

        let mut params = Vec::new();
        let mut saw_default = false;
        if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            loop {
                if self.check(|kind| matches!(kind, TokenKind::Ellipsis)) {
                    let span = self.advance().span;
                    return Err(self.error_at(
                        span,
                        "unsupported variadic parameter: variadics are not implemented",
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Ampersand)) {
                    let span = self.advance().span;
                    return Err(self.error_at(
                        span,
                        "unsupported reference parameter: references are not implemented",
                    ));
                }
                let (name, span) = self.consume_variable_with_span("expected parameter name")?;
                let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                    saw_default = true;
                    let expr = self.parse_expression()?;
                    self.ensure_supported_default_expr(&expr)?;
                    Some(expr)
                } else {
                    if saw_default {
                        return Err(self.error_at(
                            span,
                            "required parameter cannot follow a default parameter in the current subset",
                        ));
                    }
                    None
                };

                params.push(FunctionParam {
                    name,
                    default,
                    span,
                });
                if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
            }
        }

        self.consume_keyword(TokenKind::RParen, "expected ')' after parameter list")?;
        let body = self.parse_required_block("expected function body")?;

        Ok(FunctionDecl {
            name,
            params,
            body,
            span: start,
        })
    }

    fn parse_class(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Class, "expected 'class'")?
            .span;
        let name = self.consume_identifier("expected class name")?;

        if self.match_token(|kind| matches!(kind, TokenKind::Extends)) {
            return Err(self.error_at(
                self.previous().span,
                "unsupported class inheritance: extends is not implemented",
            ));
        }
        if self.match_token(|kind| matches!(kind, TokenKind::Implements)) {
            return Err(self.error_at(
                self.previous().span,
                "unsupported interface implementation: implements is not implemented",
            ));
        }

        self.consume_keyword(TokenKind::LBrace, "expected class body")?;
        let mut members = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            members.push(self.parse_class_member()?);
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after class body")?;

        Ok(Stmt::Class(ClassDecl {
            name,
            members,
            span,
        }))
    }

    fn parse_class_member(&mut self) -> CompileResult<ClassMember> {
        let (visibility, is_static) = self.parse_class_member_modifiers()?;

        if self.match_token(|kind| matches!(kind, TokenKind::Function)) {
            let span = self.previous().span;
            let function = self.parse_function_after_keyword(span)?;
            return Ok(ClassMember::Method(ClassMethodDecl {
                function,
                visibility,
                is_static,
                span,
            }));
        }

        if self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            let (name, span) = self.consume_variable_with_span("expected property name")?;
            if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                return Err(self.error_at(
                    self.previous().span,
                    "unsupported property default: property default values are not implemented",
                ));
            }
            if self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                return Err(self.error_at(
                    self.previous().span,
                    "unsupported property declaration: multiple properties in one declaration are not implemented",
                ));
            }
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after property declaration",
            )?;
            return Ok(ClassMember::Property(ClassPropertyDecl {
                name,
                visibility,
                is_static,
                span,
            }));
        }

        let token = self.peek().clone();
        Err(self.error_at(token.span, unsupported_class_member_message(&token.kind)))
    }

    fn parse_class_member_modifiers(&mut self) -> CompileResult<(ClassVisibility, bool)> {
        let mut visibility = None;
        let mut is_static = false;

        loop {
            let modifier = match &self.peek().kind {
                TokenKind::Public => Some(ClassVisibility::Public),
                TokenKind::Protected => Some(ClassVisibility::Protected),
                TokenKind::Private => Some(ClassVisibility::Private),
                TokenKind::Static => {
                    if is_static {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate static modifier in class member declaration",
                        ));
                    }
                    is_static = true;
                    self.advance();
                    continue;
                }
                _ => None,
            };

            if let Some(next_visibility) = modifier {
                if visibility.is_some() {
                    return Err(self.error_at(
                        self.peek().span,
                        "duplicate visibility modifier in class member declaration",
                    ));
                }
                visibility = Some(next_visibility);
                self.advance();
                continue;
            }

            break;
        }

        Ok((visibility.unwrap_or(ClassVisibility::Public), is_static))
    }

    fn parse_unsupported_declare(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Declare, "expected 'declare'")?
            .span;
        Err(self.error_at(
            span,
            "unsupported declare directive: strict_types is not implemented",
        ))
    }

    fn parse_unsupported_namespace(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Namespace, "expected 'namespace'")?
            .span;
        Err(self.error_at(span, unsupported_namespace_message()))
    }

    fn parse_unsupported_use(&mut self) -> CompileResult<Stmt> {
        let span = self.consume_keyword(TokenKind::Use, "expected 'use'")?.span;
        Err(self.error_at(span, unsupported_use_message()))
    }

    fn parse_unsupported_eval(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Eval, "expected 'eval'")?
            .span;
        Err(self.error_at(span, unsupported_eval_message()))
    }

    fn parse_unsupported_include_or_require(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let construct =
            include_require_name(&token.kind).expect("caller checks include/require keyword");
        Err(self.error_at(token.span, unsupported_include_require_message(construct)))
    }

    fn parse_echo(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Echo, "expected 'echo'")?
            .span;
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse_expression()?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after echo")?;
        Ok(Stmt::Echo { exprs, span })
    }

    fn parse_print(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Print, "expected 'print'")?
            .span;
        let expr = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after print")?;
        Ok(Stmt::Print { expr, span })
    }

    fn parse_if(&mut self) -> CompileResult<Stmt> {
        let span = self.consume_keyword(TokenKind::If, "expected 'if'")?.span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after if")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after if condition")?;
        let then_branch = self.parse_block_or_statement()?;
        let else_branch = if self.match_token(|kind| matches!(kind, TokenKind::Else)) {
            self.parse_block_or_statement()?
        } else {
            Vec::new()
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_while(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::While, "expected 'while'")?
            .span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after while")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after while condition")?;
        let body = self.parse_block_or_statement()?;
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_unsupported_foreach(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        Err(self.error_at(token.span, unsupported_foreach_message()))
    }

    fn parse_return(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Return, "expected 'return'")?
            .span;
        let value = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after return")?;
        Ok(Stmt::Return { value, span })
    }

    fn parse_global(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Global, "expected 'global'")?
            .span;
        let mut names = Vec::new();

        loop {
            names.push(self.consume_variable("expected variable name after global")?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }

        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after global declaration",
        )?;
        Ok(Stmt::Global { names, span })
    }

    fn parse_assignment_or_expression_statement(&mut self) -> CompileResult<Stmt> {
        if let Some(stmt) = self.try_parse_assignment_statement()? {
            return Ok(stmt);
        }

        self.parse_expression_statement()
    }

    fn try_parse_assignment_statement(&mut self) -> CompileResult<Option<Stmt>> {
        if !self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            return Ok(None);
        }

        let saved = self.current;
        let target = self.parse_assignment_target()?;
        if !self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
            self.current = saved;
            return Ok(None);
        }

        let span = target.span();
        let expr = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after assignment")?;
        Ok(Some(Stmt::Assign { target, expr, span }))
    }

    fn parse_assignment_target(&mut self) -> CompileResult<AssignTarget> {
        let token = self.advance().clone();
        let (name, span) = match token.kind {
            TokenKind::Variable(name) => (name, token.span),
            _ => unreachable!("caller checks assignment target start"),
        };

        if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            let index = if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                None
            } else {
                let index = self.parse_expression()?;
                self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
                Some(index)
            };
            return Ok(AssignTarget::ArrayIndex { name, index, span });
        }

        if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
            let operator_span = self.previous().span;
            let property = self.consume_object_property_name(operator_span)?;
            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                return Err(self.error_at(operator_span, unsupported_method_call_message()));
            }
            return Ok(AssignTarget::Property {
                object: name,
                property,
                span,
            });
        }

        Ok(AssignTarget::Variable { name, span })
    }

    fn parse_expression_statement(&mut self) -> CompileResult<Stmt> {
        let expr = self.parse_expression()?;
        let span = expr.span();
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expr { expr, span })
    }

    fn parse_block_or_statement(&mut self) -> CompileResult<Vec<Stmt>> {
        if self.match_token(|kind| matches!(kind, TokenKind::LBrace)) {
            return self.parse_block_after_open();
        }

        Ok(vec![self.parse_statement()?])
    }

    fn parse_required_block(&mut self, message: &str) -> CompileResult<Vec<Stmt>> {
        self.consume_keyword(TokenKind::LBrace, message)?;
        self.parse_block_after_open()
    }

    fn parse_block_after_open(&mut self) -> CompileResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            if self.check(|kind| matches!(kind, TokenKind::Class)) {
                return Err(self.error_at(
                    self.peek().span,
                    "unsupported nested class declaration: only top-level class declarations are implemented",
                ));
            }
            statements.push(self.parse_statement()?);
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after block")?;
        Ok(statements)
    }

    fn parse_expression(&mut self) -> CompileResult<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::EqualEqual)) {
                BinaryOp::Eq
            } else if self.match_token(|kind| matches!(kind, TokenKind::BangEqual)) {
                BinaryOp::Ne
            } else {
                break;
            };
            let right = self.parse_comparison()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_concat()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Less)) {
                BinaryOp::Lt
            } else if self.match_token(|kind| matches!(kind, TokenKind::LessEqual)) {
                BinaryOp::Le
            } else if self.match_token(|kind| matches!(kind, TokenKind::Greater)) {
                BinaryOp::Gt
            } else if self.match_token(|kind| matches!(kind, TokenKind::GreaterEqual)) {
                BinaryOp::Ge
            } else {
                break;
            };
            let right = self.parse_concat()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_concat(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_additive()?;
        while self.match_token(|kind| matches!(kind, TokenKind::Dot)) {
            let right = self.parse_additive()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Concat,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Plus)) {
                BinaryOp::Add
            } else if self.match_token(|kind| matches!(kind, TokenKind::Minus)) {
                BinaryOp::Sub
            } else {
                break;
            };
            let right = self.parse_multiplicative()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Star)) {
                BinaryOp::Mul
            } else if self.match_token(|kind| matches!(kind, TokenKind::Slash)) {
                BinaryOp::Div
            } else {
                break;
            };
            let right = self.parse_unary()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> CompileResult<Expr> {
        if self.match_token(|kind| matches!(kind, TokenKind::Minus)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
                span,
            });
        }

        if self.match_token(|kind| matches!(kind, TokenKind::Bang)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                let bracket_span = self.previous().span;
                if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                    return Err(self.error_at(
                        bracket_span,
                        "cannot use [] for reading; append syntax is only supported in assignments",
                    ));
                }

                let index = self.parse_expression()?;
                self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
                let span = expr.span();
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
                continue;
            }

            if self.match_token(|kind| matches!(kind, TokenKind::LParen)) {
                let span = expr.span();
                let args = self.parse_call_arguments_after_open()?;
                expr = Expr::DynamicCall {
                    callee: Box::new(expr),
                    args,
                    span,
                };
                continue;
            }

            if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                let operator_span = self.previous().span;
                let property = self.consume_object_property_name(operator_span)?;
                if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    return Err(self.error_at(operator_span, unsupported_method_call_message()));
                }

                let span = expr.span();
                expr = Expr::Property {
                    target: Box::new(expr),
                    property,
                    span,
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> CompileResult<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::StringLiteral(value) => Ok(Expr::String(value, token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::LBracket => self.parse_array_literal(token.span),
            TokenKind::Class => {
                Err(self.error_at(token.span, unsupported_class_expression_message()))
            }
            TokenKind::New => self.parse_new_expression(token.span),
            TokenKind::Function => Err(self.error_at(
                token.span,
                "unsupported closure: anonymous functions are not implemented",
            )),
            TokenKind::Fn => Err(self.error_at(
                token.span,
                "unsupported closure: arrow functions are not implemented",
            )),
            TokenKind::Eval => Err(self.error_at(token.span, unsupported_eval_message())),
            TokenKind::Foreach => Err(self.error_at(token.span, unsupported_foreach_message())),
            TokenKind::Include => {
                Err(self.error_at(token.span, unsupported_include_require_message("include")))
            }
            TokenKind::IncludeOnce => Err(self.error_at(
                token.span,
                unsupported_include_require_message("include_once"),
            )),
            TokenKind::Require => {
                Err(self.error_at(token.span, unsupported_include_require_message("require")))
            }
            TokenKind::RequireOnce => Err(self.error_at(
                token.span,
                unsupported_include_require_message("require_once"),
            )),
            TokenKind::Ampersand => Err(self.error_at(
                token.span,
                "unsupported reference expression: references are not implemented",
            )),
            TokenKind::Identifier(name) => {
                if name.eq_ignore_ascii_case("foreach") {
                    return Err(self.error_at(token.span, unsupported_foreach_message()));
                }
                if name.eq_ignore_ascii_case("array")
                    && self.check(|kind| matches!(kind, TokenKind::LParen))
                {
                    return Err(self.error_at(token.span, unsupported_long_array_literal_message()));
                }
                if name.eq_ignore_ascii_case("unset")
                    && self.check(|kind| matches!(kind, TokenKind::LParen))
                {
                    return Err(self.error_at(token.span, unsupported_unset_message()));
                }
                if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_namespace_qualified_function_name_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                    return self.reject_unsupported_static_member_access();
                }
                self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;
                let args = self.parse_call_arguments_after_open()?;
                Ok(Expr::Call {
                    name,
                    args,
                    span: token.span,
                })
            }
            TokenKind::Backslash => Err(self.error_at(
                token.span,
                unsupported_namespace_qualified_function_name_message(),
            )),
            TokenKind::Namespace if self.check(|kind| matches!(kind, TokenKind::Backslash)) => {
                Err(self.error_at(
                    token.span,
                    unsupported_namespace_qualified_function_name_message(),
                ))
            }
            TokenKind::Static if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) => {
                self.reject_unsupported_static_member_access()
            }
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.consume_keyword(TokenKind::RParen, "expected ')' after expression")?;
                Ok(expr)
            }
            other => Err(self.error_at(
                token.span,
                format!("expected expression, found {}", token_name(&other)),
            )),
        }
    }

    fn reject_unsupported_static_member_access(&mut self) -> CompileResult<Expr> {
        let operator_span = self
            .consume_keyword(TokenKind::DoubleColon, "expected '::'")?
            .span;
        let member = self.peek();
        match &member.kind {
            TokenKind::Variable(_) => Err(self.error_at(
                operator_span,
                "unsupported static property access: static property storage is not implemented",
            )),
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::LParen) => {
                Err(self.error_at(
                    operator_span,
                    "unsupported static method call: static method dispatch is not implemented",
                ))
            }
            TokenKind::Identifier(_) => Err(self.error_at(
                operator_span,
                "unsupported class constant access: class constants are not implemented",
            )),
            TokenKind::Class => Err(self.error_at(
                operator_span,
                "unsupported class constant access: class constants and ::class are not implemented",
            )),
            _ => Err(self.error_at(
                operator_span,
                format!(
                    "expected static member name after '::', found {}",
                    token_name(&member.kind)
                ),
            )),
        }
    }

    fn parse_new_expression(&mut self, span: Span) -> CompileResult<Expr> {
        if self.check(|kind| matches!(kind, TokenKind::Class)) {
            let token = self.advance().clone();
            return Err(self.error_at(
                token.span,
                "unsupported anonymous class: anonymous classes are not implemented",
            ));
        }

        if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_namespace_qualified_class_name_message(),
            ));
        }

        let token = self.advance().clone();
        let class_name = match token.kind {
            TokenKind::Identifier(name) => {
                if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_namespace_qualified_class_name_message(),
                    ));
                }
                name
            }
            TokenKind::Namespace if self.check(|kind| matches!(kind, TokenKind::Backslash)) => {
                return Err(self.error_at(
                    token.span,
                    unsupported_namespace_qualified_class_name_message(),
                ));
            }
            _ => return Err(self.error_at(token.span, "expected class name after 'new'")),
        };
        self.consume_keyword(TokenKind::LParen, "expected '(' after class name")?;
        let args = self.parse_call_arguments_after_open()?;
        Ok(Expr::New {
            class_name,
            args,
            span,
        })
    }

    fn parse_array_literal(&mut self, span: Span) -> CompileResult<Expr> {
        let mut items = Vec::new();
        if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
            return Ok(Expr::Array { items, span });
        }

        loop {
            let first = self.parse_expression()?;
            let item = if self.match_token(|kind| matches!(kind, TokenKind::FatArrow)) {
                ArrayItem {
                    key: Some(first),
                    value: self.parse_expression()?,
                }
            } else {
                ArrayItem {
                    key: None,
                    value: first,
                }
            };
            items.push(item);

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                break;
            }
        }

        self.consume_keyword(TokenKind::RBracket, "expected ']' after array literal")?;
        Ok(Expr::Array { items, span })
    }

    fn parse_call_arguments_after_open(&mut self) -> CompileResult<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            loop {
                self.reject_unsupported_call_argument_syntax()?;
                args.push(self.parse_expression()?);
                if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
            }
        }
        self.consume_keyword(TokenKind::RParen, "expected ')' after arguments")?;
        Ok(args)
    }

    fn reject_unsupported_call_argument_syntax(&self) -> CompileResult<()> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Ellipsis => Err(self.error_at(
                token.span,
                "unsupported argument unpacking: variadic calls are not implemented",
            )),
            TokenKind::Ampersand => Err(self.error_at(
                token.span,
                "unsupported reference argument: references are not implemented",
            )),
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::Colon) => {
                Err(self.error_at(
                    token.span,
                    "unsupported named argument: named arguments are not implemented",
                ))
            }
            _ => Ok(()),
        }
    }

    fn ensure_supported_default_expr(&self, expr: &Expr) -> CompileResult<()> {
        match expr {
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _) => Ok(()),
            Expr::Array { items, .. } => {
                for item in items {
                    if let Some(key) = &item.key {
                        self.ensure_supported_default_expr(key)?;
                    }
                    self.ensure_supported_default_expr(&item.value)?;
                }
                Ok(())
            }
            Expr::Unary { expr, .. } => self.ensure_supported_default_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.ensure_supported_default_expr(left)?;
                self.ensure_supported_default_expr(right)
            }
            Expr::Variable(_, _)
            | Expr::Index { .. }
            | Expr::Property { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::New { .. } => Err(self.error_at(
                expr.span(),
                "default parameter values only support constant expressions in the current subset",
            )),
        }
    }

    fn consume_object_property_name(&mut self, operator_span: Span) -> CompileResult<String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok(name),
            TokenKind::Variable(_) => Err(self.error_at(
                token.span,
                "unsupported dynamic property access: dynamic property names are not implemented",
            )),
            _ => Err(self.error_at(
                operator_span,
                format!(
                    "expected property name after '->', found {}",
                    token_name(&token.kind)
                ),
            )),
        }
    }

    fn consume_identifier(&mut self, message: &str) -> CompileResult<String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok(name),
            _ => Err(self.error_at(token.span, message)),
        }
    }

    fn consume_variable(&mut self, message: &str) -> CompileResult<String> {
        self.consume_variable_with_span(message)
            .map(|(name, _span)| name)
    }

    fn consume_variable_with_span(&mut self, message: &str) -> CompileResult<(String, Span)> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Variable(name) => Ok((name, token.span)),
            _ => Err(self.error_at(token.span, message)),
        }
    }

    fn consume_keyword(&mut self, expected: TokenKind, message: &str) -> CompileResult<Token> {
        if same_variant(&self.peek().kind, &expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error_at(self.peek().span, message))
        }
    }

    fn match_token(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        if predicate(&self.peek().kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        predicate(&self.peek().kind)
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;
        if !matches!(self.tokens[index].kind, TokenKind::Eof) {
            self.current += 1;
        }
        &self.tokens[index]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.current + 1)
            .unwrap_or_else(|| self.peek())
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error_at(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Parse, span.line, span.column, message)
    }
}

fn same_variant(left: &TokenKind, right: &TokenKind) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn token_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Eof => "end of file",
        TokenKind::Variable(_) => "variable",
        TokenKind::Identifier(_) => "identifier",
        TokenKind::Int(_) => "integer literal",
        TokenKind::Float(_) => "float literal",
        TokenKind::StringLiteral(_) => "string literal",
        TokenKind::Echo => "echo",
        TokenKind::Print => "print",
        TokenKind::Function => "function",
        TokenKind::Fn => "fn",
        TokenKind::Class => "class",
        TokenKind::New => "new",
        TokenKind::Public => "public",
        TokenKind::Protected => "protected",
        TokenKind::Private => "private",
        TokenKind::Static => "static",
        TokenKind::Extends => "extends",
        TokenKind::Implements => "implements",
        TokenKind::Return => "return",
        TokenKind::Global => "global",
        TokenKind::Namespace => "namespace",
        TokenKind::Use => "use",
        TokenKind::Declare => "declare",
        TokenKind::Eval => "eval",
        TokenKind::Include => "include",
        TokenKind::IncludeOnce => "include_once",
        TokenKind::Require => "require",
        TokenKind::RequireOnce => "require_once",
        TokenKind::If => "if",
        TokenKind::Else => "else",
        TokenKind::While => "while",
        TokenKind::Foreach => "foreach",
        TokenKind::Null => "null",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::LParen => "(",
        TokenKind::RParen => ")",
        TokenKind::LBrace => "{",
        TokenKind::RBrace => "}",
        TokenKind::LBracket => "[",
        TokenKind::RBracket => "]",
        TokenKind::Semicolon => ";",
        TokenKind::Comma => ",",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Dot => ".",
        TokenKind::ObjectOperator => "->",
        TokenKind::DoubleColon => "::",
        TokenKind::Backslash => "\\",
        TokenKind::Ellipsis => "...",
        TokenKind::Ampersand => "&",
        TokenKind::Colon => ":",
        TokenKind::Bang => "!",
        TokenKind::Equal => "=",
        TokenKind::FatArrow => "=>",
        TokenKind::EqualEqual => "==",
        TokenKind::BangEqual => "!=",
        TokenKind::Less => "<",
        TokenKind::LessEqual => "<=",
        TokenKind::Greater => ">",
        TokenKind::GreaterEqual => ">=",
    }
}

fn include_require_name(kind: &TokenKind) -> Option<&'static str> {
    match kind {
        TokenKind::Include => Some("include"),
        TokenKind::IncludeOnce => Some("include_once"),
        TokenKind::Require => Some("require"),
        TokenKind::RequireOnce => Some("require_once"),
        _ => None,
    }
}

fn unsupported_include_require_message(construct: &str) -> String {
    format!("unsupported {construct}: include/require resolution and execution are not implemented")
}

fn unsupported_eval_message() -> &'static str {
    "unsupported eval: eval parsing and caller-scope execution are not implemented"
}

fn unsupported_namespace_message() -> &'static str {
    "unsupported namespace declaration: namespace-aware name resolution is not implemented"
}

fn unsupported_use_message() -> &'static str {
    "unsupported use declaration: namespace imports are not implemented"
}

fn unsupported_namespace_qualified_function_name_message() -> &'static str {
    "unsupported namespace-qualified function name: namespace-aware function resolution is not implemented"
}

fn unsupported_namespace_qualified_class_name_message() -> &'static str {
    "unsupported namespace-qualified class name: namespace-aware class resolution is not implemented"
}

fn unsupported_long_array_literal_message() -> &'static str {
    "unsupported long array syntax: array(...) literals are not implemented; use short [] literals in the current subset"
}

fn unsupported_unset_message() -> &'static str {
    "unsupported unset: variable, array offset, and property removal are not implemented"
}

fn unsupported_foreach_message() -> &'static str {
    "unsupported foreach: array and object iteration are not implemented"
}

fn unsupported_class_expression_message() -> &'static str {
    "unsupported class expression: anonymous classes are not implemented"
}

fn unsupported_method_call_message() -> &'static str {
    "unsupported method call: method dispatch is not implemented"
}

fn unsupported_class_member_message(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(_) => {
            "unsupported class member: typed properties, constants, and modifiers beyond visibility/static are not implemented"
                .to_string()
        }
        TokenKind::Extends => "unsupported class inheritance: extends is not implemented".to_string(),
        TokenKind::Implements => {
            "unsupported interface implementation: implements is not implemented".to_string()
        }
        TokenKind::Use => "unsupported trait use: traits are not implemented".to_string(),
        _ => format!("expected class member, found {}", token_name(kind)),
    }
}
