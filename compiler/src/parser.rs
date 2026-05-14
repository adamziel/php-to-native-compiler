use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, ClassConstantDecl, ClassDecl, ClassMember,
    ClassMethodDecl, ClassPropertyDecl, ClassVisibility, ClosureCapture, CompoundAssignOp,
    ConstDeclarator, Expr, ForAction, FunctionDecl, FunctionParam, IncrementDecrementOp,
    IncrementDecrementPosition, Program, Span, StaticLocalDeclarator, Stmt, SwitchCase, TypeDecl,
    UnaryOp, UnsetTarget,
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
    nested_statement_depth: usize,
    function_body_depth: usize,
}

#[derive(Clone, Copy)]
enum SwitchBodyKind {
    Brace,
    Alternate,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            nested_statement_depth: 0,
            function_body_depth: 0,
        }
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
            TokenKind::Interface => self.parse_unsupported_interface_declaration(),
            TokenKind::Trait => self.parse_unsupported_trait_declaration(),
            TokenKind::Enum => self.parse_unsupported_enum_declaration(),
            TokenKind::Abstract | TokenKind::Final | TokenKind::Readonly
                if matches!(self.peek_next().kind, TokenKind::Class) =>
            {
                self.parse_unsupported_class_modifier_declaration()
            }
            TokenKind::Namespace => self.parse_unsupported_namespace(),
            TokenKind::Use => self.parse_unsupported_use(),
            TokenKind::Declare => self.parse_unsupported_declare(),
            TokenKind::Eval => self.parse_unsupported_eval(),
            TokenKind::Echo => self.parse_echo(),
            TokenKind::Print => self.parse_print(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Do => self.parse_do_while(),
            TokenKind::Foreach => self.parse_foreach(),
            TokenKind::For => self.parse_for(),
            TokenKind::Switch => self.parse_switch(),
            TokenKind::Match => self.parse_unsupported_match_expression(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::Throw => self.parse_unsupported_throw(),
            TokenKind::Try | TokenKind::Catch | TokenKind::Finally => {
                self.parse_unsupported_try_catch_finally()
            }
            TokenKind::Return => self.parse_return(),
            TokenKind::Global => self.parse_global(),
            TokenKind::Static
                if self.function_body_depth > 0
                    && matches!(self.peek_next().kind, TokenKind::Variable(_)) =>
            {
                self.parse_static_local_declaration()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("do") => self.parse_do_while(),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("foreach") => {
                self.parse_foreach()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("for") => self.parse_for(),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("switch") => {
                self.parse_switch()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("match") => {
                self.parse_unsupported_match_expression()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("break") => self.parse_break(),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("continue") => {
                self.parse_continue()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("throw") => {
                self.parse_unsupported_throw()
            }
            TokenKind::Identifier(name)
                if matches!(
                    name.to_ascii_lowercase().as_str(),
                    "try" | "catch" | "finally"
                ) =>
            {
                self.parse_unsupported_try_catch_finally()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("yield") => {
                Err(self.error_at(self.peek().span, unsupported_yield_message()))
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("goto") => self.parse_goto(),
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::Colon) => {
                self.parse_goto_label()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => {
                if self.nested_statement_depth == 0 {
                    self.parse_const_declaration()
                } else {
                    self.parse_unsupported_nested_const_declaration()
                }
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("unset") => self.parse_unset(),
            TokenKind::Include | TokenKind::IncludeOnce => self.parse_include(),
            TokenKind::Require | TokenKind::RequireOnce => self.parse_require(),
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
        let params = self.parse_function_params_after_open()?;

        let return_type = if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            Some(self.parse_type_decl(unsupported_return_type_message())?)
        } else {
            None
        };
        self.function_body_depth += 1;
        let body = self.parse_required_block("expected function body");
        self.function_body_depth -= 1;
        let body = body?;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: start,
        })
    }

    fn parse_function_params_after_open(&mut self) -> CompileResult<Vec<FunctionParam>> {
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
                let type_decl = if self.check(is_parameter_type_start) {
                    Some(self.parse_type_decl(unsupported_parameter_type_message())?)
                } else {
                    None
                };
                let by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
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
                    type_decl,
                    by_reference,
                    default,
                    span,
                });
                if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
                if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                    break;
                }
            }
        }

        self.consume_keyword(TokenKind::RParen, "expected ')' after parameter list")?;
        Ok(params)
    }

    fn parse_type_decl(&mut self, message: &'static str) -> CompileResult<TypeDecl> {
        let span = self.peek().span;
        let mut text = String::new();

        if self.match_token(|kind| matches!(kind, TokenKind::Question)) {
            text.push('?');
        }
        self.parse_type_name(&mut text, message)?;

        loop {
            let separator = match &self.peek().kind {
                TokenKind::Pipe => '|',
                TokenKind::Ampersand
                    if !matches!(self.peek_next().kind, TokenKind::Variable(_)) =>
                {
                    '&'
                }
                _ => break,
            };
            self.advance();
            text.push(separator);
            self.parse_type_name(&mut text, message)?;
        }

        Ok(TypeDecl { text, span })
    }

    fn parse_type_name(&mut self, text: &mut String, message: &'static str) -> CompileResult<()> {
        if self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            text.push('\\');
        }

        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => text.push_str(&name),
            TokenKind::Static => text.push_str("static"),
            TokenKind::Null => text.push_str("null"),
            TokenKind::True => text.push_str("true"),
            TokenKind::False => text.push_str("false"),
            _ => return Err(self.error_at(token.span, message)),
        }

        while self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            text.push('\\');
            let token = self.advance().clone();
            match token.kind {
                TokenKind::Identifier(name) => text.push_str(&name),
                _ => return Err(self.error_at(token.span, message)),
            }
        }

        Ok(())
    }

    fn parse_class(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Class, "expected 'class'")?
            .span;
        let name = self.consume_identifier("expected class name")?;

        let parent = if self.match_token(|kind| matches!(kind, TokenKind::Extends)) {
            Some(self.consume_identifier("expected parent class name after 'extends'")?)
        } else {
            None
        };
        if self.match_token(|kind| matches!(kind, TokenKind::Implements)) {
            return Err(self.error_at(
                self.previous().span,
                unsupported_interface_implementation_message(),
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
            parent,
            members,
            span,
        }))
    }

    fn parse_unsupported_trait_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Trait, "expected 'trait'")?
            .span;
        Err(self.error_at(span, unsupported_trait_declaration_message()))
    }

    fn parse_unsupported_interface_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Interface, "expected 'interface'")?
            .span;
        Err(self.error_at(span, unsupported_interface_declaration_message()))
    }

    fn parse_unsupported_enum_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Enum, "expected 'enum'")?
            .span;
        Err(self.error_at(span, unsupported_enum_declaration_message()))
    }

    fn parse_unsupported_class_modifier_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, unsupported_class_modifier_declaration_message()))
    }

    fn parse_class_member(&mut self) -> CompileResult<ClassMember> {
        let (visibility, is_static) = self.parse_class_member_modifiers()?;

        if self.match_identifier("const") {
            let span = self.previous().span;
            if is_static {
                return Err(self.error_at(
                    span,
                    "unsupported class constant declaration: static class constants are not implemented",
                ));
            }
            if matches!(self.peek().kind, TokenKind::Identifier(_))
                && matches!(self.peek_next().kind, TokenKind::Identifier(_))
            {
                return Err(self.error_at(
                    self.peek().span,
                    "unsupported class constant declaration: typed class constants are not implemented",
                ));
            }
            let (name, name_span) =
                self.consume_identifier_with_span("expected class constant name after const")?;
            self.consume_keyword(TokenKind::Equal, "expected '=' after class constant name")?;
            let value = self.parse_expression()?;
            self.ensure_supported_const_declaration_expr(&value)?;
            if self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                return Err(self.error_at(
                    self.previous().span,
                    "unsupported class constant declaration: multiple class constants in one declaration are not implemented",
                ));
            }
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after class constant declaration",
            )?;
            return Ok(ClassMember::Constant(ClassConstantDecl {
                name,
                visibility,
                value,
                span: name_span,
            }));
        }

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

        if self.check_unsupported_property_type_declaration() {
            let message = if is_static {
                unsupported_static_property_type_message()
            } else {
                unsupported_property_type_message()
            };
            return Err(self.error_at(self.peek().span, message));
        }

        if self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            let (name, span) = self.consume_variable_with_span("expected property name")?;
            let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                if !is_static {
                    return Err(self.error_at(
                        self.previous().span,
                        "unsupported property default: instance property default values are not implemented",
                    ));
                }
                let expr = self.parse_expression()?;
                self.ensure_supported_static_property_default_expr(&expr)?;
                Some(expr)
            } else {
                None
            };
            if self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                return Err(self.error_at(
                    self.previous().span,
                    unsupported_multiple_properties_message(),
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
                default,
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

    fn parse_unsupported_throw(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, unsupported_throw_message()))
    }

    fn parse_unsupported_try_catch_finally(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, unsupported_try_catch_finally_message()))
    }

    fn parse_unsupported_match_expression(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, unsupported_match_expression_message()))
    }

    fn parse_goto(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        let label = self.consume_identifier("expected label name after goto")?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after goto statement")?;
        Ok(Stmt::Goto { label, span })
    }

    fn parse_goto_label(&mut self) -> CompileResult<Stmt> {
        let (name, span) = self.consume_identifier_with_span("expected goto label")?;
        self.consume_keyword(TokenKind::Colon, "expected ':' after goto label")?;
        Ok(Stmt::Label { name, span })
    }

    fn parse_unsupported_nested_const_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, unsupported_nested_const_declaration_message()))
    }

    fn parse_const_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        let mut declarations = Vec::new();

        loop {
            let (name, name_span) =
                self.consume_identifier_with_span("expected constant name after const")?;
            if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_namespace_const_declaration_message(),
                ));
            }
            self.consume_keyword(TokenKind::Equal, "expected '=' after constant name")?;
            let value = self.parse_expression()?;
            self.ensure_supported_const_declaration_expr(&value)?;
            declarations.push(ConstDeclarator {
                name,
                value,
                span: if declarations.is_empty() {
                    span
                } else {
                    name_span
                },
            });

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }

        self.consume_keyword(TokenKind::Semicolon, "expected ';' after const declaration")?;
        Ok(Stmt::ConstDeclaration { declarations, span })
    }

    fn parse_unsupported_include_or_require(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let construct =
            include_require_name(&token.kind).expect("caller checks include/require keyword");
        Err(self.error_at(token.span, unsupported_include_require_message(construct)))
    }

    fn parse_require(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let once = match token.kind {
            TokenKind::Require => false,
            TokenKind::RequireOnce => true,
            _ => unreachable!("caller checks require keyword"),
        };
        let path = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after require")?;
        Ok(Stmt::Require {
            path,
            once,
            span: token.span,
        })
    }

    fn parse_include(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let once = match token.kind {
            TokenKind::Include => false,
            TokenKind::IncludeOnce => true,
            _ => unreachable!("caller checks include keyword"),
        };
        let path = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after include")?;
        Ok(Stmt::Include {
            path,
            once,
            span: token.span,
        })
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
        self.parse_if_after_keyword(span, "if")
    }

    fn parse_if_after_keyword(&mut self, span: Span, keyword: &str) -> CompileResult<Stmt> {
        let open_message = match keyword {
            "elseif" => "expected '(' after elseif",
            _ => "expected '(' after if",
        };
        let close_message = match keyword {
            "elseif" => "expected ')' after elseif condition",
            _ => "expected ')' after if condition",
        };

        self.consume_keyword(TokenKind::LParen, open_message)?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, close_message)?;
        if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            if keyword == "elseif" {
                return Err(self.error_at(self.previous().span, unsupported_if_alternate_message()));
            }
            return self.parse_alternate_if_after_condition(span, condition);
        }
        let then_branch = self.parse_block_or_statement()?;
        let else_branch = if let Some(elseif_span) = self.match_elseif() {
            vec![self.parse_if_after_keyword(elseif_span, "elseif")?]
        } else if self.match_else() {
            if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
                return Err(self.error_at(self.previous().span, unsupported_if_alternate_message()));
            }
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

    fn parse_alternate_if_after_condition(
        &mut self,
        span: Span,
        condition: Expr,
    ) -> CompileResult<Stmt> {
        let then_branch = self.parse_alternate_if_body()?;
        let else_branch = if let Some(elseif_span) = self.match_elseif() {
            let condition = self.parse_alternate_elseif_condition()?;
            vec![self.parse_alternate_if_after_condition(elseif_span, condition)?]
        } else if self.match_else() {
            self.consume_keyword(TokenKind::Colon, "expected ':' after else")?;
            let body = self.parse_alternate_if_body()?;
            self.consume_alternate_if_end()?;
            body
        } else {
            self.consume_alternate_if_end()?;
            Vec::new()
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    fn parse_alternate_elseif_condition(&mut self) -> CompileResult<Expr> {
        self.consume_keyword(TokenKind::LParen, "expected '(' after elseif")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after elseif condition")?;
        self.consume_keyword(TokenKind::Colon, "expected ':' after elseif condition")?;
        Ok(condition)
    }

    fn parse_alternate_if_body(&mut self) -> CompileResult<Vec<Stmt>> {
        self.nested_statement_depth += 1;
        let result = (|| {
            let mut statements = Vec::new();
            while !self.check_alternate_if_boundary()
                && !self.check(|kind| matches!(kind, TokenKind::Eof))
            {
                if self.check(|kind| matches!(kind, TokenKind::Class)) {
                    return Err(self.error_at(
                        self.peek().span,
                        "unsupported nested class declaration: only top-level class declarations are implemented",
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_trait_declaration_message())
                    );
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_enum_declaration_message())
                    );
                }
                if self.check_unsupported_class_modifier_declaration() {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_class_modifier_declaration_message(),
                    ));
                }
                statements.push(self.parse_statement()?);
            }
            Ok(statements)
        })();
        self.nested_statement_depth -= 1;
        result
    }

    fn consume_alternate_if_end(&mut self) -> CompileResult<()> {
        if !self.match_identifier("endif") {
            return Err(self.error_at(self.peek().span, "expected 'endif' after alternate if body"));
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after endif")
            .map(|_| ())
    }

    fn match_elseif(&mut self) -> Option<Span> {
        if self.check(|kind| {
            matches!(kind, TokenKind::ElseIf)
                || matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("elseif"))
        }) {
            return Some(self.advance().span);
        }
        None
    }

    fn match_else(&mut self) -> bool {
        self.match_token(|kind| {
            matches!(kind, TokenKind::Else)
                || matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("else"))
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

    fn parse_do_while(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        let body = self.parse_block_or_statement()?;
        self.consume_while_keyword("expected 'while' after do body")?;
        self.consume_keyword(TokenKind::LParen, "expected '(' after while")?;
        let condition = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after while condition")?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after do-while")?;

        Ok(Stmt::DoWhile {
            body,
            condition,
            span,
        })
    }

    fn parse_for(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after for")?;

        let initializer = self.parse_optional_for_action(TokenKind::Semicolon)?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after for initializer")?;

        let condition = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            None
        } else {
            let condition = self.parse_expression()?;
            self.reject_for_header_list_if_comma()?;
            Some(condition)
        };
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after for condition")?;

        let increment = self.parse_optional_for_action(TokenKind::RParen)?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after for increment")?;

        let body = self.parse_block_or_statement()?;
        Ok(Stmt::For {
            initializer,
            condition,
            increment,
            body,
            span,
        })
    }

    fn parse_optional_for_action(&mut self, end: TokenKind) -> CompileResult<Option<ForAction>> {
        if self.check(|kind| same_variant(kind, &end)) {
            return Ok(None);
        }

        let action = self.parse_for_action()?;
        self.reject_for_header_list_if_comma()?;
        Ok(Some(action))
    }

    fn parse_for_action(&mut self) -> CompileResult<ForAction> {
        let operator_span = self.peek().span;
        if let Some(op) = self.match_increment_decrement_operator() {
            let expr = self.parse_postfix()?;
            if matches!(expr, Expr::IncrementDecrement { .. }) {
                return Err(self.error_at(
                    operator_span,
                    unsupported_increment_decrement_expression_message(),
                ));
            }
            let target = self
                .increment_decrement_target_from_expr(expr)
                .map_err(|message| self.error_at(operator_span, message))?;
            Self::ensure_supported_increment_decrement_target(&target)
                .map_err(|message| self.error_at(target.span(), message))?;

            return Ok(ForAction::IncrementDecrement {
                target,
                op,
                span: operator_span,
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            let saved = self.current;
            let target = self.parse_assignment_target()?;
            if let Some(op) = self.match_increment_decrement_operator() {
                let span = target.span();
                Self::ensure_supported_increment_decrement_target(&target)
                    .map_err(|message| self.error_at(target.span(), message))?;

                return Ok(ForAction::IncrementDecrement { target, op, span });
            }
            if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                let expr = self.parse_expression()?;
                return Ok(ForAction::Assign { target, expr });
            }
            if let Some(op) = self.match_compound_assignment_operator() {
                let span = target.span();
                Self::ensure_supported_compound_assignment_target(&target)
                    .map_err(|message| self.error_at(target.span(), message))?;
                let expr = self.parse_expression()?;
                return Ok(ForAction::CompoundAssign {
                    target,
                    op,
                    expr,
                    span,
                });
            }
            self.current = saved;
        }

        let expr = self.parse_expression()?;
        Ok(ForAction::Expr { expr })
    }

    fn reject_for_header_list_if_comma(&self) -> CompileResult<()> {
        if self.check(|kind| matches!(kind, TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_for_header_list_message()));
        }
        Ok(())
    }

    fn parse_foreach(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after foreach")?;
        let iterable = self.parse_expression()?;
        self.consume_foreach_as()?;
        if self.match_token(|kind| matches!(kind, TokenKind::Ampersand)) {
            return Err(self.error_at(
                self.previous().span,
                unsupported_foreach_reference_message(),
            ));
        }
        if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_foreach_destructuring_message(),
            ));
        }
        let (first_variable, _) =
            self.consume_variable_with_span("expected foreach value variable")?;
        let (key, value) = if self.match_token(|kind| matches!(kind, TokenKind::FatArrow)) {
            if self.match_token(|kind| matches!(kind, TokenKind::Ampersand)) {
                return Err(self.error_at(
                    self.previous().span,
                    unsupported_foreach_reference_message(),
                ));
            }
            if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_foreach_destructuring_message(),
                ));
            }
            let (value, _) = self.consume_variable_with_span("expected foreach value variable")?;
            (Some(first_variable), value)
        } else {
            (None, first_variable)
        };
        self.consume_keyword(
            TokenKind::RParen,
            "expected ')' after foreach value variable",
        )?;
        let body = self.parse_block_or_statement()?;

        Ok(Stmt::Foreach {
            iterable,
            key,
            value,
            body,
            span,
        })
    }

    fn parse_switch(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after switch")?;
        let value = self.parse_expression()?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after switch expression")?;

        let body_kind = if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            SwitchBodyKind::Alternate
        } else {
            self.consume_keyword(TokenKind::LBrace, "expected switch body")?;
            SwitchBodyKind::Brace
        };

        let mut cases = Vec::new();
        let mut saw_default = false;

        while !self.check_switch_body_end(body_kind)
            && !self.check(|kind| matches!(kind, TokenKind::Eof))
        {
            let label = self.consume_switch_label(body_kind)?;
            match label {
                SwitchLabel::Case(label_span) => {
                    let condition = self.parse_expression()?;
                    self.consume_switch_case_separator("case")?;
                    let body = self.parse_switch_case_body(body_kind)?;
                    cases.push(SwitchCase {
                        condition: Some(condition),
                        body,
                        span: label_span,
                    });
                }
                SwitchLabel::Default(label_span) => {
                    if saw_default {
                        return Err(self
                            .error_at(label_span, "duplicate default label in switch statement"));
                    }
                    saw_default = true;
                    self.consume_switch_case_separator("default")?;
                    let body = self.parse_switch_case_body(body_kind)?;
                    cases.push(SwitchCase {
                        condition: None,
                        body,
                        span: label_span,
                    });
                }
            }
        }

        self.consume_switch_body_end(body_kind)?;
        Ok(Stmt::Switch { value, cases, span })
    }

    fn consume_switch_case_separator(&mut self, label: &str) -> CompileResult<()> {
        if self.match_token(|kind| matches!(kind, TokenKind::Colon | TokenKind::Semicolon)) {
            Ok(())
        } else {
            Err(self.error_at(
                self.peek().span,
                format!("expected ':' or ';' after switch {label}"),
            ))
        }
    }

    fn parse_switch_case_body(&mut self, body_kind: SwitchBodyKind) -> CompileResult<Vec<Stmt>> {
        self.nested_statement_depth += 1;
        let result = (|| {
            let mut statements = Vec::new();
            while !self.check_switch_body_end(body_kind)
                && !self.check(|kind| matches!(kind, TokenKind::Eof))
                && !self.check_switch_label()
            {
                if self.check(|kind| matches!(kind, TokenKind::Class)) {
                    return Err(self.error_at(
                        self.peek().span,
                        "unsupported nested class declaration: only top-level class declarations are implemented",
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_trait_declaration_message())
                    );
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_enum_declaration_message())
                    );
                }
                if self.check_unsupported_class_modifier_declaration() {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_class_modifier_declaration_message(),
                    ));
                }
                statements.push(self.parse_statement()?);
            }
            Ok(statements)
        })();
        self.nested_statement_depth -= 1;
        result
    }

    fn consume_switch_label(&mut self, body_kind: SwitchBodyKind) -> CompileResult<SwitchLabel> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("case") => {
                Ok(SwitchLabel::Case(token.span))
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("default") => {
                Ok(SwitchLabel::Default(token.span))
            }
            _ => Err(self.error_at(
                token.span,
                match body_kind {
                    SwitchBodyKind::Brace => "expected 'case' or 'default' in switch body",
                    SwitchBodyKind::Alternate => {
                        "expected 'case', 'default', or 'endswitch' in alternate switch body"
                    }
                },
            )),
        }
    }

    fn consume_switch_body_end(&mut self, body_kind: SwitchBodyKind) -> CompileResult<()> {
        match body_kind {
            SwitchBodyKind::Brace => {
                self.consume_keyword(TokenKind::RBrace, "expected '}' after switch body")?;
            }
            SwitchBodyKind::Alternate => {
                if !self.match_identifier("endswitch") {
                    return Err(self.error_at(
                        self.peek().span,
                        "expected 'endswitch' after alternate switch body",
                    ));
                }
                self.consume_keyword(TokenKind::Semicolon, "expected ';' after endswitch")?;
            }
        }
        Ok(())
    }

    fn parse_break(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        if !self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            return Err(self.error_at(token.span, unsupported_break_depth_message()));
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after break")?;
        Ok(Stmt::Break { span: token.span })
    }

    fn parse_continue(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        if !self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            return Err(self.error_at(token.span, unsupported_continue_depth_message()));
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after continue")?;
        Ok(Stmt::Continue { span: token.span })
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

    fn parse_static_local_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Static, "expected 'static'")?
            .span;
        let mut declarations = Vec::new();

        loop {
            let (name, name_span) =
                self.consume_variable_with_span("expected variable after static")?;
            let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                let expr = self.parse_expression()?;
                self.ensure_supported_default_expr(&expr)?;
                Some(expr)
            } else {
                None
            };
            declarations.push(StaticLocalDeclarator {
                name,
                default,
                span: name_span,
            });

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }

        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after static local declaration",
        )?;
        Ok(Stmt::StaticLocal { declarations, span })
    }

    fn parse_unset(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after unset")?;

        let mut targets = Vec::new();
        loop {
            targets.push(self.parse_unset_target()?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
        }

        self.consume_keyword(TokenKind::RParen, "expected ')' after unset operand")?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after unset")?;

        if targets.len() == 1 {
            return match targets.pop().expect("single unset target exists") {
                UnsetTarget::Variable { name, .. } => Ok(Stmt::UnsetVariable { name, span }),
                UnsetTarget::ArrayIndex { name, index, .. } => {
                    Ok(Stmt::UnsetArrayIndex { name, index, span })
                }
                UnsetTarget::StaticProperty {
                    class_name,
                    property,
                    ..
                } => Ok(Stmt::UnsetStaticProperty {
                    class_name,
                    property,
                    span,
                }),
                UnsetTarget::SelfStaticProperty { property, .. } => {
                    Ok(Stmt::UnsetSelfStaticProperty { property, span })
                }
                UnsetTarget::ParentStaticProperty { property, .. } => {
                    Ok(Stmt::UnsetParentStaticProperty { property, span })
                }
                UnsetTarget::LateStaticProperty { property, .. } => {
                    Ok(Stmt::UnsetLateStaticProperty { property, span })
                }
            };
        }

        Ok(Stmt::UnsetMany { targets, span })
    }

    fn parse_unset_target(&mut self) -> CompileResult<UnsetTarget> {
        let token = self.advance().clone();
        let (name, target_span) = match token.kind {
            TokenKind::Variable(name) => (name, token.span),
            TokenKind::Identifier(name)
                if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) =>
            {
                return self.parse_static_property_unset_target(Some(name), token.span);
            }
            TokenKind::Static if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) => {
                return self
                    .parse_static_property_unset_target(Some("static".to_string()), token.span);
            }
            _ => return Err(self.error_at(token.span, unsupported_unset_message())),
        };

        if !self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            if self.check(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_object_property_unset_message(),
                ));
            }
            if self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
                return Ok(UnsetTarget::Variable {
                    name,
                    span: target_span,
                });
            }
            return Err(self.error_at(target_span, unsupported_unset_message()));
        }
        let bracket_span = self.previous().span;
        if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
            return Err(self.error_at(bracket_span, unsupported_unset_message()));
        }

        let index = self.parse_expression()?;
        self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;

        if self.check(|kind| matches!(kind, TokenKind::LBracket | TokenKind::ObjectOperator)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }
        if !self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }

        Ok(UnsetTarget::ArrayIndex {
            name,
            index,
            span: target_span,
        })
    }

    fn parse_static_property_unset_target(
        &mut self,
        receiver: Option<String>,
        target_span: Span,
    ) -> CompileResult<UnsetTarget> {
        self.consume_keyword(TokenKind::DoubleColon, "expected '::' after class name")?;
        let operator_span = self.previous().span;
        let member = self.advance().clone();
        let property = match member.kind {
            TokenKind::Variable(property) => property,
            _ => return Err(self.error_at(member.span, unsupported_unset_message())),
        };

        if !self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }

        match receiver.as_deref() {
            Some(receiver) if receiver.eq_ignore_ascii_case("self") => {
                Ok(UnsetTarget::SelfStaticProperty {
                    property,
                    span: operator_span,
                })
            }
            Some(receiver) if receiver.eq_ignore_ascii_case("parent") => {
                Ok(UnsetTarget::ParentStaticProperty {
                    property,
                    span: operator_span,
                })
            }
            Some(receiver) if receiver.eq_ignore_ascii_case("static") => {
                Ok(UnsetTarget::LateStaticProperty {
                    property,
                    span: operator_span,
                })
            }
            Some(class_name) => Ok(UnsetTarget::StaticProperty {
                class_name: class_name.to_string(),
                property,
                span: target_span,
            }),
            None => Err(self.error_at(target_span, unsupported_unset_message())),
        }
    }

    fn parse_assignment_or_expression_statement(&mut self) -> CompileResult<Stmt> {
        if let Some(stmt) = self.try_parse_prefix_increment_decrement_statement()? {
            return Ok(stmt);
        }

        if let Some(stmt) = self.try_parse_assignment_statement()? {
            return Ok(stmt);
        }

        self.parse_expression_statement()
    }

    fn try_parse_prefix_increment_decrement_statement(&mut self) -> CompileResult<Option<Stmt>> {
        let saved = self.current;
        let operator_span = self.peek().span;
        let Some(op) = self.match_increment_decrement_operator() else {
            return Ok(None);
        };

        let expr = self.parse_postfix()?;
        if matches!(expr, Expr::IncrementDecrement { .. }) {
            return Err(self.error_at(
                operator_span,
                unsupported_increment_decrement_expression_message(),
            ));
        }
        let target = self
            .increment_decrement_target_from_expr(expr)
            .map_err(|message| self.error_at(operator_span, message))?;
        if !self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            if matches!(
                target,
                AssignTarget::Variable { .. }
                    | AssignTarget::ArrayIndex { index: Some(_), .. }
                    | AssignTarget::Property { .. }
                    | AssignTarget::StaticProperty { .. }
                    | AssignTarget::SelfStaticProperty { .. }
                    | AssignTarget::ParentStaticProperty { .. }
                    | AssignTarget::LateStaticProperty { .. }
            ) {
                self.current = saved;
                return Ok(None);
            }
            return Err(self.error_at(
                target.span(),
                unsupported_increment_decrement_target_message(),
            ));
        }
        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after increment/decrement",
        )?;

        Self::ensure_supported_increment_decrement_target(&target)
            .map_err(|message| self.error_at(target.span(), message))?;

        Ok(Some(Stmt::IncrementDecrement {
            target,
            op,
            span: operator_span,
        }))
    }

    fn try_parse_assignment_statement(&mut self) -> CompileResult<Option<Stmt>> {
        if !self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            return Ok(None);
        }

        let saved = self.current;
        let target = self.parse_assignment_target()?;
        if !self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
            if let Some(op) = self.match_increment_decrement_operator() {
                if !self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                    if matches!(
                        target,
                        AssignTarget::Variable { .. }
                            | AssignTarget::ArrayIndex { index: Some(_), .. }
                            | AssignTarget::Property { .. }
                    ) {
                        self.current = saved;
                        return Ok(None);
                    }
                    return Err(self.error_at(
                        target.span(),
                        unsupported_increment_decrement_target_message(),
                    ));
                }
                self.consume_keyword(
                    TokenKind::Semicolon,
                    "expected ';' after increment/decrement",
                )?;

                let span = target.span();
                Self::ensure_supported_increment_decrement_target(&target)
                    .map_err(|message| self.error_at(target.span(), message))?;

                return Ok(Some(Stmt::IncrementDecrement { target, op, span }));
            }
            if let Some(op) = self.match_compound_assignment_operator() {
                let span = target.span();
                Self::ensure_supported_compound_assignment_target(&target)
                    .map_err(|message| self.error_at(target.span(), message))?;
                let expr = self.parse_assignment_expression()?;
                if self.check_low_precedence_logical_operator() {
                    self.current = saved;
                    return Ok(None);
                }
                self.consume_keyword(
                    TokenKind::Semicolon,
                    "expected ';' after compound assignment",
                )?;
                return Ok(Some(Stmt::CompoundAssign {
                    target,
                    op,
                    expr,
                    span,
                }));
            }
            if self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
                && matches!(self.peek_next().kind, TokenKind::Equal)
            {
                let operator_span = self.advance().span;
                self.advance();
                let span = target.span();
                match &target {
                    AssignTarget::Variable { .. }
                    | AssignTarget::ArrayIndex { index: Some(_), .. }
                    | AssignTarget::Property { .. }
                    | AssignTarget::StaticProperty { .. }
                    | AssignTarget::SelfStaticProperty { .. }
                    | AssignTarget::ParentStaticProperty { .. }
                    | AssignTarget::LateStaticProperty { .. } => {}
                    AssignTarget::ArrayIndex { index: None, .. } => {
                        return Err(self.error_at(
                            operator_span,
                            unsupported_null_coalescing_assignment_message(),
                        ));
                    }
                }
                let expr = self.parse_assignment_expression()?;
                if self.check_low_precedence_logical_operator() {
                    self.current = saved;
                    return Ok(None);
                }
                self.consume_keyword(
                    TokenKind::Semicolon,
                    "expected ';' after null coalescing assignment",
                )?;
                return Ok(Some(Stmt::NullCoalesceAssign { target, expr, span }));
            }
            self.current = saved;
            return Ok(None);
        }

        let span = target.span();
        let expr = self.parse_assignment_expression()?;
        if self.check_low_precedence_logical_operator() {
            self.current = saved;
            return Ok(None);
        }
        if Self::expr_contains_unsupported_assignment_rhs(&expr) {
            return Err(self.error_at(
                expr.span(),
                unsupported_chained_assignment_expression_message(),
            ));
        }
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
                return Ok(AssignTarget::Variable { name, span });
            }
            return Ok(AssignTarget::Property {
                object: name,
                property,
                span,
            });
        }

        Ok(AssignTarget::Variable { name, span })
    }

    fn ensure_supported_compound_assignment_target(
        target: &AssignTarget,
    ) -> Result<(), &'static str> {
        match target {
            AssignTarget::Variable { .. }
            | AssignTarget::ArrayIndex { index: Some(_), .. }
            | AssignTarget::Property { .. }
            | AssignTarget::StaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. } => Ok(()),
            AssignTarget::ArrayIndex { index: None, .. } => {
                Err(unsupported_compound_assignment_target_message())
            }
        }
    }

    fn ensure_supported_increment_decrement_target(
        target: &AssignTarget,
    ) -> Result<(), &'static str> {
        match target {
            AssignTarget::Variable { .. }
            | AssignTarget::ArrayIndex { index: Some(_), .. }
            | AssignTarget::Property { .. }
            | AssignTarget::StaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. } => Ok(()),
            AssignTarget::ArrayIndex { index: None, .. } => {
                Err(unsupported_increment_decrement_target_message())
            }
        }
    }

    fn increment_decrement_target_from_expr(
        &self,
        expr: Expr,
    ) -> Result<AssignTarget, &'static str> {
        match expr {
            Expr::Variable(name, span) => Ok(AssignTarget::Variable { name, span }),
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: Some(*index),
                    span,
                }),
                _ => Err(unsupported_increment_decrement_target_message()),
            },
            Expr::Property {
                target,
                property,
                span,
            } => match *target {
                Expr::Variable(object, _) => Ok(AssignTarget::Property {
                    object,
                    property,
                    span,
                }),
                _ => Err(unsupported_increment_decrement_target_message()),
            },
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            _ => Err(unsupported_increment_decrement_target_message()),
        }
    }

    fn compound_assignment_target_from_expr(
        &self,
        expr: Expr,
    ) -> Result<AssignTarget, &'static str> {
        match expr {
            Expr::Variable(name, span) => Ok(AssignTarget::Variable { name, span }),
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: Some(*index),
                    span,
                }),
                _ => Err(unsupported_compound_assignment_target_message()),
            },
            Expr::AppendIndex { .. } => Err(unsupported_compound_assignment_target_message()),
            Expr::Property {
                target,
                property,
                span,
            } => match *target {
                Expr::Variable(object, _) => Ok(AssignTarget::Property {
                    object,
                    property,
                    span,
                }),
                _ => Err(unsupported_compound_assignment_target_message()),
            },
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            _ => Err(unsupported_compound_assignment_target_message()),
        }
    }

    fn assignment_expression_target_from_expr(
        &self,
        expr: Expr,
    ) -> Result<AssignTarget, &'static str> {
        match expr {
            Expr::Variable(name, span) => Ok(AssignTarget::Variable { name, span }),
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: Some(*index),
                    span,
                }),
                _ => Err(unsupported_assignment_expression_target_message()),
            },
            Expr::AppendIndex { target, span } => match *target {
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: None,
                    span,
                }),
                _ => Err(unsupported_assignment_expression_target_message()),
            },
            Expr::Property {
                target,
                property,
                span,
            } => match *target {
                Expr::Variable(object, _) => Ok(AssignTarget::Property {
                    object,
                    property,
                    span,
                }),
                _ => Err(unsupported_assignment_expression_target_message()),
            },
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            Expr::Array { .. } => Err(unsupported_array_destructuring_assignment_message()),
            _ => Err(unsupported_assignment_expression_target_message()),
        }
    }

    fn null_coalescing_assignment_expression_target_from_expr(
        &self,
        expr: Expr,
    ) -> Result<AssignTarget, &'static str> {
        match expr {
            Expr::Variable(name, span) => Ok(AssignTarget::Variable { name, span }),
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: Some(*index),
                    span,
                }),
                _ => Err(unsupported_null_coalescing_assignment_message()),
            },
            Expr::AppendIndex { .. } => Err(unsupported_null_coalescing_assignment_message()),
            Expr::Property {
                target,
                property,
                span,
            } => match *target {
                Expr::Variable(object, _) => Ok(AssignTarget::Property {
                    object,
                    property,
                    span,
                }),
                _ => Err(unsupported_null_coalescing_assignment_message()),
            },
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            _ => Err(unsupported_null_coalescing_assignment_message()),
        }
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

        Ok(vec![self.parse_nested_statement()?])
    }

    fn parse_required_block(&mut self, message: &str) -> CompileResult<Vec<Stmt>> {
        self.consume_keyword(TokenKind::LBrace, message)?;
        self.parse_block_after_open()
    }

    fn parse_block_after_open(&mut self) -> CompileResult<Vec<Stmt>> {
        self.nested_statement_depth += 1;
        let result = (|| {
            let mut statements = Vec::new();
            while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
                if self.check(|kind| matches!(kind, TokenKind::Class)) {
                    return Err(self.error_at(
                        self.peek().span,
                        "unsupported nested class declaration: only top-level class declarations are implemented",
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_trait_declaration_message())
                    );
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(
                        self.error_at(self.peek().span, unsupported_enum_declaration_message())
                    );
                }
                if self.check_unsupported_class_modifier_declaration() {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_class_modifier_declaration_message(),
                    ));
                }
                statements.push(self.parse_statement()?);
            }
            self.consume_keyword(TokenKind::RBrace, "expected '}' after block")?;
            Ok(statements)
        })();
        self.nested_statement_depth -= 1;
        result
    }

    fn parse_nested_statement(&mut self) -> CompileResult<Stmt> {
        self.nested_statement_depth += 1;
        let result = self.parse_statement();
        self.nested_statement_depth -= 1;
        result
    }

    fn parse_expression(&mut self) -> CompileResult<Expr> {
        let expr = self.parse_low_precedence_logical_or()?;
        self.reject_assignment_expression_operator()?;
        Ok(expr)
    }

    fn parse_low_precedence_logical_or(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_low_precedence_logical_xor()?;
        while self.match_identifier("or") {
            let right = self.parse_low_precedence_logical_xor()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalOr,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_low_precedence_logical_xor(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_low_precedence_logical_and()?;
        while self.match_identifier("xor") {
            let right = self.parse_low_precedence_logical_and()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalXor,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_low_precedence_logical_and(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_assignment_expression()?;
        while self.match_identifier("and") {
            let right = self.parse_assignment_expression()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalAnd,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_assignment_expression(&mut self) -> CompileResult<Expr> {
        self.parse_assignment_expression_with_ternary(true)
    }

    fn parse_assignment_expression_without_unparenthesized_ternary(
        &mut self,
    ) -> CompileResult<Expr> {
        self.parse_assignment_expression_with_ternary(false)
    }

    fn parse_assignment_expression_with_ternary(
        &mut self,
        allow_ternary: bool,
    ) -> CompileResult<Expr> {
        let expr = self.parse_non_assignment_expression_with_ternary(allow_ternary)?;
        if let Some(op) = self.match_compound_assignment_operator() {
            let operator_span = self.previous().span;
            let target = self
                .compound_assignment_target_from_expr(expr)
                .map_err(|message| self.error_at(operator_span, message))?;
            let span = target.span();

            let value = self.parse_non_assignment_expression_with_ternary(allow_ternary)?;
            if let Some(span) = Self::find_append_index_span(&value) {
                return Err(self.error_at(
                    span,
                    "cannot use [] for reading; append syntax is only supported in assignments",
                ));
            }
            if Self::expr_contains_assignment(&value)
                || self.check(|kind| matches!(kind, TokenKind::Equal))
                || (self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
                    && matches!(self.peek_next().kind, TokenKind::Equal))
                || self.check_compound_assignment_operator()
            {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_chained_assignment_expression_message(),
                ));
            }

            return Ok(Expr::CompoundAssign {
                target: Box::new(target),
                op,
                expr: Box::new(value),
                span,
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
            && matches!(self.peek_next().kind, TokenKind::Equal)
        {
            let operator_span = self.advance().span;
            self.advance();
            let target = self
                .null_coalescing_assignment_expression_target_from_expr(expr)
                .map_err(|message| self.error_at(operator_span, message))?;
            let span = target.span();

            let value = self.parse_non_assignment_expression_with_ternary(allow_ternary)?;
            if let Some(span) = Self::find_append_index_span(&value) {
                return Err(self.error_at(
                    span,
                    "cannot use [] for reading; append syntax is only supported in assignments",
                ));
            }
            if Self::expr_contains_assignment(&value)
                || self.check(|kind| matches!(kind, TokenKind::Equal))
                || (self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
                    && matches!(self.peek_next().kind, TokenKind::Equal))
                || self.check_compound_assignment_operator()
            {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_chained_assignment_expression_message(),
                ));
            }

            return Ok(Expr::NullCoalesceAssign {
                target: Box::new(target),
                expr: Box::new(value),
                span,
            });
        }

        if !self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
            if let Some(span) = Self::find_append_index_span(&expr) {
                return Err(self.error_at(
                    span,
                    "cannot use [] for reading; append syntax is only supported in assignments",
                ));
            }
            return Ok(expr);
        }

        let operator_span = self.previous().span;
        let target = self
            .assignment_expression_target_from_expr(expr)
            .map_err(|message| self.error_at(operator_span, message))?;
        let span = target.span();

        let value = self.parse_assignment_expression_with_ternary(allow_ternary)?;
        if let Some(span) = Self::find_append_index_span(&value) {
            return Err(self.error_at(
                span,
                "cannot use [] for reading; append syntax is only supported in assignments",
            ));
        }
        if matches!(target, AssignTarget::ArrayIndex { index: None, .. })
            && Self::expr_contains_assignment(&value)
        {
            return Err(self.error_at(
                value.span(),
                unsupported_chained_assignment_expression_message(),
            ));
        }
        if Self::expr_contains_unsupported_assignment_rhs(&value)
            || self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
            || self.check_compound_assignment_operator()
        {
            return Err(self.error_at(
                value.span(),
                unsupported_chained_assignment_expression_message(),
            ));
        }

        Ok(Expr::Assign {
            target: Box::new(target),
            expr: Box::new(value),
            span,
        })
    }

    fn parse_non_assignment_expression_with_ternary(
        &mut self,
        allow_ternary: bool,
    ) -> CompileResult<Expr> {
        let mut expr = self.parse_symbolic_logical_or()?;
        if self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
            && matches!(self.peek_next().kind, TokenKind::Equal)
        {
            return Ok(expr);
        }
        if self.match_token(|kind| matches!(kind, TokenKind::QuestionQuestion)) {
            let operator_span = self.previous().span;
            if self.check(|kind| matches!(kind, TokenKind::Equal)) {
                return Err(
                    self.error_at(operator_span, unsupported_assignment_expression_message())
                );
            }
            let right = self.parse_symbolic_logical_or()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::NullCoalesce,
                right: Box::new(right),
                span,
            };
            if self.check(|kind| matches!(kind, TokenKind::QuestionQuestion)) {
                if matches!(self.peek_next().kind, TokenKind::Equal) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_assignment_expression_message(),
                    ));
                }
                return Err(self.error_at(self.peek().span, unsupported_null_coalescing_message()));
            }
        }
        if self.match_token(|kind| matches!(kind, TokenKind::Question)) {
            let question_span = self.previous().span;
            if !allow_ternary {
                return Err(self.error_at(question_span, unsupported_nested_ternary_message()));
            }
            if self.check(|kind| matches!(kind, TokenKind::Colon)) {
                self.advance();
                let if_false =
                    self.parse_assignment_expression_without_unparenthesized_ternary()?;
                let span = expr.span();
                expr = Expr::ShortTernary {
                    condition: Box::new(expr),
                    if_false: Box::new(if_false),
                    span,
                };
                return Ok(expr);
            }

            let if_true = self.parse_assignment_expression_without_unparenthesized_ternary()?;
            self.consume_keyword(
                TokenKind::Colon,
                "expected ':' after ternary true expression",
            )?;
            let if_false = self.parse_assignment_expression_without_unparenthesized_ternary()?;
            let span = expr.span();
            expr = Expr::Ternary {
                condition: Box::new(expr),
                if_true: Box::new(if_true),
                if_false: Box::new(if_false),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_symbolic_logical_or(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_symbolic_logical_and()?;
        while self.match_token(|kind| matches!(kind, TokenKind::PipePipe)) {
            let right = self.parse_symbolic_logical_and()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalOr,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_symbolic_logical_and(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_bitwise_or()?;
        while self.match_token(|kind| matches!(kind, TokenKind::AmpAmp)) {
            let right = self.parse_bitwise_or()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::LogicalAnd,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_bitwise_xor()?;
        while !self.check_compound_assignment_operator()
            && self.match_token(|kind| matches!(kind, TokenKind::Pipe))
        {
            let right = self.parse_bitwise_xor()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseOr,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_bitwise_and()?;
        while !self.check_compound_assignment_operator()
            && self.match_token(|kind| matches!(kind, TokenKind::Caret))
        {
            let right = self.parse_bitwise_and()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseXor,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_equality()?;
        while !self.check_compound_assignment_operator()
            && self.match_token(|kind| matches!(kind, TokenKind::Ampersand))
        {
            let right = self.parse_equality()?;
            let span = expr.span();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::BitwiseAnd,
                right: Box::new(right),
                span,
            };
        }
        Ok(expr)
    }

    fn reject_assignment_expression_operator(&self) -> CompileResult<()> {
        if self.check(|kind| matches!(kind, TokenKind::Equal)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_assignment_expression_message(),
            ));
        }
        if self.check_compound_assignment_operator() {
            return Err(self.error_at(
                self.peek().span,
                unsupported_assignment_expression_message(),
            ));
        }
        if self.check(|kind| matches!(kind, TokenKind::QuestionQuestion))
            && matches!(self.peek_next().kind, TokenKind::Equal)
        {
            return Err(self.error_at(
                self.peek().span,
                unsupported_assignment_expression_message(),
            ));
        }
        Ok(())
    }

    fn parse_equality(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = if self.match_token(|kind| matches!(kind, TokenKind::EqualEqual)) {
                BinaryOp::Eq
            } else if self.match_token(|kind| matches!(kind, TokenKind::BangEqual)) {
                BinaryOp::Ne
            } else if self.match_token(|kind| matches!(kind, TokenKind::StrictEqual)) {
                BinaryOp::StrictEq
            } else if self.match_token(|kind| matches!(kind, TokenKind::StrictBangEqual)) {
                BinaryOp::StrictNe
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
        if self.check_instanceof_operator() {
            self.advance();
            let class_name = self.consume_instanceof_class_name()?;
            let span = expr.span();
            expr = Expr::InstanceOf {
                expr: Box::new(expr),
                class_name,
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
        let mut expr = self.parse_shift()?;
        loop {
            if self.check_compound_assignment_operator() {
                break;
            }
            if !self.match_token(|kind| matches!(kind, TokenKind::Dot)) {
                break;
            }
            let right = self.parse_shift()?;
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

    fn parse_shift(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_additive()?;
        loop {
            if self.check_compound_assignment_operator() {
                break;
            }
            let op = if self.match_token(|kind| matches!(kind, TokenKind::LeftShift)) {
                BinaryOp::ShiftLeft
            } else if self.match_token(|kind| matches!(kind, TokenKind::RightShift)) {
                BinaryOp::ShiftRight
            } else {
                break;
            };
            let right = self.parse_additive()?;
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

    fn parse_additive(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            if self.check_compound_assignment_operator() {
                break;
            }
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
            if self.check_compound_assignment_operator() {
                break;
            }
            if self.check(|kind| matches!(kind, TokenKind::StarStar)) {
                return Err(self.error_at(self.peek().span, unsupported_exponentiation_message()));
            }
            let op = if self.match_token(|kind| matches!(kind, TokenKind::Star)) {
                BinaryOp::Mul
            } else if self.match_token(|kind| matches!(kind, TokenKind::Slash)) {
                BinaryOp::Div
            } else if self.match_token(|kind| matches!(kind, TokenKind::Percent)) {
                BinaryOp::Mod
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
        let operator_span = self.peek().span;
        if let Some(cast) = self.current_cast_kind()? {
            let span = self.advance().span;
            self.advance();
            self.consume_keyword(TokenKind::RParen, "expected ')' after cast type")?;
            let expr = self.parse_unary()?;
            return Ok(Expr::Cast {
                kind: cast,
                expr: Box::new(expr),
                span,
            });
        }

        if let Some(op) = self.match_increment_decrement_operator() {
            let expr = self.parse_postfix()?;
            if matches!(expr, Expr::IncrementDecrement { .. }) {
                return Err(self.error_at(
                    operator_span,
                    unsupported_increment_decrement_expression_message(),
                ));
            }
            let target = self
                .increment_decrement_target_from_expr(expr)
                .map_err(|message| self.error_at(operator_span, message))?;
            Self::ensure_supported_increment_decrement_target(&target)
                .map_err(|message| self.error_at(target.span(), message))?;

            if self.check_increment_decrement_operator() {
                return Err(self.error_at(
                    operator_span,
                    unsupported_increment_decrement_expression_message(),
                ));
            }

            return Ok(Expr::IncrementDecrement {
                target: Box::new(target),
                op,
                position: IncrementDecrementPosition::Pre,
                span: operator_span,
            });
        }

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

        if self.match_token(|kind| matches!(kind, TokenKind::Tilde)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::BitwiseNot,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_postfix()
    }

    fn current_cast_kind(&self) -> CompileResult<Option<CastKind>> {
        if !matches!(self.peek().kind, TokenKind::LParen) {
            return Ok(None);
        }
        let TokenKind::Identifier(name) = &self.peek_next().kind else {
            return Ok(None);
        };
        if !matches!(self.peek_n(2).kind, TokenKind::RParen) {
            return Ok(None);
        }

        match name.to_ascii_lowercase().as_str() {
            "string" => Ok(Some(CastKind::String)),
            "int" | "integer" | "bool" | "boolean" | "float" | "double" | "real" | "array"
            | "object" | "unset" | "binary" => Err(self.error_at(
                self.peek().span,
                "unsupported cast expression: only (string) casts are implemented",
            )),
            _ => Ok(None),
        }
    }

    fn parse_postfix(&mut self) -> CompileResult<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                    self.advance();
                    let span = expr.span();
                    expr = Expr::AppendIndex {
                        target: Box::new(expr),
                        span,
                    };
                    continue;
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
                let member = self.consume_object_property_name(operator_span)?;
                if self.match_token(|kind| matches!(kind, TokenKind::LParen)) {
                    let span = expr.span();
                    let args = self.parse_call_arguments_after_open()?;
                    expr = Expr::MethodCall {
                        target: Box::new(expr),
                        method: member,
                        args,
                        span,
                    };
                    continue;
                }

                let span = expr.span();
                expr = Expr::Property {
                    target: Box::new(expr),
                    property: member,
                    span,
                };
                continue;
            }

            if self.match_token(|kind| matches!(kind, TokenKind::DoubleColon)) {
                let operator_span = self.previous().span;
                let member = self.peek().clone();
                match member.kind {
                    TokenKind::Identifier(method)
                        if matches!(self.peek_next().kind, TokenKind::LParen) =>
                    {
                        self.advance();
                        self.consume_keyword(TokenKind::LParen, "expected '(' after method name")?;
                        let span = expr.span();
                        let args = self.parse_call_arguments_after_open()?;
                        expr = Expr::ObjectStaticMethodCall {
                            target: Box::new(expr),
                            method,
                            args,
                            span,
                        };
                        continue;
                    }
                    TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                        return Err(self.error_at(
                            operator_span,
                            "unsupported object static class-name constant: object receiver ::class is not implemented",
                        ));
                    }
                    TokenKind::Identifier(_) | TokenKind::Class => {
                        return Err(self.error_at(
                            operator_span,
                            "unsupported object static class constant access: object receiver class constants are not implemented",
                        ));
                    }
                    TokenKind::Variable(_) => {
                        return Err(self.error_at(
                            operator_span,
                            "unsupported object static property access: object receiver static properties are not implemented",
                        ));
                    }
                    _ => {
                        return Err(self.error_at(
                            operator_span,
                            format!(
                                "expected object static member name after '::', found {}",
                                token_name(&member.kind)
                            ),
                        ));
                    }
                }
            }

            if self.check_increment_decrement_operator() {
                let span = expr.span();
                let target = self
                    .increment_decrement_target_from_expr(expr)
                    .map_err(|message| self.error_at(span, message))?;
                let op = self
                    .match_increment_decrement_operator()
                    .expect("caller checked increment/decrement operator");
                let span = target.span();
                expr = Expr::IncrementDecrement {
                    target: Box::new(target),
                    op,
                    position: IncrementDecrementPosition::Post,
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
            TokenKind::LBracket => {
                self.parse_array_literal(token.span, ArrayLiteralDelimiter::Short)
            }
            TokenKind::Class => {
                Err(self.error_at(token.span, unsupported_class_expression_message()))
            }
            TokenKind::New => self.parse_new_expression(token.span),
            TokenKind::Clone => Err(self.error_at(token.span, unsupported_clone_message())),
            TokenKind::Instanceof => {
                Err(self.error_at(token.span, unsupported_instanceof_message()))
            }
            TokenKind::Function => self.parse_closure_expression(token.span),
            TokenKind::Fn => Err(self.error_at(
                token.span,
                "unsupported closure: arrow functions are not implemented",
            )),
            TokenKind::Eval => Err(self.error_at(token.span, unsupported_eval_message())),
            TokenKind::Do => {
                Err(self.error_at(token.span, unsupported_do_while_expression_message()))
            }
            TokenKind::Foreach => {
                Err(self.error_at(token.span, unsupported_foreach_expression_message()))
            }
            TokenKind::For => Err(self.error_at(token.span, unsupported_for_expression_message())),
            TokenKind::Switch => {
                Err(self.error_at(token.span, unsupported_switch_expression_message()))
            }
            TokenKind::Match => {
                Err(self.error_at(token.span, unsupported_match_expression_message()))
            }
            TokenKind::Break => {
                Err(self.error_at(token.span, unsupported_break_expression_message()))
            }
            TokenKind::Continue => {
                Err(self.error_at(token.span, unsupported_continue_expression_message()))
            }
            TokenKind::Throw => Err(self.error_at(token.span, unsupported_throw_message())),
            TokenKind::Try | TokenKind::Catch | TokenKind::Finally => {
                Err(self.error_at(token.span, unsupported_try_catch_finally_message()))
            }
            TokenKind::Include => {
                Err(self.error_at(token.span, unsupported_include_expression_message()))
            }
            TokenKind::IncludeOnce => {
                Err(self.error_at(token.span, unsupported_include_once_expression_message()))
            }
            TokenKind::Require => {
                Err(self.error_at(token.span, unsupported_require_expression_message()))
            }
            TokenKind::RequireOnce => {
                Err(self.error_at(token.span, unsupported_require_once_expression_message()))
            }
            TokenKind::Ampersand => Err(self.error_at(
                token.span,
                "unsupported reference expression: references are not implemented",
            )),
            TokenKind::Identifier(name) => {
                if let Some(magic_name) = magic_constant_name(&name) {
                    if magic_name == "__LINE__" {
                        return Ok(Expr::MagicLine { span: token.span });
                    }
                    if magic_name == "__FILE__" {
                        return Ok(Expr::MagicFile { span: token.span });
                    }
                    if magic_name == "__DIR__" {
                        return Ok(Expr::MagicDir { span: token.span });
                    }
                    if magic_name == "__FUNCTION__" {
                        return Ok(Expr::MagicFunction { span: token.span });
                    }
                    return Err(
                        self.error_at(token.span, unsupported_magic_constant_message(magic_name))
                    );
                }
                if name.eq_ignore_ascii_case("do") {
                    return Err(
                        self.error_at(token.span, unsupported_do_while_expression_message())
                    );
                }
                if name.eq_ignore_ascii_case("foreach") {
                    return Err(self.error_at(token.span, unsupported_foreach_expression_message()));
                }
                if name.eq_ignore_ascii_case("for") {
                    return Err(self.error_at(token.span, unsupported_for_expression_message()));
                }
                if name.eq_ignore_ascii_case("switch") {
                    return Err(self.error_at(token.span, unsupported_switch_expression_message()));
                }
                if name.eq_ignore_ascii_case("match") {
                    return Err(self.error_at(token.span, unsupported_match_expression_message()));
                }
                if name.eq_ignore_ascii_case("break") {
                    return Err(self.error_at(token.span, unsupported_break_expression_message()));
                }
                if name.eq_ignore_ascii_case("continue") {
                    return Err(
                        self.error_at(token.span, unsupported_continue_expression_message())
                    );
                }
                if name.eq_ignore_ascii_case("throw") {
                    return Err(self.error_at(token.span, unsupported_throw_message()));
                }
                if matches!(
                    name.to_ascii_lowercase().as_str(),
                    "try" | "catch" | "finally"
                ) {
                    return Err(self.error_at(token.span, unsupported_try_catch_finally_message()));
                }
                if name.eq_ignore_ascii_case("yield") {
                    return Err(self.error_at(token.span, unsupported_yield_message()));
                }
                if name.eq_ignore_ascii_case("goto") {
                    return Err(self.error_at(token.span, unsupported_goto_message()));
                }
                if name.eq_ignore_ascii_case("clone") {
                    return Err(self.error_at(token.span, unsupported_clone_message()));
                }
                if name.eq_ignore_ascii_case("instanceof") {
                    return Err(self.error_at(token.span, unsupported_instanceof_message()));
                }
                if name.eq_ignore_ascii_case("array")
                    && self.check(|kind| matches!(kind, TokenKind::LParen))
                {
                    self.consume_keyword(TokenKind::LParen, "expected '(' after array")?;
                    return self.parse_array_literal(token.span, ArrayLiteralDelimiter::Long);
                }
                if name.eq_ignore_ascii_case("list")
                    && self.check(|kind| matches!(kind, TokenKind::LParen))
                {
                    return Err(self.error_at(
                        token.span,
                        unsupported_array_destructuring_assignment_message(),
                    ));
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
                    return self.reject_unsupported_static_member_access(Some(&name));
                }
                if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    return Ok(Expr::GlobalConstant {
                        name,
                        span: token.span,
                    });
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
                self.reject_unsupported_static_member_access(Some("static"))
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

    fn parse_closure_expression(&mut self, span: Span) -> CompileResult<Expr> {
        if self.check(|kind| matches!(kind, TokenKind::Ampersand)) {
            let span = self.advance().span;
            return Err(self.error_at(
                span,
                "unsupported reference return: returning closures by reference is not implemented",
            ));
        }

        self.consume_keyword(TokenKind::LParen, "expected '(' after function")?;
        let params = self.parse_function_params_after_open()?;

        let mut captures = Vec::new();
        if self.match_token(|kind| matches!(kind, TokenKind::Use)) {
            self.consume_keyword(TokenKind::LParen, "expected '(' after closure use")?;
            if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
                loop {
                    let by_reference =
                        self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
                    let (name, capture_span) =
                        self.consume_variable_with_span("expected closure capture variable")?;
                    captures.push(ClosureCapture {
                        name,
                        by_reference,
                        span: capture_span,
                    });

                    if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                        break;
                    }
                    if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                        break;
                    }
                }
            }
            self.consume_keyword(TokenKind::RParen, "expected ')' after closure use list")?;
        }

        let return_type = if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            Some(self.parse_type_decl(unsupported_return_type_message())?)
        } else {
            None
        };

        self.function_body_depth += 1;
        let body = self.parse_required_block("expected closure body");
        self.function_body_depth -= 1;
        let body = body?;

        Ok(Expr::Closure {
            params,
            captures,
            return_type,
            body,
            span,
        })
    }

    fn reject_unsupported_static_member_access(
        &mut self,
        receiver: Option<&str>,
    ) -> CompileResult<Expr> {
        let operator_span = self
            .consume_keyword(TokenKind::DoubleColon, "expected '::'")?
            .span;
        if receiver.is_some_and(|receiver| receiver.eq_ignore_ascii_case("parent")) {
            let member = self.peek().clone();
            return match member.kind {
                TokenKind::Variable(property) => {
                    self.advance();
                    Ok(Expr::ParentStaticProperty {
                        property,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                    self.advance();
                    Ok(Expr::ParentClassNameConstant {
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(method)
                    if matches!(self.peek_next().kind, TokenKind::LParen) =>
                {
                    self.advance();
                    self.consume_keyword(TokenKind::LParen, "expected '(' after method name")?;
                    let args = self.parse_call_arguments_after_open()?;
                    Ok(Expr::ParentMethodCall {
                        method,
                        args,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(constant) => {
                    self.advance();
                    Ok(Expr::ParentClassConstant {
                        constant,
                        span: operator_span,
                    })
                }
                TokenKind::Class => {
                    self.advance();
                    Ok(Expr::ParentClassNameConstant {
                        span: operator_span,
                    })
                }
                _ => Err(self.error_at(
                    operator_span,
                    format!(
                        "expected parent member name after '::', found {}",
                        token_name(&member.kind)
                    ),
                )),
            };
        }
        if receiver.is_some_and(|receiver| receiver.eq_ignore_ascii_case("self")) {
            let member = self.peek().clone();
            return match member.kind {
                TokenKind::Variable(property) => {
                    self.advance();
                    Ok(Expr::SelfStaticProperty {
                        property,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                    self.advance();
                    Ok(Expr::SelfClassNameConstant {
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(method)
                    if matches!(self.peek_next().kind, TokenKind::LParen) =>
                {
                    self.advance();
                    self.consume_keyword(TokenKind::LParen, "expected '(' after method name")?;
                    let args = self.parse_call_arguments_after_open()?;
                    Ok(Expr::SelfMethodCall {
                        method,
                        args,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(constant) => {
                    self.advance();
                    Ok(Expr::SelfClassConstant {
                        constant,
                        span: operator_span,
                    })
                }
                TokenKind::Class => {
                    self.advance();
                    Ok(Expr::SelfClassNameConstant {
                        span: operator_span,
                    })
                }
                _ => Err(self.error_at(
                    operator_span,
                    format!(
                        "expected self member name after '::', found {}",
                        token_name(&member.kind)
                    ),
                )),
            };
        }
        if receiver.is_some_and(|receiver| receiver.eq_ignore_ascii_case("static")) {
            let member = self.peek().clone();
            return match member.kind {
                TokenKind::Variable(property) => {
                    self.advance();
                    Ok(Expr::LateStaticProperty {
                        property,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                    self.advance();
                    Ok(Expr::StaticClassNameConstant {
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(method)
                    if matches!(self.peek_next().kind, TokenKind::LParen) =>
                {
                    self.advance();
                    self.consume_keyword(TokenKind::LParen, "expected '(' after method name")?;
                    let args = self.parse_call_arguments_after_open()?;
                    Ok(Expr::LateStaticMethodCall {
                        method,
                        args,
                        span: operator_span,
                    })
                }
                TokenKind::Identifier(constant) => {
                    self.advance();
                    Ok(Expr::LateStaticClassConstant {
                        constant,
                        span: operator_span,
                    })
                }
                TokenKind::Class => {
                    self.advance();
                    Ok(Expr::StaticClassNameConstant {
                        span: operator_span,
                    })
                }
                _ => Err(self.error_at(
                    operator_span,
                    format!(
                        "expected static member name after '::', found {}",
                        token_name(&member.kind)
                    ),
                )),
            };
        }
        let member = self.peek().clone();
        match member.kind {
            TokenKind::Variable(property) => {
                self.advance();
                Ok(Expr::StaticProperty {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    property,
                    span: operator_span,
                })
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                self.advance();
                Ok(Expr::ClassNameConstant {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    span: operator_span,
                })
            }
            TokenKind::Identifier(method) if matches!(self.peek_next().kind, TokenKind::LParen) => {
                self.advance();
                self.consume_keyword(TokenKind::LParen, "expected '(' after method name")?;
                let args = self.parse_call_arguments_after_open()?;
                Ok(Expr::StaticMethodCall {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    method,
                    args,
                    span: operator_span,
                })
            }
            TokenKind::Identifier(constant) => {
                self.advance();
                Ok(Expr::ClassConstant {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    constant,
                    span: operator_span,
                })
            }
            TokenKind::Class => {
                self.advance();
                Ok(Expr::ClassNameConstant {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    span: operator_span,
                })
            }
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
            TokenKind::Static => {
                return Err(self.error_at(token.span, unsupported_magic_class_name_message()));
            }
            TokenKind::Identifier(name) => {
                if is_magic_static_receiver(&name) {
                    return Err(self.error_at(token.span, unsupported_magic_class_name_message()));
                }
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

    fn consume_instanceof_class_name(&mut self) -> CompileResult<String> {
        if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_namespace_qualified_class_name_message(),
            ));
        }

        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => {
                if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_namespace_qualified_class_name_message(),
                    ));
                }
                Ok(name)
            }
            TokenKind::Variable(_) => Err(self.error_at(
                token.span,
                "unsupported instanceof class expression: dynamic class names are not implemented",
            )),
            _ => Err(self.error_at(token.span, "expected class name after instanceof")),
        }
    }

    fn parse_array_literal(
        &mut self,
        span: Span,
        delimiter: ArrayLiteralDelimiter,
    ) -> CompileResult<Expr> {
        let mut items = Vec::new();
        if self.match_array_close(delimiter) {
            return Ok(Expr::Array { items, span });
        }

        loop {
            self.reject_unsupported_array_item_syntax()?;
            let first = self.parse_expression()?;
            let item = if self.match_token(|kind| matches!(kind, TokenKind::FatArrow)) {
                self.reject_unsupported_array_item_syntax()?;
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
            if self.check_array_close(delimiter) {
                break;
            }
        }

        self.consume_keyword(delimiter.close_token(), delimiter.close_message())?;
        Ok(Expr::Array { items, span })
    }

    fn reject_unsupported_array_item_syntax(&self) -> CompileResult<()> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Ellipsis => {
                Err(self.error_at(token.span, unsupported_array_spread_message()))
            }
            TokenKind::Ampersand => {
                Err(self.error_at(token.span, unsupported_array_reference_element_message()))
            }
            _ => Ok(()),
        }
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
                if self.check(|kind| matches!(kind, TokenKind::RParen)) {
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
            TokenKind::Ellipsis if matches!(self.peek_next().kind, TokenKind::RParen) => {
                Err(self.error_at(token.span, unsupported_first_class_callable_message()))
            }
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
            Expr::Binary {
                left, op, right, ..
            } => {
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.error_at(expr.span(), unsupported_null_coalescing_message()));
                }
                self.ensure_supported_default_expr(left)?;
                self.ensure_supported_default_expr(right)
            }
            Expr::Ternary { .. } | Expr::ShortTernary { .. } => Err(self.error_at(
                expr.span(),
                "default parameter values only support constant expressions in the current subset",
            )),
            Expr::GlobalConstant { .. } | Expr::ClassNameConstant { .. } => Ok(()),
            Expr::MagicLine { .. } => Ok(()),
            Expr::MagicFile { .. } => Ok(()),
            Expr::MagicDir { .. } => Ok(()),
            Expr::MagicFunction { .. } => Ok(()),
            Expr::Variable(_, _)
            | Expr::Cast { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::Index { .. }
            | Expr::AppendIndex { .. }
            | Expr::Property { .. }
            | Expr::MethodCall { .. }
            | Expr::InstanceOf { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::Closure { .. }
            | Expr::Assign { .. }
            | Expr::CompoundAssign { .. }
            | Expr::NullCoalesceAssign { .. }
            | Expr::IncrementDecrement { .. }
            | Expr::New { .. } => Err(self.error_at(
                expr.span(),
                "default parameter values only support constant expressions in the current subset",
            )),
        }
    }

    fn ensure_supported_const_declaration_expr(&self, expr: &Expr) -> CompileResult<()> {
        match expr {
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _) => Ok(()),
            Expr::Array { items, .. } => {
                for item in items {
                    if let Some(key) = &item.key {
                        self.ensure_supported_const_declaration_expr(key)?;
                    }
                    self.ensure_supported_const_declaration_expr(&item.value)?;
                }
                Ok(())
            }
            Expr::Unary { expr, .. } => self.ensure_supported_const_declaration_expr(expr),
            Expr::Binary {
                left, op, right, ..
            } => {
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.error_at(expr.span(), unsupported_null_coalescing_message()));
                }
                self.ensure_supported_const_declaration_expr(left)?;
                self.ensure_supported_const_declaration_expr(right)
            }
            Expr::Ternary { .. } | Expr::ShortTernary { .. } => Err(self.error_at(
                expr.span(),
                "const declaration values only support constant expressions in the current subset",
            )),
            Expr::GlobalConstant { .. } | Expr::ClassNameConstant { .. } => Ok(()),
            Expr::MagicLine { .. } => Ok(()),
            Expr::MagicFile { .. } => Ok(()),
            Expr::MagicDir { .. } => Ok(()),
            Expr::MagicFunction { .. } => Ok(()),
            Expr::Variable(_, _)
            | Expr::Cast { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::Index { .. }
            | Expr::AppendIndex { .. }
            | Expr::Property { .. }
            | Expr::MethodCall { .. }
            | Expr::InstanceOf { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::Closure { .. }
            | Expr::Assign { .. }
            | Expr::CompoundAssign { .. }
            | Expr::NullCoalesceAssign { .. }
            | Expr::IncrementDecrement { .. }
            | Expr::New { .. } => Err(self.error_at(
                expr.span(),
                "const declaration values only support constant expressions in the current subset",
            )),
        }
    }

    fn ensure_supported_static_property_default_expr(&self, expr: &Expr) -> CompileResult<()> {
        self.ensure_supported_const_declaration_expr(expr)
            .map_err(|_error| {
                self.error_at(
                    expr.span(),
                    "static property default values only support constant expressions in the current subset",
                )
            })
    }

    fn expr_contains_assignment(expr: &Expr) -> bool {
        match expr {
            Expr::Assign { .. } | Expr::CompoundAssign { .. } | Expr::NullCoalesceAssign { .. } => {
                true
            }
            Expr::Array { items, .. } => items.iter().any(|item| {
                item.key
                    .as_ref()
                    .is_some_and(Self::expr_contains_assignment)
                    || Self::expr_contains_assignment(&item.value)
            }),
            Expr::Index { target, index, .. } => {
                Self::expr_contains_assignment(target) || Self::expr_contains_assignment(index)
            }
            Expr::AppendIndex { target, .. } => Self::expr_contains_assignment(target),
            Expr::Property { target, .. } => Self::expr_contains_assignment(target),
            Expr::MethodCall { target, args, .. } => {
                Self::expr_contains_assignment(target)
                    || args.iter().any(Self::expr_contains_assignment)
            }
            Expr::ParentMethodCall { args, .. } => args.iter().any(Self::expr_contains_assignment),
            Expr::StaticMethodCall { args, .. } => args.iter().any(Self::expr_contains_assignment),
            Expr::ObjectStaticMethodCall { target, args, .. } => {
                Self::expr_contains_assignment(target)
                    || args.iter().any(Self::expr_contains_assignment)
            }
            Expr::SelfMethodCall { args, .. } => args.iter().any(Self::expr_contains_assignment),
            Expr::LateStaticMethodCall { args, .. } => {
                args.iter().any(Self::expr_contains_assignment)
            }
            Expr::Call { args, .. } | Expr::New { args, .. } => {
                args.iter().any(Self::expr_contains_assignment)
            }
            Expr::Closure { params, .. } => params
                .iter()
                .filter_map(|param| param.default.as_ref())
                .any(Self::expr_contains_assignment),
            Expr::DynamicCall { callee, args, .. } => {
                Self::expr_contains_assignment(callee)
                    || args.iter().any(Self::expr_contains_assignment)
            }
            Expr::InstanceOf { expr, .. } => Self::expr_contains_assignment(expr),
            Expr::Binary { left, right, .. } => {
                Self::expr_contains_assignment(left) || Self::expr_contains_assignment(right)
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                Self::expr_contains_assignment(condition)
                    || Self::expr_contains_assignment(if_true)
                    || Self::expr_contains_assignment(if_false)
            }
            Expr::ShortTernary {
                condition,
                if_false,
                ..
            } => {
                Self::expr_contains_assignment(condition)
                    || Self::expr_contains_assignment(if_false)
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => {
                Self::expr_contains_assignment(expr)
            }
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::GlobalConstant { .. }
            | Expr::ClassNameConstant { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::IncrementDecrement { .. } => false,
        }
    }

    fn expr_contains_unsupported_assignment_rhs(expr: &Expr) -> bool {
        match expr {
            Expr::Assign { target, expr, .. } => {
                matches!(
                    target.as_ref(),
                    AssignTarget::ArrayIndex { index: None, .. }
                ) || Self::expr_contains_unsupported_assignment_rhs(expr)
            }
            Expr::CompoundAssign { expr, .. } | Expr::NullCoalesceAssign { expr, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(expr)
            }
            Expr::Array { items, .. } => items.iter().any(|item| {
                item.key
                    .as_ref()
                    .is_some_and(Self::expr_contains_unsupported_assignment_rhs)
                    || Self::expr_contains_unsupported_assignment_rhs(&item.value)
            }),
            Expr::Index { target, index, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || Self::expr_contains_unsupported_assignment_rhs(index)
            }
            Expr::AppendIndex { target, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
            }
            Expr::Property { target, .. } => Self::expr_contains_unsupported_assignment_rhs(target),
            Expr::MethodCall { target, args, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || args
                        .iter()
                        .any(Self::expr_contains_unsupported_assignment_rhs)
            }
            Expr::ParentMethodCall { args, .. } => args
                .iter()
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::StaticMethodCall { args, .. } => args
                .iter()
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::ObjectStaticMethodCall { target, args, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || args
                        .iter()
                        .any(Self::expr_contains_unsupported_assignment_rhs)
            }
            Expr::SelfMethodCall { args, .. } => args
                .iter()
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::LateStaticMethodCall { args, .. } => args
                .iter()
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::Call { args, .. } | Expr::New { args, .. } => args
                .iter()
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::Closure { params, .. } => params
                .iter()
                .filter_map(|param| param.default.as_ref())
                .any(Self::expr_contains_unsupported_assignment_rhs),
            Expr::DynamicCall { callee, args, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(callee)
                    || args
                        .iter()
                        .any(Self::expr_contains_unsupported_assignment_rhs)
            }
            Expr::InstanceOf { expr, .. } => Self::expr_contains_unsupported_assignment_rhs(expr),
            Expr::Binary { left, right, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(left)
                    || Self::expr_contains_unsupported_assignment_rhs(right)
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                Self::expr_contains_unsupported_assignment_rhs(condition)
                    || Self::expr_contains_unsupported_assignment_rhs(if_true)
                    || Self::expr_contains_unsupported_assignment_rhs(if_false)
            }
            Expr::ShortTernary {
                condition,
                if_false,
                ..
            } => {
                Self::expr_contains_unsupported_assignment_rhs(condition)
                    || Self::expr_contains_unsupported_assignment_rhs(if_false)
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(expr)
            }
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::GlobalConstant { .. }
            | Expr::ClassNameConstant { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::IncrementDecrement { .. } => false,
        }
    }

    fn find_append_index_span(expr: &Expr) -> Option<Span> {
        match expr {
            Expr::AppendIndex { span, .. } => Some(*span),
            Expr::Array { items, .. } => items.iter().find_map(|item| {
                item.key
                    .as_ref()
                    .and_then(Self::find_append_index_span)
                    .or_else(|| Self::find_append_index_span(&item.value))
            }),
            Expr::Index { target, index, .. } => {
                Self::find_append_index_span(target).or_else(|| Self::find_append_index_span(index))
            }
            Expr::Property { target, .. } => Self::find_append_index_span(target),
            Expr::MethodCall { target, args, .. } => Self::find_append_index_span(target)
                .or_else(|| args.iter().find_map(Self::find_append_index_span)),
            Expr::ParentMethodCall { args, .. } => {
                args.iter().find_map(Self::find_append_index_span)
            }
            Expr::StaticMethodCall { args, .. } => {
                args.iter().find_map(Self::find_append_index_span)
            }
            Expr::ObjectStaticMethodCall { target, args, .. } => {
                Self::find_append_index_span(target)
                    .or_else(|| args.iter().find_map(Self::find_append_index_span))
            }
            Expr::SelfMethodCall { args, .. } => args.iter().find_map(Self::find_append_index_span),
            Expr::LateStaticMethodCall { args, .. } => {
                args.iter().find_map(Self::find_append_index_span)
            }
            Expr::Call { args, .. } | Expr::New { args, .. } => {
                args.iter().find_map(Self::find_append_index_span)
            }
            Expr::Closure { params, .. } => params
                .iter()
                .filter_map(|param| param.default.as_ref())
                .find_map(Self::find_append_index_span),
            Expr::DynamicCall { callee, args, .. } => Self::find_append_index_span(callee)
                .or_else(|| args.iter().find_map(Self::find_append_index_span)),
            Expr::InstanceOf { expr, .. } => Self::find_append_index_span(expr),
            Expr::Binary { left, right, .. } => {
                Self::find_append_index_span(left).or_else(|| Self::find_append_index_span(right))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => Self::find_append_index_span(condition)
                .or_else(|| Self::find_append_index_span(if_true))
                .or_else(|| Self::find_append_index_span(if_false)),
            Expr::ShortTernary {
                condition,
                if_false,
                ..
            } => Self::find_append_index_span(condition)
                .or_else(|| Self::find_append_index_span(if_false)),
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => {
                Self::find_append_index_span(expr)
            }
            Expr::Assign { expr, .. }
            | Expr::CompoundAssign { expr, .. }
            | Expr::NullCoalesceAssign { expr, .. } => Self::find_append_index_span(expr),
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::GlobalConstant { .. }
            | Expr::ClassNameConstant { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::IncrementDecrement { .. } => None,
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
        self.consume_identifier_with_span(message)
            .map(|(name, _span)| name)
    }

    fn consume_identifier_with_span(&mut self, message: &str) -> CompileResult<(String, Span)> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok((name, token.span)),
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

    fn consume_foreach_as(&mut self) -> CompileResult<()> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("as") => Ok(()),
            _ => Err(self.error_at(token.span, "expected 'as' in foreach")),
        }
    }

    fn consume_while_keyword(&mut self, message: &str) -> CompileResult<()> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::While => Ok(()),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("while") => Ok(()),
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

    fn match_identifier(&mut self, expected: &str) -> bool {
        self.match_token(|kind| {
            matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case(expected))
        })
    }

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        predicate(&self.peek().kind)
    }

    fn check_switch_label(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("case") || name.eq_ignore_ascii_case("default")
        )
    }

    fn check_switch_body_end(&self, body_kind: SwitchBodyKind) -> bool {
        match body_kind {
            SwitchBodyKind::Brace => self.check(|kind| matches!(kind, TokenKind::RBrace)),
            SwitchBodyKind::Alternate => self.check(|kind| {
                matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("endswitch"))
            }),
        }
    }

    fn check_alternate_if_boundary(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Else | TokenKind::ElseIf => true,
            TokenKind::Identifier(name) => {
                name.eq_ignore_ascii_case("else")
                    || name.eq_ignore_ascii_case("elseif")
                    || name.eq_ignore_ascii_case("endif")
            }
            _ => false,
        }
    }

    fn check_instanceof_operator(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Instanceof => true,
            TokenKind::Identifier(name) => name.eq_ignore_ascii_case("instanceof"),
            _ => false,
        }
    }

    fn check_low_precedence_logical_operator(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name)
                if name.eq_ignore_ascii_case("and")
                    || name.eq_ignore_ascii_case("xor")
                    || name.eq_ignore_ascii_case("or")
        )
    }

    fn match_array_close(&mut self, delimiter: ArrayLiteralDelimiter) -> bool {
        self.match_token(|kind| same_variant(kind, &delimiter.close_token()))
    }

    fn check_array_close(&self, delimiter: ArrayLiteralDelimiter) -> bool {
        self.check(|kind| same_variant(kind, &delimiter.close_token()))
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
        self.peek_n(1)
    }

    fn peek_n(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.current + offset)
            .unwrap_or_else(|| self.peek())
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check_compound_assignment_operator(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Dot
                | TokenKind::Ampersand
                | TokenKind::Pipe
                | TokenKind::Caret
                | TokenKind::LeftShift
                | TokenKind::RightShift
        ) && matches!(self.peek_next().kind, TokenKind::Equal)
    }

    fn check_increment_decrement_operator(&self) -> bool {
        matches!(
            (&self.peek().kind, &self.peek_next().kind),
            (TokenKind::Plus, TokenKind::Plus) | (TokenKind::Minus, TokenKind::Minus)
        )
    }

    fn match_compound_assignment_operator(&mut self) -> Option<CompoundAssignOp> {
        if !self.check_compound_assignment_operator() {
            return None;
        }

        let op = match self.peek().kind {
            TokenKind::Plus => CompoundAssignOp::Add,
            TokenKind::Minus => CompoundAssignOp::Sub,
            TokenKind::Star => CompoundAssignOp::Mul,
            TokenKind::Slash => CompoundAssignOp::Div,
            TokenKind::Percent => CompoundAssignOp::Mod,
            TokenKind::Dot => CompoundAssignOp::Concat,
            TokenKind::Ampersand => CompoundAssignOp::BitwiseAnd,
            TokenKind::Pipe => CompoundAssignOp::BitwiseOr,
            TokenKind::Caret => CompoundAssignOp::BitwiseXor,
            TokenKind::LeftShift => CompoundAssignOp::ShiftLeft,
            TokenKind::RightShift => CompoundAssignOp::ShiftRight,
            _ => unreachable!("caller checked compound assignment operator"),
        };
        self.advance();
        self.advance();
        Some(op)
    }

    fn match_increment_decrement_operator(&mut self) -> Option<IncrementDecrementOp> {
        if !self.check_increment_decrement_operator() {
            return None;
        }

        let op = match self.peek().kind {
            TokenKind::Plus => IncrementDecrementOp::Increment,
            TokenKind::Minus => IncrementDecrementOp::Decrement,
            _ => unreachable!("caller checked increment/decrement operator"),
        };
        self.advance();
        self.advance();
        Some(op)
    }

    fn error_at(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Parse, span.line, span.column, message)
    }
}

#[derive(Debug, Clone, Copy)]
enum ArrayLiteralDelimiter {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy)]
enum SwitchLabel {
    Case(Span),
    Default(Span),
}

impl ArrayLiteralDelimiter {
    fn close_token(self) -> TokenKind {
        match self {
            ArrayLiteralDelimiter::Short => TokenKind::RBracket,
            ArrayLiteralDelimiter::Long => TokenKind::RParen,
        }
    }

    fn close_message(self) -> &'static str {
        match self {
            ArrayLiteralDelimiter::Short => "expected ']' after array literal",
            ArrayLiteralDelimiter::Long => "expected ')' after array literal",
        }
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
        TokenKind::Interface => "interface",
        TokenKind::Trait => "trait",
        TokenKind::Enum => "enum",
        TokenKind::Abstract => "abstract",
        TokenKind::Final => "final",
        TokenKind::Readonly => "readonly",
        TokenKind::New => "new",
        TokenKind::Public => "public",
        TokenKind::Protected => "protected",
        TokenKind::Private => "private",
        TokenKind::Static => "static",
        TokenKind::Extends => "extends",
        TokenKind::Implements => "implements",
        TokenKind::Clone => "clone",
        TokenKind::Instanceof => "instanceof",
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
        TokenKind::ElseIf => "elseif",
        TokenKind::While => "while",
        TokenKind::Do => "do",
        TokenKind::Foreach => "foreach",
        TokenKind::For => "for",
        TokenKind::Switch => "switch",
        TokenKind::Match => "match",
        TokenKind::Break => "break",
        TokenKind::Continue => "continue",
        TokenKind::Throw => "throw",
        TokenKind::Try => "try",
        TokenKind::Catch => "catch",
        TokenKind::Finally => "finally",
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
        TokenKind::StarStar => "**",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::Dot => ".",
        TokenKind::ObjectOperator => "->",
        TokenKind::DoubleColon => "::",
        TokenKind::Backslash => "\\",
        TokenKind::Ellipsis => "...",
        TokenKind::Ampersand => "&",
        TokenKind::AmpAmp => "&&",
        TokenKind::Question => "?",
        TokenKind::QuestionQuestion => "??",
        TokenKind::Pipe => "|",
        TokenKind::PipePipe => "||",
        TokenKind::Caret => "^",
        TokenKind::Tilde => "~",
        TokenKind::Colon => ":",
        TokenKind::Bang => "!",
        TokenKind::Equal => "=",
        TokenKind::FatArrow => "=>",
        TokenKind::EqualEqual => "==",
        TokenKind::StrictEqual => "===",
        TokenKind::BangEqual => "!=",
        TokenKind::StrictBangEqual => "!==",
        TokenKind::Less => "<",
        TokenKind::LessEqual => "<=",
        TokenKind::LeftShift => "<<",
        TokenKind::Greater => ">",
        TokenKind::GreaterEqual => ">=",
        TokenKind::RightShift => ">>",
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

fn unsupported_require_expression_message() -> &'static str {
    "unsupported require expression: expression-form require is not implemented; use statement-form require path; for local files"
}

fn unsupported_require_once_expression_message() -> &'static str {
    "unsupported require_once expression: expression-form require_once is not implemented; use statement-form require_once path; for local files"
}

fn unsupported_include_expression_message() -> &'static str {
    "unsupported include expression: expression-form include and include return values are not implemented; use statement-form include path; for existing local files"
}

fn unsupported_include_once_expression_message() -> &'static str {
    "unsupported include_once expression: expression-form include_once and include_once return values are not implemented; use statement-form include_once path; for existing local files"
}

fn is_parameter_type_start(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier(_)
            | TokenKind::Question
            | TokenKind::Backslash
            | TokenKind::Static
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False
    )
}

fn unsupported_parameter_type_message() -> &'static str {
    "unsupported parameter type declaration: parameter type enforcement is not implemented"
}

fn unsupported_return_type_message() -> &'static str {
    "unsupported return type declaration: return type enforcement is not implemented"
}

fn unsupported_property_type_message() -> &'static str {
    "unsupported property type declaration: typed property storage and enforcement are not implemented"
}

fn unsupported_static_property_type_message() -> &'static str {
    "unsupported static property type declaration: typed static property metadata, uninitialized state, and write enforcement are not implemented"
}

fn unsupported_multiple_properties_message() -> &'static str {
    "unsupported property declaration: multiple properties in one declaration are not implemented"
}

fn magic_constant_name(name: &str) -> Option<&'static str> {
    match name.to_ascii_uppercase().as_str() {
        "__LINE__" => Some("__LINE__"),
        "__FILE__" => Some("__FILE__"),
        "__DIR__" => Some("__DIR__"),
        "__FUNCTION__" => Some("__FUNCTION__"),
        "__CLASS__" => Some("__CLASS__"),
        "__TRAIT__" => Some("__TRAIT__"),
        "__METHOD__" => Some("__METHOD__"),
        "__NAMESPACE__" => Some("__NAMESPACE__"),
        _ => None,
    }
}

fn unsupported_magic_constant_message(name: &str) -> String {
    if name == "__METHOD__" {
        return "unsupported magic constant __METHOD__: method context evaluation requires method dispatch, which is not implemented".to_string();
    }
    if name == "__CLASS__" {
        return "unsupported magic constant __CLASS__: class context evaluation requires class-context tracking, which is not implemented".to_string();
    }
    if name == "__TRAIT__" {
        return "unsupported magic constant __TRAIT__: trait context evaluation requires trait declarations, trait use, and trait-context tracking, which are not implemented".to_string();
    }
    if name == "__NAMESPACE__" {
        return "unsupported magic constant __NAMESPACE__: namespace context evaluation requires namespace-aware name resolution, which is not implemented".to_string();
    }
    format!(
        "unsupported magic constant {name}: source-aware magic constant evaluation is not implemented"
    )
}

fn unsupported_eval_message() -> &'static str {
    "unsupported eval: eval parsing and caller-scope execution are not implemented"
}

fn unsupported_throw_message() -> &'static str {
    "unsupported throw: exception objects and stack unwinding are not implemented"
}

fn unsupported_try_catch_finally_message() -> &'static str {
    "unsupported try/catch/finally: exception handling and stack unwinding are not implemented"
}

fn unsupported_yield_message() -> &'static str {
    "unsupported yield expression: generators and generator object execution are not implemented"
}

fn unsupported_match_expression_message() -> &'static str {
    "unsupported match expression: expression-form branching is not implemented"
}

fn unsupported_goto_message() -> &'static str {
    "unsupported goto: goto statements and labels are not implemented"
}

fn unsupported_nested_ternary_message() -> &'static str {
    "unsupported nested ternary expression: parenthesize nested ternary expressions in the current subset"
}

fn unsupported_null_coalescing_message() -> &'static str {
    "unsupported null coalescing expression: null-aware expression-form branching is not implemented"
}

fn unsupported_exponentiation_message() -> &'static str {
    "unsupported exponentiation operator: ** and **= are not implemented"
}

fn unsupported_null_coalescing_assignment_message() -> &'static str {
    "unsupported null coalescing assignment: only direct variable, direct array-offset, and direct object-property targets are implemented"
}

fn unsupported_assignment_expression_message() -> &'static str {
    "unsupported assignment expression: this assignment form is not implemented in the current expression context"
}

fn unsupported_assignment_expression_target_message() -> &'static str {
    "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, and direct object properties are implemented; nested targets are not implemented"
}

fn unsupported_chained_assignment_expression_message() -> &'static str {
    "unsupported assignment expression: this chained assignment form is not implemented in the current subset"
}

fn unsupported_compound_assignment_target_message() -> &'static str {
    "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented; append offsets and nested targets are not implemented"
}

fn unsupported_increment_decrement_expression_message() -> &'static str {
    "unsupported increment/decrement expression: chained increment/decrement expressions are not implemented"
}

fn unsupported_increment_decrement_target_message() -> &'static str {
    "unsupported increment/decrement target: only direct static variables, direct array offsets, direct object properties, and supported static properties are implemented for integer and float values; append offsets and nested targets are not implemented"
}

fn unsupported_namespace_message() -> &'static str {
    "unsupported namespace declaration: namespace-aware name resolution is not implemented"
}

fn unsupported_use_message() -> &'static str {
    "unsupported use declaration: namespace imports are not implemented"
}

fn unsupported_nested_const_declaration_message() -> &'static str {
    "unsupported const declaration: only top-level constant declarations are implemented"
}

fn unsupported_namespace_const_declaration_message() -> &'static str {
    "unsupported const declaration: namespace-qualified constant declarations are not implemented"
}

fn unsupported_namespace_qualified_function_name_message() -> &'static str {
    "unsupported namespace-qualified function name: namespace-aware function resolution is not implemented"
}

fn unsupported_namespace_qualified_class_name_message() -> &'static str {
    "unsupported namespace-qualified class name: namespace-aware class resolution is not implemented"
}

fn unsupported_array_spread_message() -> &'static str {
    "unsupported array spread: spread elements are not implemented"
}

fn unsupported_array_reference_element_message() -> &'static str {
    "unsupported array reference element: references are not implemented"
}

fn unsupported_array_destructuring_assignment_message() -> &'static str {
    "unsupported array destructuring assignment: list(...) and [...] destructuring targets are not implemented; use direct variable, array offset, append offset, or object property assignments"
}

fn unsupported_first_class_callable_message() -> &'static str {
    "unsupported first-class callable syntax: Closure creation with ... is not implemented"
}

fn unsupported_unset_message() -> &'static str {
    "unsupported unset: only direct variables like unset($name), direct array offset removal like unset($array[$key]), and direct static property operands like unset(ClassName::$property) are implemented; object property, append, and nested unset forms are not implemented"
}

fn unsupported_object_property_unset_message() -> &'static str {
    "unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled"
}

fn unsupported_foreach_expression_message() -> &'static str {
    "unsupported foreach: foreach is only supported as a statement in the current subset"
}

fn unsupported_foreach_reference_message() -> &'static str {
    "unsupported foreach: by-reference iteration is not implemented; only by-value iteration is supported"
}

fn unsupported_foreach_destructuring_message() -> &'static str {
    "unsupported foreach: destructuring loop targets are not implemented"
}

fn unsupported_do_while_expression_message() -> &'static str {
    "unsupported do-while: do-while loops are only supported as statements in the current subset"
}

fn unsupported_for_expression_message() -> &'static str {
    "unsupported for: for loops are only supported as statements in the current subset"
}

fn unsupported_for_header_list_message() -> &'static str {
    "unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot"
}

fn unsupported_switch_expression_message() -> &'static str {
    "unsupported switch: switch is only supported as a statement in the current subset"
}

fn unsupported_if_alternate_message() -> &'static str {
    "unsupported if: alternate if/elseif/else colon/endif syntax is not implemented; use brace blocks or single-statement bodies"
}

fn unsupported_break_depth_message() -> &'static str {
    "unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported"
}

fn unsupported_break_expression_message() -> &'static str {
    "unsupported break: break is only supported as a statement in the current subset"
}

fn unsupported_continue_depth_message() -> &'static str {
    "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported"
}

fn unsupported_continue_expression_message() -> &'static str {
    "unsupported continue: continue is only supported as a statement in the current subset"
}

fn unsupported_class_expression_message() -> &'static str {
    "unsupported class expression: anonymous classes are not implemented"
}

fn unsupported_trait_declaration_message() -> &'static str {
    "unsupported trait declaration: trait parsing and trait use execution are not implemented"
}

fn unsupported_interface_declaration_message() -> &'static str {
    "unsupported interface declaration: interface parsing and implementation execution are not implemented"
}

fn unsupported_interface_implementation_message() -> &'static str {
    "unsupported interface implementation: implements clauses are not implemented"
}

fn unsupported_enum_declaration_message() -> &'static str {
    "unsupported enum declaration: enum parsing and case/value execution are not implemented"
}

fn unsupported_class_modifier_declaration_message() -> &'static str {
    "unsupported class modifier: abstract, final, and readonly class modifiers are not implemented"
}

fn unsupported_clone_message() -> &'static str {
    "unsupported clone expression: object handle copying and __clone dispatch are not implemented"
}

fn unsupported_instanceof_message() -> &'static str {
    "unsupported instanceof expression: class/interface relationship checks are not implemented"
}

fn unsupported_magic_class_name_message() -> &'static str {
    "unsupported magic class name: self, parent, and static class name resolution is not implemented"
}

fn is_magic_static_receiver(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "self" | "parent" | "static"
    )
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
        TokenKind::Use => unsupported_trait_use_message().to_string(),
        TokenKind::Interface => unsupported_interface_declaration_message().to_string(),
        TokenKind::Trait => unsupported_trait_declaration_message().to_string(),
        TokenKind::Enum => unsupported_enum_declaration_message().to_string(),
        TokenKind::Abstract | TokenKind::Final | TokenKind::Readonly => {
            unsupported_class_member_modifier_message().to_string()
        }
        _ => format!("expected class member, found {}", token_name(kind)),
    }
}

fn unsupported_class_member_modifier_message() -> &'static str {
    "unsupported class member modifier: abstract, final, and readonly member modifiers are not implemented"
}

fn unsupported_trait_use_message() -> &'static str {
    "unsupported trait use: trait composition inside classes is not implemented"
}

impl Parser {
    fn check_unsupported_class_modifier_declaration(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Abstract | TokenKind::Final | TokenKind::Readonly
        ) && matches!(self.peek_next().kind, TokenKind::Class)
    }

    fn check_unsupported_property_type_declaration(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return false,
            TokenKind::Identifier(_) | TokenKind::Question | TokenKind::Backslash => {}
            _ => return false,
        }

        self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
            .any(|token| matches!(token.kind, TokenKind::Variable(_)))
    }
}
