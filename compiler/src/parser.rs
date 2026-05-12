use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, ClassDecl, ClassMember, ClassMethodDecl, ClassPropertyDecl,
    ClassVisibility, ConstDeclarator, Expr, ForAction, FunctionDecl, FunctionParam, Program, Span,
    Stmt, SwitchCase, UnaryOp, UnsetTarget,
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
                self.parse_unsupported_static_local_declaration()
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
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => {
                if self.nested_statement_depth == 0 {
                    self.parse_const_declaration()
                } else {
                    self.parse_unsupported_nested_const_declaration()
                }
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("unset") => self.parse_unset(),
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
                if self.check(is_parameter_type_start) {
                    let span = self.peek().span;
                    return Err(self.error_at(span, unsupported_parameter_type_message()));
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
        if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            return Err(self.error_at(self.previous().span, unsupported_return_type_message()));
        }
        self.function_body_depth += 1;
        let body = self.parse_required_block("expected function body");
        self.function_body_depth -= 1;
        let body = body?;

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

        if self.check_unsupported_class_constant_declaration() {
            return Err(self.error_at(
                self.peek().span,
                unsupported_class_constant_declaration_message(),
            ));
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
            return Err(self.error_at(self.peek().span, unsupported_property_type_message()));
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
            return Err(self.error_at(self.previous().span, unsupported_if_alternate_message()));
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
        if self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            let saved = self.current;
            let target = self.parse_assignment_target()?;
            if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                let expr = self.parse_expression()?;
                return Ok(ForAction::Assign { target, expr });
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

        if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            return Err(self.error_at(self.previous().span, unsupported_switch_alternate_message()));
        }

        self.consume_keyword(TokenKind::LBrace, "expected switch body")?;
        let mut cases = Vec::new();
        let mut saw_default = false;

        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            let label = self.consume_switch_label()?;
            match label {
                SwitchLabel::Case(label_span) => {
                    let condition = self.parse_expression()?;
                    self.consume_keyword(TokenKind::Colon, "expected ':' after switch case")?;
                    let body = self.parse_switch_case_body()?;
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
                    self.consume_keyword(TokenKind::Colon, "expected ':' after switch default")?;
                    let body = self.parse_switch_case_body()?;
                    cases.push(SwitchCase {
                        condition: None,
                        body,
                        span: label_span,
                    });
                }
            }
        }

        self.consume_keyword(TokenKind::RBrace, "expected '}' after switch body")?;
        Ok(Stmt::Switch { value, cases, span })
    }

    fn parse_switch_case_body(&mut self) -> CompileResult<Vec<Stmt>> {
        self.nested_statement_depth += 1;
        let result = (|| {
            let mut statements = Vec::new();
            while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof))
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

    fn consume_switch_label(&mut self) -> CompileResult<SwitchLabel> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("case") => {
                Ok(SwitchLabel::Case(token.span))
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("default") => {
                Ok(SwitchLabel::Default(token.span))
            }
            _ => Err(self.error_at(token.span, "expected 'case' or 'default' in switch body")),
        }
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

    fn parse_unsupported_static_local_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Static, "expected 'static'")?
            .span;
        Err(self.error_at(span, unsupported_static_local_message()))
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
            };
        }

        Ok(Stmt::UnsetMany { targets, span })
    }

    fn parse_unset_target(&mut self) -> CompileResult<UnsetTarget> {
        let token = self.advance().clone();
        let (name, target_span) = match token.kind {
            TokenKind::Variable(name) => (name, token.span),
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
        let expr = self.parse_equality()?;
        if self.match_token(|kind| matches!(kind, TokenKind::Question)) {
            return Err(self.error_at(self.previous().span, unsupported_ternary_message()));
        }
        Ok(expr)
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
            } else if self.check_instanceof_operator() {
                return Err(self.error_at(self.peek().span, unsupported_instanceof_message()));
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
            TokenKind::Variable(name) => {
                if name.eq_ignore_ascii_case("this") {
                    return Err(self.error_at(token.span, unsupported_this_message()));
                }
                Ok(Expr::Variable(name, token.span))
            }
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
            TokenKind::Function => Err(self.error_at(
                token.span,
                "unsupported closure: anonymous functions are not implemented",
            )),
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

    fn reject_unsupported_static_member_access(
        &mut self,
        receiver: Option<&str>,
    ) -> CompileResult<Expr> {
        let operator_span = self
            .consume_keyword(TokenKind::DoubleColon, "expected '::'")?
            .span;
        if receiver.is_some_and(is_magic_static_receiver) {
            return Err(self.error_at(operator_span, unsupported_magic_static_receiver_message()));
        }
        let member = self.peek();
        match &member.kind {
            TokenKind::Variable(_) => Err(self.error_at(
                operator_span,
                "unsupported static property access: static property storage is not implemented",
            )),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("class") => {
                Err(self.error_at(operator_span, unsupported_class_name_constant_message()))
            }
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
            TokenKind::Class => {
                Err(self.error_at(operator_span, unsupported_class_name_constant_message()))
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
            Expr::GlobalConstant { .. } => Ok(()),
            Expr::MagicLine { .. } => Ok(()),
            Expr::MagicFile { .. } => Ok(()),
            Expr::MagicDir { .. } => Ok(()),
            Expr::MagicFunction { .. } => Ok(()),
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
            Expr::Binary { left, right, .. } => {
                self.ensure_supported_const_declaration_expr(left)?;
                self.ensure_supported_const_declaration_expr(right)
            }
            Expr::GlobalConstant { .. } => Ok(()),
            Expr::MagicLine { .. } => Ok(()),
            Expr::MagicFile { .. } => Ok(()),
            Expr::MagicDir { .. } => Ok(()),
            Expr::MagicFunction { .. } => Ok(()),
            Expr::Variable(_, _)
            | Expr::Index { .. }
            | Expr::Property { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::New { .. } => Err(self.error_at(
                expr.span(),
                "const declaration values only support constant expressions in the current subset",
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

    fn check(&self, predicate: impl FnOnce(&TokenKind) -> bool) -> bool {
        predicate(&self.peek().kind)
    }

    fn check_switch_label(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("case") || name.eq_ignore_ascii_case("default")
        )
    }

    fn check_instanceof_operator(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Instanceof => true,
            TokenKind::Identifier(name) => name.eq_ignore_ascii_case("instanceof"),
            _ => false,
        }
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
        TokenKind::Slash => "/",
        TokenKind::Dot => ".",
        TokenKind::ObjectOperator => "->",
        TokenKind::DoubleColon => "::",
        TokenKind::Backslash => "\\",
        TokenKind::Ellipsis => "...",
        TokenKind::Ampersand => "&",
        TokenKind::Question => "?",
        TokenKind::Pipe => "|",
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

fn unsupported_multiple_properties_message() -> &'static str {
    "unsupported property declaration: multiple properties in one declaration are not implemented"
}

fn unsupported_static_local_message() -> &'static str {
    "unsupported static local variable declaration: function-local static storage is not implemented"
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

fn unsupported_match_expression_message() -> &'static str {
    "unsupported match expression: expression-form branching is not implemented"
}

fn unsupported_ternary_message() -> &'static str {
    "unsupported ternary expression: expression-form branching is not implemented"
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

fn unsupported_unset_message() -> &'static str {
    "unsupported unset: only direct variables like unset($name) and direct array offset removal like unset($array[$key]) are implemented; property, append, and nested unset forms are not implemented"
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

fn unsupported_switch_alternate_message() -> &'static str {
    "unsupported switch: alternate colon/endswitch syntax is not implemented; use brace switch blocks"
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

fn unsupported_method_call_message() -> &'static str {
    "unsupported method call: method dispatch is not implemented"
}

fn unsupported_this_message() -> &'static str {
    "unsupported object context: $this requires method execution and object binding, which are not implemented"
}

fn unsupported_clone_message() -> &'static str {
    "unsupported clone expression: object handle copying and __clone dispatch are not implemented"
}

fn unsupported_instanceof_message() -> &'static str {
    "unsupported instanceof expression: class/interface relationship checks are not implemented"
}

fn unsupported_class_name_constant_message() -> &'static str {
    "unsupported class name constant: ::class resolution is not implemented"
}

fn unsupported_magic_static_receiver_message() -> &'static str {
    "unsupported magic static receiver: self, parent, and static resolution is not implemented"
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

fn unsupported_class_constant_declaration_message() -> &'static str {
    "unsupported class constant declaration: class constant metadata and lookup are not implemented"
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

    fn check_unsupported_class_constant_declaration(&self) -> bool {
        matches!(&self.peek().kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const"))
    }
}
