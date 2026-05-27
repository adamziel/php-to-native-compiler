use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, CatchClause, CatchType, ClassConstantDecl,
    ClassDecl, ClassMember, ClassMethodDecl, ClassPropertyDecl, ClassVisibility, ClosureCapture,
    CompoundAssignOp, ConstDeclarator, EnumCaseDecl, EnumDecl, Expr, ForAction, FunctionDecl,
    FunctionParam, IncrementDecrementOp, IncrementDecrementPosition, InterfaceDecl,
    InterfaceMethodDecl, NewClassName, Program, ReferenceSource, Span, StaticLocalDeclarator, Stmt,
    SwitchCase, TraitDecl, TraitMethodAliasDecl, TraitMethodPrecedenceDecl,
    TraitMethodVisibilityDecl, TraitUseDecl, TypeDecl, UnaryOp, UnsetTarget, UseImport,
    UseImportKind,
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
    current_namespace: String,
    class_imports: Vec<(String, String)>,
    function_imports: Vec<(String, String)>,
    constant_imports: Vec<(String, String)>,
    function_declarations: Vec<String>,
    constant_declarations: Vec<String>,
    namespace_declared: bool,
    pending_doc_comment: Option<String>,
    trace_parse: bool,
}

#[derive(Clone, Copy)]
enum SwitchBodyKind {
    Brace,
    Alternate,
}

#[derive(Debug, Clone, Copy)]
struct ClassMemberModifiers {
    visibility: ClassVisibility,
    is_static: bool,
    is_abstract: bool,
    is_final: bool,
    abstract_span: Option<Span>,
    final_span: Option<Span>,
}

impl ClassMemberModifiers {
    fn abstract_or_final_span(&self) -> Option<Span> {
        self.abstract_span.or(self.final_span)
    }

    fn abstract_final_conflict_span(&self) -> Option<Span> {
        self.final_span.or(self.abstract_span)
    }
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            nested_statement_depth: 0,
            function_body_depth: 0,
            current_namespace: String::new(),
            class_imports: Vec::new(),
            function_imports: Vec::new(),
            constant_imports: Vec::new(),
            function_declarations: Vec::new(),
            constant_declarations: Vec::new(),
            namespace_declared: false,
            pending_doc_comment: None,
            trace_parse: std::env::var_os("PHPC_TRACE_PARSE").is_some(),
        }
    }

    fn parse_program(mut self) -> CompileResult<Program> {
        let mut statements = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::Eof)) {
            self.trace_parse("top-level");
            if self.skip_doc_comments_before(|kind| matches!(kind, TokenKind::Eof)) {
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> CompileResult<Stmt> {
        while let TokenKind::DocComment(comment) = &self.peek().kind {
            self.pending_doc_comment = Some(comment.clone());
            self.advance();
        }
        if !matches!(
            self.peek().kind,
            TokenKind::Function
                | TokenKind::Class
                | TokenKind::Interface
                | TokenKind::Trait
                | TokenKind::Abstract
                | TokenKind::Final
                | TokenKind::Readonly
        ) {
            self.pending_doc_comment = None;
        }
        match &self.peek().kind {
            TokenKind::Function => self.parse_function(),
            TokenKind::Class => self.parse_class(),
            TokenKind::Interface => self.parse_interface(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Abstract | TokenKind::Final | TokenKind::Readonly => self.parse_class(),
            TokenKind::Namespace => self.parse_namespace(),
            TokenKind::Use => self.parse_use_declaration(),
            TokenKind::Declare => self.parse_unsupported_declare(),
            TokenKind::Eval => self.parse_unsupported_eval(),
            TokenKind::InlineHtml(_) => self.parse_inline_html(),
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
            TokenKind::Throw => self.parse_throw(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Catch => self.parse_unexpected_catch(),
            TokenKind::Finally => self.parse_unexpected_finally(),
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
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("throw") => self.parse_throw(),
            TokenKind::Identifier(name)
                if matches!(name.to_ascii_lowercase().as_str(), "catch") =>
            {
                self.parse_unexpected_catch()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("finally") => {
                self.parse_unexpected_finally()
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("try") => self.parse_try(),
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("yield") => {
                let message = if matches!(
                    &self.peek_next().kind,
                    TokenKind::Identifier(next) if next.eq_ignore_ascii_case("from")
                ) {
                    unsupported_yield_from_message()
                } else {
                    unsupported_yield_message()
                };
                Err(self.error_at(self.peek().span, message))
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
        Ok(Stmt::Function(
            self.parse_function_after_keyword(start, true)?,
        ))
    }

    fn parse_function_after_keyword(
        &mut self,
        start: Span,
        resolve_namespace: bool,
    ) -> CompileResult<FunctionDecl> {
        let is_nested = self.nested_statement_depth > 0 || self.function_body_depth > 0;
        let returns_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
        let name = self.consume_identifier("expected function name")?;
        if resolve_namespace {
            let alias = name.to_ascii_lowercase();
            if self
                .function_imports
                .iter()
                .any(|(import_alias, _)| import_alias == &alias)
            {
                return Err(self.error_at(start, function_declaration_import_conflict_message()));
            }
            if !is_nested {
                self.function_declarations.push(alias);
            }
        }
        let name = if resolve_namespace {
            self.resolve_function_declaration_name(&name)
        } else {
            name
        };
        let doc_comment = self.pending_doc_comment.take();
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
        let end_line = self.previous().span.line;

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            returns_by_reference,
            body,
            is_nested,
            end_line,
            doc_comment,
            span: start,
        })
    }

    fn parse_function_params_after_open(&mut self) -> CompileResult<Vec<FunctionParam>> {
        let mut params = Vec::new();
        let mut saw_default = false;
        if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            loop {
                if self.check(is_promoted_property_parameter_start) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_promoted_property_parameter_message(),
                    ));
                }
                let type_decl = if self.check(is_parameter_type_start) {
                    Some(self.parse_type_decl(unsupported_parameter_type_message())?)
                } else {
                    None
                };
                let by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
                let is_variadic = self.match_token(|kind| matches!(kind, TokenKind::Ellipsis));
                let (name, span) = self.consume_variable_with_span("expected parameter name")?;
                let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                    if is_variadic {
                        return Err(self.error_at(
                            self.previous().span,
                            "unsupported variadic parameter default: variadic parameters cannot declare default values",
                        ));
                    }
                    saw_default = true;
                    let expr = self.parse_expression()?;
                    self.ensure_supported_default_expr(&expr)?;
                    Some(expr)
                } else {
                    if saw_default && !is_variadic {
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
                    is_variadic,
                    default,
                    span,
                });
                if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
                if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                    break;
                }
                if is_variadic {
                    return Err(self.error_at(
                        self.peek().span,
                        "unsupported variadic parameter: variadic parameters must be the final parameter",
                    ));
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
        if self.check(|kind| matches!(kind, TokenKind::LParen)) {
            return Err(self.error_at(self.peek().span, unsupported_dnf_type_message()));
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
            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                return Err(self.error_at(self.peek().span, unsupported_dnf_type_message()));
            }
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
        let mut is_abstract = false;
        let mut is_final = false;
        let mut is_readonly = false;
        let mut modifier_span = None;
        let mut readonly_span = None;

        loop {
            match self.peek().kind {
                TokenKind::Abstract => {
                    if is_abstract {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate abstract modifier in class declaration",
                        ));
                    }
                    is_abstract = true;
                    modifier_span.get_or_insert(self.peek().span);
                    self.advance();
                }
                TokenKind::Final => {
                    if is_final {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate final modifier in class declaration",
                        ));
                    }
                    is_final = true;
                    modifier_span.get_or_insert(self.peek().span);
                    self.advance();
                }
                TokenKind::Readonly => {
                    if is_readonly {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate readonly modifier in class declaration",
                        ));
                    }
                    is_readonly = true;
                    readonly_span = Some(self.peek().span);
                    modifier_span.get_or_insert(self.peek().span);
                    self.advance();
                }
                _ => break,
            }
        }

        if is_abstract && is_final {
            return Err(self.error_at(
                modifier_span.expect("abstract/final modifier should set span"),
                "unsupported class modifier combination: abstract final classes are not implemented",
            ));
        }

        if let Some(readonly_span) = readonly_span {
            return Err(self.error_at(readonly_span, unsupported_readonly_class_message()));
        }

        let class_span = self
            .consume_keyword(TokenKind::Class, "expected 'class'")?
            .span;
        let span = modifier_span.unwrap_or(class_span);
        let doc_comment = self.pending_doc_comment.take();
        let is_nested = self.nested_statement_depth > 0;
        let name = self.consume_identifier("expected class name")?;
        let name = self.resolve_declared_class_name(&name);

        let parent = if self.match_token(|kind| matches!(kind, TokenKind::Extends)) {
            Some(self.consume_class_like_name("expected parent class name after 'extends'")?)
        } else {
            None
        };
        let interfaces = if self.match_token(|kind| matches!(kind, TokenKind::Implements)) {
            self.parse_class_implements_list()?
        } else {
            Vec::new()
        };

        self.consume_keyword(TokenKind::LBrace, "expected class body")?;
        let mut members = Vec::new();
        let mut trait_uses = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            self.trace_parse("class member");
            if let TokenKind::DocComment(comment) = &self.peek().kind {
                self.pending_doc_comment = Some(comment.clone());
                self.advance();
                continue;
            }
            if self.check(|kind| matches!(kind, TokenKind::Use)) {
                self.pending_doc_comment = None;
                trait_uses.extend(self.parse_class_trait_use()?);
            } else {
                members.push(self.parse_class_member()?);
            }
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after class body")?;
        let end_line = self.previous().span.line;

        Ok(Stmt::Class(ClassDecl {
            name,
            parent,
            interfaces,
            trait_uses,
            members,
            is_abstract,
            is_final,
            is_readonly,
            is_nested,
            end_line,
            doc_comment,
            span,
        }))
    }

    fn parse_class_implements_list(&mut self) -> CompileResult<Vec<String>> {
        let mut interfaces = Vec::new();
        loop {
            interfaces
                .push(self.consume_class_like_name("expected interface name after 'implements'")?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| matches!(kind, TokenKind::LBrace | TokenKind::Eof)) {
                return Err(self.error_at(
                    self.peek().span,
                    "expected interface name after ',' in implements clause",
                ));
            }
        }
        Ok(interfaces)
    }

    fn parse_trait(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Trait, "expected 'trait'")?
            .span;
        let doc_comment = self.pending_doc_comment.take();
        if self.nested_statement_depth > 0 || self.function_body_depth > 0 {
            return Err(self.error_at(span, unsupported_nested_trait_declaration_message()));
        }
        let name = self.consume_identifier("expected trait name")?;
        let name = self.resolve_declared_class_name(&name);
        self.consume_keyword(TokenKind::LBrace, "expected trait body")?;
        let mut trait_uses = Vec::new();
        let mut constants = Vec::new();
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            self.trace_parse("trait member");
            if let TokenKind::DocComment(comment) = &self.peek().kind {
                self.pending_doc_comment = Some(comment.clone());
                self.advance();
                continue;
            }
            if self.check(|kind| matches!(kind, TokenKind::Use)) {
                self.pending_doc_comment = None;
                trait_uses.extend(self.parse_trait_body_use()?);
            } else if self.check_trait_constant_declaration() {
                constants.push(self.parse_trait_constant()?);
            } else if self.check_trait_property_declaration() {
                properties.push(self.parse_trait_property()?);
            } else {
                methods.push(self.parse_trait_method()?);
            }
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after trait body")?;
        let end_line = self.previous().span.line;
        Ok(Stmt::Trait(TraitDecl {
            name,
            trait_uses,
            constants,
            properties,
            methods,
            end_line,
            doc_comment,
            span,
        }))
    }

    fn parse_trait_body_use(&mut self) -> CompileResult<Vec<TraitUseDecl>> {
        let span = self.consume_keyword(TokenKind::Use, "expected 'use'")?.span;
        let mut trait_uses = Vec::new();
        loop {
            let name = self.consume_class_like_name("expected trait name after 'use'")?;
            trait_uses.push(TraitUseDecl {
                name,
                aliases: Vec::new(),
                visibility_adaptations: Vec::new(),
                precedences: Vec::new(),
                span,
            });
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| {
                matches!(
                    kind,
                    TokenKind::Semicolon | TokenKind::LBrace | TokenKind::Eof
                )
            }) {
                return Err(self.error_at(
                    self.peek().span,
                    "expected trait name after ',' in trait-body use",
                ));
            }
        }
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            self.parse_trait_adaptation_block(&mut trait_uses)?;
            return Ok(trait_uses);
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after trait-body use")?;
        Ok(trait_uses)
    }

    fn parse_trait_constant(&mut self) -> CompileResult<ClassConstantDecl> {
        self.pending_doc_comment = None;
        let modifiers = self.parse_class_member_modifiers()?;
        self.match_identifier("const");
        let const_span = self.previous().span;
        if modifiers.is_static {
            return Err(self.error_at(
                const_span,
                "unsupported trait constant declaration: static trait constants are not implemented",
            ));
        }
        if modifiers.is_abstract || modifiers.is_final {
            return Err(self.error_at(
                modifiers.abstract_or_final_span().unwrap_or(const_span),
                "unsupported trait constant declaration: abstract/final trait constants are not implemented",
            ));
        }
        if !matches!(modifiers.visibility, ClassVisibility::Public) {
            return Err(self.error_at(
                const_span,
                "unsupported trait constant declaration: only public trait constants are implemented",
            ));
        }
        if matches!(self.peek().kind, TokenKind::Identifier(_))
            && matches!(self.peek_next().kind, TokenKind::Identifier(_))
        {
            return Err(self.error_at(
                self.peek().span,
                "unsupported trait constant declaration: typed trait constants are not implemented",
            ));
        }
        let (name, name_span) =
            self.consume_identifier_with_span("expected trait constant name after const")?;
        self.consume_keyword(TokenKind::Equal, "expected '=' after trait constant name")?;
        let value = self.parse_expression()?;
        self.ensure_supported_const_declaration_expr(&value)?;
        if self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
            return Err(self.error_at(
                self.previous().span,
                "unsupported trait constant declaration: multiple trait constants in one declaration are not implemented",
            ));
        }
        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after trait constant declaration",
        )?;
        Ok(ClassConstantDecl {
            name,
            visibility: ClassVisibility::Public,
            value,
            span: name_span,
        })
    }

    fn parse_trait_property(&mut self) -> CompileResult<ClassPropertyDecl> {
        let modifiers = self.parse_class_member_modifiers()?;
        let doc_comment = self.pending_doc_comment.take();
        if modifiers.is_abstract || modifiers.is_final {
            return Err(self.error_at(
                modifiers
                    .abstract_or_final_span()
                    .unwrap_or_else(|| self.peek().span),
                unsupported_abstract_final_property_message(),
            ));
        }

        let type_decl = if self.check_unsupported_property_type_declaration() {
            let type_decl = self.parse_type_decl(unsupported_property_type_message())?;
            if type_decl.text.contains('|') && type_decl.text.contains('&') {
                return Err(self.error_at(type_decl.span, unsupported_dnf_type_message()));
            }
            Some(type_decl)
        } else {
            None
        };

        let (name, span) = self.consume_variable_with_span("expected trait property name")?;
        let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
            let expr = self.parse_expression()?;
            if modifiers.is_static {
                self.ensure_supported_static_property_default_expr(&expr)?;
            } else {
                self.ensure_supported_instance_property_default_expr(&expr)?;
            }
            if let Some(type_decl) = &type_decl {
                self.ensure_supported_typed_property_default_expr(type_decl, &expr)?;
            }
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
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return Err(self.error_at(self.peek().span, unsupported_property_hook_message()));
        }
        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after trait property declaration",
        )?;
        Ok(ClassPropertyDecl {
            name,
            visibility: modifiers.visibility,
            is_static: modifiers.is_static,
            type_decl,
            default,
            doc_comment,
            span,
        })
    }

    fn parse_trait_method(&mut self) -> CompileResult<ClassMethodDecl> {
        if !self.check_trait_method_declaration() {
            return Err(self.error_at(
                self.peek().span,
                unsupported_trait_member_declaration_message(),
            ));
        }

        let modifiers = self.parse_class_member_modifiers()?;
        let span = self
            .consume_keyword(TokenKind::Function, "expected trait method declaration")?
            .span;
        if modifiers.is_abstract
            || modifiers.is_final
            || !matches!(modifiers.visibility, ClassVisibility::Public)
        {
            return Err(self.error_at(span, unsupported_trait_method_message()));
        }

        let function = self.parse_function_after_keyword(span, false)?;
        Ok(ClassMethodDecl {
            function,
            visibility: ClassVisibility::Public,
            is_static: modifiers.is_static,
            is_abstract: false,
            is_final: false,
            span,
        })
    }

    fn parse_class_trait_use(&mut self) -> CompileResult<Vec<TraitUseDecl>> {
        let span = self.consume_keyword(TokenKind::Use, "expected 'use'")?.span;
        let mut trait_uses = Vec::new();
        loop {
            let name = self.consume_class_like_name("expected trait name after 'use'")?;
            trait_uses.push(TraitUseDecl {
                name,
                aliases: Vec::new(),
                visibility_adaptations: Vec::new(),
                precedences: Vec::new(),
                span,
            });
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| {
                matches!(
                    kind,
                    TokenKind::Semicolon | TokenKind::LBrace | TokenKind::Eof
                )
            }) {
                return Err(self.error_at(
                    self.peek().span,
                    "expected trait name after ',' in trait use",
                ));
            }
        }
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            self.parse_trait_adaptation_block(&mut trait_uses)?;
            return Ok(trait_uses);
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after trait use")?;
        Ok(trait_uses)
    }

    fn parse_trait_adaptation_block(
        &mut self,
        trait_uses: &mut [TraitUseDecl],
    ) -> CompileResult<()> {
        self.consume_keyword(TokenKind::LBrace, "expected trait adaptation block")?;
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            let (first, span) =
                self.consume_identifier_with_span("expected trait method name in adaptation")?;
            let (trait_name, method_name) =
                if self.match_token(|kind| matches!(kind, TokenKind::DoubleColon)) {
                    let method_name = self.consume_identifier("expected trait method name")?;
                    (Some(self.resolve_class_like_name(&first)), method_name)
                } else {
                    (None, first)
                };

            if matches!(&self.peek().kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("insteadof"))
            {
                self.consume_trait_adaptation_insteadof()?;
                let Some(winner_trait_name) = trait_name.clone() else {
                    return Err(self.error_at(
                        span,
                        "unsupported trait use adaptation: unqualified insteadof adaptations are not implemented",
                    ));
                };
                let mut loser_trait_names = Vec::new();
                loser_trait_names
                    .push(self.consume_class_like_name("expected trait name after 'insteadof'")?);
                while self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                    loser_trait_names.push(self.consume_class_like_name(
                        "expected trait name after ',' in insteadof adaptation",
                    )?);
                }
                self.consume_keyword(
                    TokenKind::Semicolon,
                    "expected ';' after trait method insteadof adaptation",
                )?;
                let target_index = trait_uses
                    .iter()
                    .position(|trait_use| trait_use.name.eq_ignore_ascii_case(&winner_trait_name))
                    .ok_or_else(|| {
                        self.error_at(
                            span,
                            "unsupported trait use adaptation: trait-qualified insteadof adaptations must target a trait in the same use declaration",
                        )
                    })?;
                for loser_trait_name in loser_trait_names {
                    if !trait_uses
                        .iter()
                        .any(|trait_use| trait_use.name.eq_ignore_ascii_case(&loser_trait_name))
                    {
                        return Err(self.error_at(
                            span,
                            "unsupported trait use adaptation: insteadof loser traits must be in the same use declaration",
                        ));
                    }
                    trait_uses[target_index]
                        .precedences
                        .push(TraitMethodPrecedenceDecl {
                            trait_name: winner_trait_name.clone(),
                            method_name: method_name.clone(),
                            loser_trait_name,
                            span,
                        });
                }
                continue;
            }
            self.consume_trait_adaptation_as()?;
            let alias_visibility = if let Some(visibility) =
                self.match_trait_visibility_adaptation()
            {
                if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                    self.consume_keyword(
                        TokenKind::Semicolon,
                        "expected ';' after trait method visibility adaptation",
                    )?;
                    let target_index = match &trait_name {
                            Some(name) => trait_uses
                                .iter()
                                .position(|trait_use| trait_use.name.eq_ignore_ascii_case(name))
                                .ok_or_else(|| {
                                    self.error_at(
                                        span,
                                        "unsupported trait use adaptation: trait-qualified visibility adaptations must target a trait in the same use declaration",
                                    )
                                })?,
                            None if trait_uses.len() == 1 => 0,
                            None => {
                                return Err(self.error_at(
                                    span,
                                    "unsupported trait use adaptation: unqualified visibility adaptations with multiple used traits are not implemented",
                                ));
                            }
                        };
                    trait_uses[target_index].visibility_adaptations.push(
                        TraitMethodVisibilityDecl {
                            trait_name,
                            method_name,
                            visibility,
                            span,
                        },
                    );
                    continue;
                }
                visibility
            } else {
                ClassVisibility::Public
            };
            let alias = self.consume_identifier("expected trait method alias after 'as'")?;
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after trait method alias adaptation",
            )?;

            let target_index = match &trait_name {
                Some(name) => trait_uses
                    .iter()
                    .position(|trait_use| trait_use.name.eq_ignore_ascii_case(name))
                    .ok_or_else(|| {
                        self.error_at(
                            span,
                            "unsupported trait use adaptation: trait-qualified aliases must target a trait in the same use declaration",
                        )
                    })?,
                None if trait_uses.len() == 1 => 0,
                None => {
                    return Err(self.error_at(
                        span,
                        "unsupported trait use adaptation: unqualified aliases with multiple used traits are not implemented",
                    ));
                }
            };

            trait_uses[target_index].aliases.push(TraitMethodAliasDecl {
                trait_name,
                method_name,
                alias,
                visibility: alias_visibility,
                span,
            });
        }
        self.consume_keyword(
            TokenKind::RBrace,
            "expected '}' after trait adaptation block",
        )?;
        Ok(())
    }

    fn parse_interface(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Interface, "expected 'interface'")?
            .span;
        let doc_comment = self.pending_doc_comment.take();
        if self.nested_statement_depth > 0 || self.function_body_depth > 0 {
            return Err(self.error_at(span, unsupported_nested_interface_declaration_message()));
        }
        let name = self.consume_identifier("expected interface name")?;
        let name = self.resolve_declared_class_name(&name);
        let mut parents = Vec::new();
        if self.match_token(|kind| matches!(kind, TokenKind::Extends)) {
            parents.push(self.consume_class_like_name("expected interface name after 'extends'")?);
            while self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                parents.push(self.consume_class_like_name("expected interface name after ','")?);
            }
        }

        self.consume_keyword(TokenKind::LBrace, "expected interface body")?;
        let mut constants = Vec::new();
        let mut methods = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            self.trace_parse("interface member");
            if let TokenKind::DocComment(comment) = &self.peek().kind {
                self.pending_doc_comment = Some(comment.clone());
                self.advance();
                continue;
            }
            if self.check_interface_constant_declaration() {
                constants.push(self.parse_interface_constant()?);
            } else {
                methods.push(self.parse_interface_method()?);
            }
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after interface body")?;
        let end_line = self.previous().span.line;

        Ok(Stmt::Interface(InterfaceDecl {
            name,
            parents,
            constants,
            methods,
            end_line,
            doc_comment,
            span,
        }))
    }

    fn parse_interface_constant(&mut self) -> CompileResult<ClassConstantDecl> {
        self.pending_doc_comment = None;
        let modifiers = self.parse_class_member_modifiers()?;
        self.match_identifier("const");
        let const_span = self.previous().span;
        if modifiers.is_static {
            return Err(self.error_at(
                const_span,
                "unsupported interface constant declaration: static interface constants are not implemented",
            ));
        }
        if modifiers.is_abstract || modifiers.is_final {
            return Err(self.error_at(
                modifiers.abstract_or_final_span().unwrap_or(const_span),
                "unsupported interface constant declaration: abstract/final interface constants are not implemented",
            ));
        }
        if !matches!(modifiers.visibility, ClassVisibility::Public) {
            return Err(self.error_at(
                const_span,
                "unsupported interface constant declaration: only public interface constants are implemented",
            ));
        }
        if matches!(self.peek().kind, TokenKind::Identifier(_))
            && matches!(self.peek_next().kind, TokenKind::Identifier(_))
        {
            return Err(self.error_at(
                self.peek().span,
                "unsupported interface constant declaration: typed interface constants are not implemented",
            ));
        }
        let (name, name_span) =
            self.consume_identifier_with_span("expected interface constant name after const")?;
        self.consume_keyword(
            TokenKind::Equal,
            "expected '=' after interface constant name",
        )?;
        let value = self.parse_expression()?;
        self.ensure_supported_const_declaration_expr(&value)?;
        if self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
            return Err(self.error_at(
                self.previous().span,
                "unsupported interface constant declaration: multiple interface constants in one declaration are not implemented",
            ));
        }
        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after interface constant declaration",
        )?;
        Ok(ClassConstantDecl {
            name,
            visibility: ClassVisibility::Public,
            value,
            span: name_span,
        })
    }

    fn parse_interface_method(&mut self) -> CompileResult<InterfaceMethodDecl> {
        let modifiers = self.parse_class_member_modifiers()?;
        if !matches!(modifiers.visibility, ClassVisibility::Public) {
            return Err(self.error_at(
                self.previous().span,
                unsupported_interface_method_visibility_message(),
            ));
        }
        if modifiers.is_abstract || modifiers.is_final {
            return Err(self.error_at(
                modifiers
                    .abstract_or_final_span()
                    .unwrap_or_else(|| self.peek().span),
                "unsupported interface method declaration: abstract/final interface methods are not implemented",
            ));
        }
        let span = self
            .consume_keyword(TokenKind::Function, "expected interface method declaration")?
            .span;
        let function = self.parse_function_signature_after_keyword(span)?;
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_interface_method_body_message(),
            ));
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after interface method")?;

        Ok(InterfaceMethodDecl {
            function,
            is_static: modifiers.is_static,
            span,
        })
    }

    fn parse_function_signature_after_keyword(
        &mut self,
        start: Span,
    ) -> CompileResult<FunctionDecl> {
        let returns_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
        let name = self.consume_identifier("expected function name")?;
        let doc_comment = self.pending_doc_comment.take();
        self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;
        let params = self.parse_function_params_after_open()?;
        let return_type = if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            Some(self.parse_type_decl(unsupported_return_type_message())?)
        } else {
            None
        };

        Ok(FunctionDecl {
            name,
            params,
            return_type,
            returns_by_reference,
            body: Vec::new(),
            is_nested: false,
            end_line: start.line,
            doc_comment,
            span: start,
        })
    }

    fn parse_enum(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Enum, "expected 'enum'")?
            .span;
        if self.nested_statement_depth > 0 || self.function_body_depth > 0 {
            return Err(self.error_at(span, unsupported_nested_enum_declaration_message()));
        }
        let name = self.consume_identifier("expected enum name")?;
        let name = self.resolve_declared_class_name(&name);
        if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            return Err(self.error_at(self.previous().span, unsupported_backed_enum_message()));
        }
        if self.match_token(|kind| matches!(kind, TokenKind::Implements)) {
            return Err(self.error_at(
                self.previous().span,
                unsupported_enum_implementation_message(),
            ));
        }

        self.consume_keyword(TokenKind::LBrace, "expected enum body")?;
        let mut cases = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace | TokenKind::Eof)) {
            self.trace_parse("enum member");
            cases.push(self.parse_enum_case()?);
        }
        self.consume_keyword(TokenKind::RBrace, "expected '}' after enum body")?;
        Ok(Stmt::Enum(EnumDecl { name, cases, span }))
    }

    fn parse_enum_case(&mut self) -> CompileResult<EnumCaseDecl> {
        if !self.check(
            |kind| matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("case")),
        ) {
            return Err(self.error_at(self.peek().span, unsupported_enum_member_message()));
        }
        let span = self.advance().span;
        let name = self.consume_identifier("expected enum case name")?;
        if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
            return Err(self.error_at(self.previous().span, unsupported_enum_case_value_message()));
        }
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after enum case")?;
        Ok(EnumCaseDecl { name, span })
    }

    fn parse_class_member(&mut self) -> CompileResult<ClassMember> {
        let modifiers = self.parse_class_member_modifiers()?;

        if self.match_identifier("const") {
            self.pending_doc_comment = None;
            let const_span = self.previous().span;
            if modifiers.is_static {
                return Err(self.error_at(
                    const_span,
                    "unsupported class constant declaration: static class constants are not implemented",
                ));
            }
            if modifiers.is_abstract || modifiers.is_final {
                return Err(self.error_at(
                    modifiers.abstract_or_final_span().unwrap_or(const_span),
                    unsupported_abstract_final_class_constant_message(),
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
                visibility: modifiers.visibility,
                value,
                span: name_span,
            }));
        }

        if self.match_token(|kind| matches!(kind, TokenKind::Function)) {
            let span = self.previous().span;
            if modifiers.is_abstract && modifiers.is_final {
                return Err(self.error_at(
                    modifiers.abstract_final_conflict_span().unwrap_or(span),
                    "unsupported class member modifier combination: abstract final methods are not implemented",
                ));
            }
            let function = if modifiers.is_abstract {
                let function = self.parse_function_signature_after_keyword(span)?;
                self.consume_keyword(
                    TokenKind::Semicolon,
                    "expected ';' after abstract method declaration",
                )?;
                function
            } else {
                self.parse_function_after_keyword(span, false)?
            };
            return Ok(ClassMember::Method(ClassMethodDecl {
                function,
                visibility: modifiers.visibility,
                is_static: modifiers.is_static,
                is_abstract: modifiers.is_abstract,
                is_final: modifiers.is_final,
                span,
            }));
        }

        let doc_comment = self.pending_doc_comment.take();

        if self.check_unsupported_property_type_declaration() {
            if modifiers.is_abstract || modifiers.is_final {
                return Err(self.error_at(
                    modifiers
                        .abstract_or_final_span()
                        .unwrap_or_else(|| self.peek().span),
                    unsupported_abstract_final_property_message(),
                ));
            }
            if let Some(hook_span) = self.property_hook_span_before_member_end() {
                return Err(self.error_at(hook_span, unsupported_property_hook_message()));
            }
            let type_decl = self.parse_type_decl(unsupported_property_type_message())?;
            if type_decl.text.contains('|') && type_decl.text.contains('&') {
                return Err(self.error_at(type_decl.span, unsupported_dnf_type_message()));
            }
            let (name, span) = self.consume_variable_with_span("expected property name")?;
            let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                let expr = self.parse_expression()?;
                if modifiers.is_static {
                    self.ensure_supported_static_property_default_expr(&expr)?;
                } else {
                    self.ensure_supported_instance_property_default_expr(&expr)?;
                }
                self.ensure_supported_typed_property_default_expr(&type_decl, &expr)?;
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
            if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
                return Err(self.error_at(self.peek().span, unsupported_property_hook_message()));
            }
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after property declaration",
            )?;
            return Ok(ClassMember::Property(ClassPropertyDecl {
                name,
                visibility: modifiers.visibility,
                is_static: modifiers.is_static,
                type_decl: Some(type_decl),
                default,
                doc_comment,
                span,
            }));
        }

        if self.check(|kind| matches!(kind, TokenKind::Variable(_))) {
            if modifiers.is_abstract || modifiers.is_final {
                return Err(self.error_at(
                    modifiers
                        .abstract_or_final_span()
                        .unwrap_or_else(|| self.peek().span),
                    unsupported_abstract_final_property_message(),
                ));
            }
            let (name, span) = self.consume_variable_with_span("expected property name")?;
            let default = if self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                let expr = self.parse_expression()?;
                if modifiers.is_static {
                    self.ensure_supported_static_property_default_expr(&expr)?;
                } else {
                    self.ensure_supported_instance_property_default_expr(&expr)?;
                }
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
            if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
                return Err(self.error_at(self.peek().span, unsupported_property_hook_message()));
            }
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after property declaration",
            )?;
            return Ok(ClassMember::Property(ClassPropertyDecl {
                name,
                visibility: modifiers.visibility,
                is_static: modifiers.is_static,
                type_decl: None,
                default,
                doc_comment,
                span,
            }));
        }

        let token = self.peek().clone();
        Err(self.error_at(token.span, unsupported_class_member_message(&token.kind)))
    }

    fn parse_class_member_modifiers(&mut self) -> CompileResult<ClassMemberModifiers> {
        let mut visibility = None;
        let mut is_static = false;
        let mut is_abstract = false;
        let mut is_final = false;
        let mut abstract_span = None;
        let mut final_span = None;

        loop {
            if self.check_asymmetric_property_visibility_modifier() {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_asymmetric_property_visibility_message(),
                ));
            }

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
                TokenKind::Abstract => {
                    if is_abstract {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate abstract modifier in class member declaration",
                        ));
                    }
                    is_abstract = true;
                    abstract_span = Some(self.peek().span);
                    self.advance();
                    continue;
                }
                TokenKind::Final => {
                    if is_final {
                        return Err(self.error_at(
                            self.peek().span,
                            "duplicate final modifier in class member declaration",
                        ));
                    }
                    is_final = true;
                    final_span = Some(self.peek().span);
                    self.advance();
                    continue;
                }
                TokenKind::Readonly => {
                    let span = self.advance().span;
                    if self.check_readonly_property_declaration() {
                        return Err(self.error_at(span, unsupported_readonly_property_message()));
                    }
                    return Err(
                        self.error_at(span, unsupported_readonly_class_member_modifier_message())
                    );
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

        Ok(ClassMemberModifiers {
            visibility: visibility.unwrap_or(ClassVisibility::Public),
            is_static,
            is_abstract,
            is_final,
            abstract_span,
            final_span,
        })
    }

    fn parse_unsupported_declare(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Declare, "expected 'declare'")?
            .span;
        let message = match (
            &self.peek().kind,
            &self.peek_next().kind,
            &self.peek_n(2).kind,
        ) {
            (TokenKind::LParen, TokenKind::Identifier(name), TokenKind::Equal)
                if name.eq_ignore_ascii_case("strict_types") =>
            {
                "unsupported declare directive: strict_types is not implemented"
            }
            (TokenKind::LParen, TokenKind::Identifier(name), TokenKind::Equal)
                if name.eq_ignore_ascii_case("ticks") =>
            {
                "unsupported declare directive: ticks requires tick handlers and execution hooks, which are not implemented"
            }
            (TokenKind::LParen, TokenKind::Identifier(name), TokenKind::Equal)
                if name.eq_ignore_ascii_case("encoding") =>
            {
                "unsupported declare directive: encoding requires source encoding, lexer decoding, and runtime text handling, which are not implemented"
            }
            _ => "unsupported declare directive: declare semantics are not implemented",
        };
        Err(self.error_at(span, message))
    }

    fn parse_namespace(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Namespace, "expected 'namespace'")?
            .span;
        if self.nested_statement_depth > 0 || self.function_body_depth > 0 {
            return Err(self.error_at(span, unsupported_nested_namespace_message()));
        }
        if self.namespace_declared {
            return Err(self.error_at(span, unsupported_multiple_namespace_message()));
        }
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return Err(self.error_at(span, unsupported_bracketed_namespace_message()));
        }
        let name = self.parse_qualified_name(false, "expected namespace name")?;
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return Err(self.error_at(span, unsupported_bracketed_namespace_message()));
        }
        self.consume_keyword(
            TokenKind::Semicolon,
            "expected ';' after namespace declaration",
        )?;
        self.current_namespace = name.clone();
        self.class_imports.clear();
        self.function_imports.clear();
        self.constant_imports.clear();
        self.function_declarations.clear();
        self.constant_declarations.clear();
        self.namespace_declared = true;
        Ok(Stmt::Namespace { name, span })
    }

    fn parse_use_declaration(&mut self) -> CompileResult<Stmt> {
        let span = self.consume_keyword(TokenKind::Use, "expected 'use'")?.span;
        if self.nested_statement_depth > 0 || self.function_body_depth > 0 {
            return Err(self.error_at(span, unsupported_use_message()));
        }
        if let Some(grouped_span) = self.grouped_use_brace_span() {
            return Err(self.error_at(grouped_span, unsupported_grouped_use_message()));
        }
        let kind = if self.match_token(|kind| matches!(kind, TokenKind::Function)) {
            UseImportKind::Function
        } else if self.check(|kind| {
            matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const"))
        }) {
            self.advance();
            UseImportKind::Constant
        } else {
            UseImportKind::Class
        };

        let mut imports = Vec::new();
        loop {
            let (name, import_span) = self.parse_use_import_name()?;
            if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
                return Err(self.error_at(self.peek().span, unsupported_use_message()));
            }
            let alias = if self.match_identifier("as") {
                self.consume_identifier("expected import alias after 'as'")?
            } else {
                name.rsplit('\\')
                    .next()
                    .expect("qualified name has at least one segment")
                    .to_string()
            };

            match kind {
                UseImportKind::Class => {
                    self.class_imports
                        .push((alias.to_ascii_lowercase(), name.clone()));
                }
                UseImportKind::Function => {
                    let alias_key = alias.to_ascii_lowercase();
                    if self
                        .function_imports
                        .iter()
                        .any(|(import_alias, _)| import_alias == &alias_key)
                        || self
                            .function_declarations
                            .iter()
                            .any(|declaration| declaration == &alias_key)
                    {
                        return Err(
                            self.error_at(import_span, function_import_alias_conflict_message())
                        );
                    }
                    self.function_imports
                        .push((alias_key, format!("\\{}", name.trim_start_matches('\\'))));
                }
                UseImportKind::Constant => {
                    if self
                        .constant_imports
                        .iter()
                        .any(|(import_alias, _)| import_alias == &alias)
                        || self
                            .constant_declarations
                            .iter()
                            .any(|declaration| declaration == &alias)
                    {
                        return Err(
                            self.error_at(import_span, constant_import_alias_conflict_message())
                        );
                    }
                    self.constant_imports.push((
                        alias.clone(),
                        format!("\\{}", name.trim_start_matches('\\')),
                    ));
                }
            }

            imports.push(UseImport {
                name,
                alias,
                kind,
                span: import_span,
            });

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if kind == UseImportKind::Class {
                return Err(self.error_at(
                    self.previous().span,
                    unsupported_multiple_class_use_message(),
                ));
            }
        }

        self.consume_keyword(TokenKind::Semicolon, "expected ';' after use declaration")?;

        Ok(Stmt::Use { imports, span })
    }

    fn grouped_use_brace_span(&self) -> Option<Span> {
        let mut offset = 0;
        loop {
            let token = self.peek_n(offset);
            match token.kind {
                TokenKind::LBrace => return Some(token.span),
                TokenKind::Semicolon | TokenKind::Eof => return None,
                _ => offset += 1,
            }
        }
    }

    fn parse_unsupported_eval(&mut self) -> CompileResult<Stmt> {
        let span = self
            .consume_keyword(TokenKind::Eval, "expected 'eval'")?
            .span;
        Err(self.error_at(span, unsupported_eval_message()))
    }

    fn parse_use_import_name(&mut self) -> CompileResult<(String, Span)> {
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return Err(self.error_at(self.peek().span, unsupported_grouped_use_message()));
        }

        let (first, span) = self.consume_identifier_with_span("expected import name")?;
        let mut name = first;

        while self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            name.push('\\');
            if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
                return Err(self.error_at(self.peek().span, unsupported_grouped_use_message()));
            }
            name.push_str(&self.consume_identifier("expected import name")?);
        }

        Ok((name, span))
    }

    fn parse_throw(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        let expr = self.parse_expression()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after throw")?;
        Ok(Stmt::Throw { expr, span })
    }

    fn parse_try(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        let body = self.parse_required_block("expected try block")?;
        let mut catches = Vec::new();

        while self.check_exception_keyword("catch") {
            catches.push(self.parse_catch_clause()?);
        }

        let finally_body = if self.match_exception_keyword("finally").is_some() {
            Some(self.parse_required_block("expected finally block")?)
        } else {
            None
        };

        if catches.is_empty() && finally_body.is_none() {
            return Err(self.error_at(span, "expected catch or finally after try block"));
        }

        Ok(Stmt::Try {
            body,
            catches,
            finally_body,
            span,
        })
    }

    fn parse_catch_clause(&mut self) -> CompileResult<CatchClause> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after catch")?;
        let types = self.parse_catch_type_list()?;
        let variable = if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            None
        } else {
            Some(self.consume_variable("expected catch variable")?)
        };
        self.consume_keyword(TokenKind::RParen, "expected ')' after catch clause")?;
        let body = self.parse_required_block("expected catch block")?;

        Ok(CatchClause {
            types,
            variable,
            body,
            span,
        })
    }

    fn parse_catch_type_list(&mut self) -> CompileResult<Vec<CatchType>> {
        let mut types = vec![self.parse_catch_type_name()?];

        while self.match_token(|kind| matches!(kind, TokenKind::Pipe)) {
            types.push(self.parse_catch_type_name()?);
        }

        Ok(types)
    }

    fn parse_catch_type_name(&mut self) -> CompileResult<CatchType> {
        let span = self.peek().span;
        let mut name = String::new();

        if self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            name.push('\\');
        }

        let (first, _) = self.consume_identifier_with_span("expected catch type name")?;
        name.push_str(&first);

        while self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            name.push('\\');
            let (segment, _) = self.consume_identifier_with_span("expected catch type name")?;
            name.push_str(&segment);
        }

        Ok(CatchType { name, span })
    }

    fn parse_unexpected_catch(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, "unexpected catch: catch must follow a try block"))
    }

    fn parse_unexpected_finally(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        Err(self.error_at(span, "unexpected finally: finally must follow a try block"))
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
            if self
                .constant_imports
                .iter()
                .any(|(import_alias, _)| import_alias == &name)
            {
                return Err(
                    self.error_at(name_span, constant_declaration_import_conflict_message())
                );
            }
            self.consume_keyword(TokenKind::Equal, "expected '=' after constant name")?;
            let value = self.parse_expression()?;
            self.ensure_supported_const_declaration_expr(&value)?;
            self.constant_declarations.push(name.clone());
            let name = self.resolve_constant_declaration_name(&name);
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

    fn parse_inline_html(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::InlineHtml(html) => Ok(Stmt::Echo {
                exprs: vec![Expr::String(html, token.span)],
                span: token.span,
            }),
            _ => unreachable!("caller checked inline HTML token"),
        }
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
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_trait_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_enum_declaration_message(),
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

        let initializers = self.parse_for_action_list(TokenKind::Semicolon)?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after for initializer")?;

        let conditions = self.parse_for_condition_list()?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after for condition")?;

        let increments = self.parse_for_action_list(TokenKind::RParen)?;
        self.consume_keyword(TokenKind::RParen, "expected ')' after for increment")?;

        let body = self.parse_block_or_statement()?;
        Ok(Stmt::For {
            initializers,
            conditions,
            increments,
            body,
            span,
        })
    }

    fn parse_for_action_list(&mut self, end: TokenKind) -> CompileResult<Vec<ForAction>> {
        let mut actions = Vec::new();
        if self.check(|kind| same_variant(kind, &end)) {
            return Ok(actions);
        }

        loop {
            actions.push(self.parse_for_action()?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| same_variant(kind, &end)) {
                return Err(self.error_at(
                    self.peek().span,
                    "expected expression after ',' in for header",
                ));
            }
        }

        Ok(actions)
    }

    fn parse_for_condition_list(&mut self) -> CompileResult<Vec<Expr>> {
        let mut conditions = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            return Ok(conditions);
        }

        loop {
            conditions.push(self.parse_expression()?);
            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                return Err(self.error_at(
                    self.peek().span,
                    "expected expression after ',' in for condition",
                ));
            }
        }

        Ok(conditions)
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

    fn parse_foreach(&mut self) -> CompileResult<Stmt> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after foreach")?;
        let iterable = self.parse_expression()?;
        self.consume_foreach_as()?;
        let first_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
        if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_foreach_destructuring_message(),
            ));
        }
        let (first_variable, _) =
            self.consume_variable_with_span("expected foreach value variable")?;
        let (key, value, by_reference) = if self
            .match_token(|kind| matches!(kind, TokenKind::FatArrow))
        {
            if first_by_reference {
                return Err(self.error_at(
                        span,
                        "unsupported foreach: key variables cannot be by-reference in the current subset",
                    ));
            }
            let value_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
            if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_foreach_destructuring_message(),
                ));
            }
            let (value, _) = self.consume_variable_with_span("expected foreach value variable")?;
            (Some(first_variable), value, value_by_reference)
        } else {
            (None, first_variable, first_by_reference)
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
            by_reference,
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
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_trait_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_enum_declaration_message(),
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
        let depth = self.parse_loop_control_depth(token.span, "break")?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after break")?;
        Ok(Stmt::Break {
            depth,
            span: token.span,
        })
    }

    fn parse_continue(&mut self) -> CompileResult<Stmt> {
        let token = self.advance().clone();
        let depth = self.parse_loop_control_depth(token.span, "continue")?;
        self.consume_keyword(TokenKind::Semicolon, "expected ';' after continue")?;
        Ok(Stmt::Continue {
            depth,
            span: token.span,
        })
    }

    fn parse_loop_control_depth(
        &mut self,
        keyword_span: Span,
        keyword: &str,
    ) -> CompileResult<usize> {
        if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            return Ok(1);
        }

        let token = self.advance().clone();
        let TokenKind::Int(depth) = token.kind else {
            let message = if keyword.eq_ignore_ascii_case("break") {
                unsupported_break_depth_message()
            } else {
                unsupported_continue_depth_message()
            };
            return Err(self.error_at(keyword_span, message));
        };

        if depth < 1 {
            return Err(self.error_at(
                token.span,
                format!(
                    "unsupported {keyword}: loop-depth must be a positive integer literal in the current subset"
                ),
            ));
        }

        usize::try_from(depth).map_err(|_| {
            self.error_at(
                token.span,
                format!("unsupported {keyword}: loop-depth is too large for the current subset"),
            )
        })
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
                UnsetTarget::NestedArrayIndex { name, indices, .. } => {
                    Ok(Stmt::UnsetNestedArrayIndex {
                        name,
                        indices,
                        span,
                    })
                }
                UnsetTarget::ObjectProperty {
                    object, property, ..
                } => Ok(Stmt::UnsetObjectProperty {
                    object,
                    property,
                    span,
                }),
                UnsetTarget::DynamicObjectProperty {
                    object, property, ..
                } => Ok(Stmt::UnsetDynamicObjectProperty {
                    object,
                    property,
                    span,
                }),
                target @ (UnsetTarget::ObjectPropertyArrayIndex { .. }
                | UnsetTarget::DynamicObjectPropertyArrayIndex { .. }
                | UnsetTarget::NonDirectObjectPropertyArrayIndex { .. }
                | UnsetTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
                | UnsetTarget::StaticPropertyArrayIndex { .. }
                | UnsetTarget::NonDirectObjectProperty { .. }
                | UnsetTarget::NonDirectDynamicObjectProperty { .. }
                | UnsetTarget::ObjectStaticProperty { .. }) => Ok(Stmt::UnsetMany {
                    targets: vec![target],
                    span,
                }),
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
                let receiver = if is_magic_static_receiver(&name) {
                    name
                } else {
                    self.resolve_class_like_name(&name)
                };
                return self.parse_static_property_unset_target(Some(receiver), token.span);
            }
            TokenKind::Static if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) => {
                return self
                    .parse_static_property_unset_target(Some("static".to_string()), token.span);
            }
            _ => return Err(self.error_at(token.span, unsupported_unset_message())),
        };

        if !self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                return self.parse_object_static_property_unset_target(
                    Expr::Variable(name, target_span),
                    target_span,
                );
            }
            if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                let operator_span = self.previous().span;
                let holder = Expr::Variable(name.clone(), target_span);
                return self.parse_object_property_unset_target_after_operator(
                    holder,
                    Some(name),
                    operator_span,
                    target_span,
                );
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

        let first_index = self.parse_expression()?;
        self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
        let mut indices = vec![first_index];

        while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            let bracket_span = self.previous().span;
            if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                return Err(self.error_at(bracket_span, unsupported_unset_message()));
            }
            let index = self.parse_expression()?;
            self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
            indices.push(index);
        }

        if self.check(|kind| matches!(kind, TokenKind::ObjectOperator)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }
        if !self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }

        if indices.len() == 1 {
            Ok(UnsetTarget::ArrayIndex {
                name,
                index: indices.remove(0),
                span: target_span,
            })
        } else {
            Ok(UnsetTarget::NestedArrayIndex {
                name,
                indices,
                span: target_span,
            })
        }
    }

    fn parse_object_property_unset_target_after_operator(
        &mut self,
        mut holder: Expr,
        mut direct_object: Option<String>,
        mut operator_span: Span,
        target_span: Span,
    ) -> CompileResult<UnsetTarget> {
        loop {
            if matches!(self.peek().kind, TokenKind::Variable(_) | TokenKind::LBrace) {
                let property = self.parse_dynamic_property_name_expr(operator_span)?;
                if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                    holder = Expr::DynamicProperty {
                        target: Box::new(holder),
                        property: Box::new(property),
                        span: operator_span,
                    };
                    direct_object = None;
                    operator_span = self.previous().span;
                    continue;
                }
                if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                    let indices = self.parse_unset_indices_after_open_bracket()?;
                    self.ensure_unset_target_tail_is_done()?;
                    return Ok(match direct_object {
                        Some(object) => UnsetTarget::DynamicObjectPropertyArrayIndex {
                            object,
                            property,
                            indices,
                            span: target_span,
                        },
                        None => UnsetTarget::NonDirectDynamicObjectPropertyArrayIndex {
                            holder,
                            property,
                            indices,
                            span: target_span,
                        },
                    });
                }
                self.ensure_unset_target_tail_is_done()?;
                return Ok(match direct_object {
                    Some(object) => UnsetTarget::DynamicObjectProperty {
                        object,
                        property,
                        span: target_span,
                    },
                    None => UnsetTarget::NonDirectDynamicObjectProperty {
                        holder,
                        property,
                        span: target_span,
                    },
                });
            }

            let (property, _) = self.consume_object_property_name(operator_span)?;
            if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                holder = Expr::Property {
                    target: Box::new(holder),
                    property,
                    span: operator_span,
                };
                direct_object = None;
                operator_span = self.previous().span;
                continue;
            }
            if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                let indices = self.parse_unset_indices_after_open_bracket()?;
                self.ensure_unset_target_tail_is_done()?;
                return Ok(match direct_object {
                    Some(object) => UnsetTarget::ObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span: target_span,
                    },
                    None => UnsetTarget::NonDirectObjectPropertyArrayIndex {
                        holder,
                        property,
                        indices,
                        span: target_span,
                    },
                });
            }
            self.ensure_unset_target_tail_is_done()?;
            return Ok(match direct_object {
                Some(object) => UnsetTarget::ObjectProperty {
                    object,
                    property,
                    span: target_span,
                },
                None => UnsetTarget::NonDirectObjectProperty {
                    holder,
                    property,
                    span: target_span,
                },
            });
        }
    }

    fn parse_unset_indices_after_open_bracket(&mut self) -> CompileResult<Vec<Expr>> {
        let bracket_span = self.previous().span;
        if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
            return Err(self.error_at(bracket_span, unsupported_unset_message()));
        }
        let first_index = self.parse_expression()?;
        self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
        let mut indices = vec![first_index];

        while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            let bracket_span = self.previous().span;
            if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                return Err(self.error_at(bracket_span, unsupported_unset_message()));
            }
            let index = self.parse_expression()?;
            self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
            indices.push(index);
        }

        Ok(indices)
    }

    fn ensure_unset_target_tail_is_done(&self) -> CompileResult<()> {
        if self.check(|kind| matches!(kind, TokenKind::ObjectOperator)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }
        if !self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }
        Ok(())
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

        if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            let indices = self.parse_unset_indices_after_open_bracket()?;
            self.ensure_unset_target_tail_is_done()?;
            let expr = match receiver.as_deref() {
                Some(receiver) if receiver.eq_ignore_ascii_case("self") => {
                    Expr::SelfStaticProperty {
                        property,
                        span: operator_span,
                    }
                }
                Some(receiver) if receiver.eq_ignore_ascii_case("parent") => {
                    Expr::ParentStaticProperty {
                        property,
                        span: operator_span,
                    }
                }
                Some(receiver) if receiver.eq_ignore_ascii_case("static") => {
                    Expr::LateStaticProperty {
                        property,
                        span: operator_span,
                    }
                }
                Some(class_name) => Expr::StaticProperty {
                    class_name: class_name.to_string(),
                    property,
                    span: target_span,
                },
                None => return Err(self.error_at(target_span, unsupported_unset_message())),
            };
            return Ok(UnsetTarget::StaticPropertyArrayIndex {
                expr,
                indices,
                span: target_span,
            });
        }

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

    fn parse_object_static_property_unset_target(
        &mut self,
        target: Expr,
        target_span: Span,
    ) -> CompileResult<UnsetTarget> {
        self.consume_keyword(TokenKind::DoubleColon, "expected '::' after receiver")?;
        let member = self.advance().clone();
        let property = match member.kind {
            TokenKind::Variable(property) => property,
            _ => return Err(self.error_at(member.span, unsupported_unset_message())),
        };

        if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            let indices = self.parse_unset_indices_after_open_bracket()?;
            self.ensure_unset_target_tail_is_done()?;
            return Ok(UnsetTarget::StaticPropertyArrayIndex {
                expr: Expr::ObjectStaticProperty {
                    target: Box::new(target),
                    property,
                    span: target_span,
                },
                indices,
                span: target_span,
            });
        }

        if !self.check(|kind| matches!(kind, TokenKind::RParen | TokenKind::Comma)) {
            return Err(self.error_at(self.peek().span, unsupported_unset_message()));
        }

        Ok(UnsetTarget::ObjectStaticProperty {
            target,
            property,
            span: target_span,
        })
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
                    | AssignTarget::ObjectStaticProperty { .. }
                    | AssignTarget::DynamicObjectStaticProperty { .. }
                    | AssignTarget::StaticProperty { .. }
                    | AssignTarget::DynamicStaticProperty { .. }
                    | AssignTarget::SelfStaticProperty { .. }
                    | AssignTarget::DynamicSelfStaticProperty { .. }
                    | AssignTarget::ParentStaticProperty { .. }
                    | AssignTarget::DynamicParentStaticProperty { .. }
                    | AssignTarget::LateStaticProperty { .. }
                    | AssignTarget::DynamicLateStaticProperty { .. }
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
        if self.check(
            |kind| matches!(kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("list")),
        ) && matches!(self.peek_next().kind, TokenKind::LParen)
        {
            let target = self.parse_list_assignment_target()?;
            if !self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                return Err(self.error_at(
                    target.span(),
                    unsupported_array_destructuring_assignment_message(),
                ));
            }
            let span = target.span();
            let expr = self.parse_assignment_expression()?;
            if self.check_low_precedence_logical_operator() {
                return Err(self.error_at(
                    self.peek().span,
                    unsupported_chained_assignment_expression_message(),
                ));
            }
            if Self::expr_contains_unsupported_assignment_rhs(&expr) {
                return Err(self.error_at(
                    expr.span(),
                    unsupported_chained_assignment_expression_message(),
                ));
            }
            self.consume_keyword(TokenKind::Semicolon, "expected ';' after assignment")?;
            return Ok(Some(Stmt::Assign { target, expr, span }));
        }

        if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
            let saved = self.current;
            let starts_short_destructuring = self.starts_short_destructuring_assignment();
            if starts_short_destructuring {
                let target = self.parse_short_list_assignment_target()?;
                if !self.match_token(|kind| matches!(kind, TokenKind::Equal)) {
                    return Err(self.error_at(
                        target.span(),
                        unsupported_array_destructuring_assignment_message(),
                    ));
                }
                let span = target.span();
                let expr = self.parse_assignment_expression()?;
                if self.check_low_precedence_logical_operator() {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_chained_assignment_expression_message(),
                    ));
                }
                if Self::expr_contains_unsupported_assignment_rhs(&expr) {
                    return Err(self.error_at(
                        expr.span(),
                        unsupported_chained_assignment_expression_message(),
                    ));
                }
                self.consume_keyword(TokenKind::Semicolon, "expected ';' after assignment")?;
                return Ok(Some(Stmt::Assign { target, expr, span }));
            }
            self.current = saved;
        }

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
                    | AssignTarget::List { .. }
                    | AssignTarget::ArrayIndex { index: Some(_), .. }
                    | AssignTarget::NestedArrayIndex { .. }
                    | AssignTarget::StaticPropertyArrayIndex { .. }
                    | AssignTarget::ObjectPropertyArrayIndex { .. }
                    | AssignTarget::DynamicObjectPropertyArrayIndex { .. }
                    | AssignTarget::Property { .. }
                    | AssignTarget::StaticProperty { .. }
                    | AssignTarget::DynamicStaticProperty { .. }
                    | AssignTarget::SelfStaticProperty { .. }
                    | AssignTarget::DynamicSelfStaticProperty { .. }
                    | AssignTarget::ParentStaticProperty { .. }
                    | AssignTarget::DynamicParentStaticProperty { .. }
                    | AssignTarget::LateStaticProperty { .. }
                    | AssignTarget::DynamicLateStaticProperty { .. }
                    | AssignTarget::ObjectStaticProperty { .. }
                    | AssignTarget::DynamicObjectStaticProperty { .. } => {}
                    AssignTarget::NestedArrayAppend { .. }
                    | AssignTarget::StaticPropertyArrayAppend { .. }
                    | AssignTarget::NonDirectObjectPropertyArrayIndex { .. }
                    | AssignTarget::NonDirectObjectPropertyArrayAppend { .. }
                    | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
                    | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { .. }
                    | AssignTarget::ObjectPropertyArrayAppend { .. }
                    | AssignTarget::DynamicObjectPropertyArrayAppend { .. }
                    | AssignTarget::DynamicProperty { .. }
                    | AssignTarget::NonDirectProperty { .. }
                    | AssignTarget::NonDirectDynamicProperty { .. }
                    | AssignTarget::ArrayIndex { index: None, .. } => {
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
        if self.match_token(|kind| matches!(kind, TokenKind::Ampersand)) {
            let source = self.parse_reference_assignment_source()?;
            if self.check_low_precedence_logical_operator() {
                self.current = saved;
                return Ok(None);
            }
            self.consume_keyword(
                TokenKind::Semicolon,
                "expected ';' after reference assignment",
            )?;
            return Ok(Some(Stmt::ReferenceAssign {
                target,
                source,
                span,
            }));
        }

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

    fn parse_reference_assignment_source(&mut self) -> CompileResult<ReferenceSource> {
        let expr = self.parse_non_assignment_expression_with_ternary(true)?;
        let span = expr.span();
        match expr {
            Expr::Variable(name, span) => Ok(ReferenceSource::Variable { name, span }),
            Expr::Index { .. } => {
                if let Some((object, property, mut indices, span)) =
                    Self::dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    if indices.len() == 1 {
                        return Ok(ReferenceSource::DynamicObjectPropertyArrayIndex {
                            object,
                            property,
                            index: indices.remove(0),
                            span,
                        });
                    }
                    return Ok(ReferenceSource::DynamicObjectPropertyNestedArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }

                if let Some((object, property, mut indices, span)) =
                    Self::object_property_array_index_path_from_expr(&expr)
                {
                    if indices.len() == 1 {
                        return Ok(ReferenceSource::ObjectPropertyArrayIndex {
                            object,
                            property,
                            index: indices.remove(0),
                            span,
                        });
                    }
                    return Ok(ReferenceSource::ObjectPropertyNestedArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }

                if let Some((holder, property, indices, span)) =
                    Self::non_direct_dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(
                        ReferenceSource::NonDirectDynamicObjectPropertyNestedArrayIndex {
                            holder,
                            property,
                            indices,
                            span,
                        },
                    );
                }

                if let Some((holder, property, indices, span)) =
                    Self::non_direct_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(ReferenceSource::NonDirectObjectPropertyNestedArrayIndex {
                        holder,
                        property,
                        indices,
                        span,
                    });
                }

                if let Some((expr, indices, span)) =
                    Self::static_property_array_index_path_from_expr(&expr)
                {
                    return Ok(ReferenceSource::StaticPropertyArrayIndex {
                        expr,
                        indices,
                        span,
                    });
                }

                if let Some((name, mut indices, span)) =
                    Self::array_index_path_from_expr(expr.clone())
                {
                    if indices.len() == 1 {
                        return Ok(ReferenceSource::ArrayIndex {
                            name,
                            index: indices.remove(0),
                            span,
                        });
                    }
                    return Ok(ReferenceSource::NestedArrayIndex {
                        name,
                        indices,
                        span,
                    });
                }

                if let Some((target, indices, span)) =
                    Self::expression_array_index_path_from_expr(expr.clone())
                {
                    return Ok(ReferenceSource::ExpressionArrayIndex {
                        target,
                        indices,
                        span,
                    });
                }

                Err(self.error_at(span, unsupported_reference_assignment_source_message()))
            }
            Expr::AppendIndex { target, span } => {
                if let Some((object, property, indices, _)) =
                    Self::dynamic_object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(ReferenceSource::DynamicObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((object, property, indices, _)) =
                    Self::object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(ReferenceSource::ObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, _)) =
                    Self::non_direct_dynamic_object_property_array_append_target_from_expr(
                        target.as_ref(),
                    )
                {
                    return Ok(ReferenceSource::NonDirectDynamicObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, _)) =
                    Self::non_direct_object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(ReferenceSource::NonDirectObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        span,
                    });
                }
                match *target {
                    Expr::Variable(name, _) => Ok(ReferenceSource::ArrayAppend {
                        name,
                        indices: Vec::new(),
                        span,
                    }),
                    nested @ Expr::Index { .. } => {
                        if let Some((name, indices, _)) =
                            Self::array_index_path_from_expr(nested.clone())
                        {
                            Ok(ReferenceSource::ArrayAppend {
                                name,
                                indices,
                                span,
                            })
                        } else if let Some((target, indices, _)) =
                            Self::expression_array_index_path_from_expr(nested)
                        {
                            Ok(ReferenceSource::ExpressionArrayAppend {
                                target,
                                indices,
                                span,
                            })
                        } else {
                            Err(self
                                .error_at(span, unsupported_reference_assignment_source_message()))
                        }
                    }
                    target => {
                        if matches!(target, Expr::Variable(_, _)) {
                            unreachable!("variable append target handled above");
                        }
                        Ok(ReferenceSource::ExpressionArrayAppend {
                            target,
                            indices: Vec::new(),
                            span,
                        })
                    }
                }
            }
            Expr::Property { .. } | Expr::DynamicProperty { .. } => {
                Ok(ReferenceSource::Property { expr, span })
            }
            Expr::StaticProperty { .. }
            | Expr::DynamicStaticProperty { .. }
            | Expr::ObjectStaticProperty { .. }
            | Expr::DynamicObjectStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. } => {
                Ok(ReferenceSource::StaticProperty { expr, span })
            }
            Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::LateStaticMethodCall { .. } => Ok(ReferenceSource::MethodCall { expr, span }),
            _ => Err(self.error_at(span, unsupported_reference_assignment_source_message())),
        }
    }

    fn parse_list_assignment_target(&mut self) -> CompileResult<AssignTarget> {
        let span = self.advance().span;
        self.consume_keyword(TokenKind::LParen, "expected '(' after list")?;
        self.parse_positional_list_assignment_target(span, TokenKind::RParen)
    }

    fn parse_short_list_assignment_target(&mut self) -> CompileResult<AssignTarget> {
        let span = self.advance().span;
        self.parse_positional_list_assignment_target(span, TokenKind::RBracket)
    }

    fn parse_positional_list_assignment_target(
        &mut self,
        span: Span,
        closing_token: TokenKind,
    ) -> CompileResult<AssignTarget> {
        let mut names = Vec::new();

        if self.check(|kind| same_token_kind(kind, &closing_token)) {
            return Err(self.error_at(span, unsupported_array_destructuring_assignment_message()));
        }

        loop {
            match self.peek().kind.clone() {
                TokenKind::Comma => {
                    names.push(None);
                    self.advance();
                    if self.check(|kind| same_token_kind(kind, &closing_token)) {
                        break;
                    }
                    continue;
                }
                TokenKind::Variable(name) => {
                    self.advance();
                    if !self.check(|kind| {
                        matches!(kind, TokenKind::Comma) || same_token_kind(kind, &closing_token)
                    }) {
                        return Err(self.error_at(
                            self.peek().span,
                            unsupported_array_destructuring_assignment_message(),
                        ));
                    }
                    names.push(Some(name));
                }
                _ => {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_array_destructuring_assignment_message(),
                    ));
                }
            }

            if !self.match_token(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            if self.check(|kind| same_token_kind(kind, &closing_token)) {
                break;
            }
        }

        if names.iter().all(Option::is_none) {
            return Err(self.error_at(span, unsupported_array_destructuring_assignment_message()));
        }

        let message = if same_token_kind(&closing_token, &TokenKind::RParen) {
            "expected ')' after list assignment target"
        } else if same_token_kind(&closing_token, &TokenKind::RBracket) {
            "expected ']' after short list assignment target"
        } else {
            unreachable!("list assignment target uses a closing delimiter")
        };
        self.consume_keyword(closing_token, message)?;
        Ok(AssignTarget::List { names, span })
    }

    fn starts_short_destructuring_assignment(&self) -> bool {
        let mut depth = 0usize;
        let mut index = self.current;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                TokenKind::LBracket => depth += 1,
                TokenKind::RBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if depth == 0 {
                        return matches!(
                            self.tokens.get(index + 1).map(|token| &token.kind),
                            Some(TokenKind::Equal)
                        );
                    }
                }
                TokenKind::Semicolon | TokenKind::Eof if depth == 0 => return false,
                _ => {}
            }
            index += 1;
        }
        false
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
            let Some(index) = index else {
                let suffix_indices = self.parse_append_suffix_indices_after_append()?;
                if suffix_indices.is_empty() {
                    return Ok(AssignTarget::ArrayIndex { name, index, span });
                }
                return Ok(AssignTarget::NestedArrayAppend {
                    name,
                    indices: Vec::new(),
                    suffix_indices,
                    span,
                });
            };
            let mut indices = vec![index];
            while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                    let suffix_indices = self.parse_append_suffix_indices_after_append()?;
                    return Ok(AssignTarget::NestedArrayAppend {
                        name,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                indices.push(self.parse_expression()?);
                self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
            }
            if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
                let holder = Self::array_index_expr_from_parts(&name, span, &indices);
                let operator_span = self.previous().span;
                if matches!(self.peek().kind, TokenKind::Variable(_) | TokenKind::LBrace) {
                    let property = self.parse_dynamic_property_name_expr(operator_span)?;
                    let property_span = property.span();
                    if !self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                        return Ok(AssignTarget::NonDirectDynamicProperty {
                            holder,
                            property,
                            span: property_span,
                        });
                    } else {
                        let (indices, suffix_indices, is_append) =
                            self.parse_object_property_array_indices_or_append()?;
                        if is_append {
                            return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
                                holder,
                                property,
                                indices,
                                suffix_indices,
                                span: property_span,
                            });
                        }
                        return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayIndex {
                            holder,
                            property,
                            indices,
                            span: property_span,
                        });
                    }
                } else {
                    let (property, _) = self.consume_object_property_name(operator_span)?;
                    if !self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                        return Ok(AssignTarget::NonDirectProperty {
                            holder,
                            property,
                            span: operator_span,
                        });
                    } else {
                        let (indices, suffix_indices, is_append) =
                            self.parse_object_property_array_indices_or_append()?;
                        if is_append {
                            return Ok(AssignTarget::NonDirectObjectPropertyArrayAppend {
                                holder,
                                property,
                                indices,
                                suffix_indices,
                                span: operator_span,
                            });
                        }
                        return Ok(AssignTarget::NonDirectObjectPropertyArrayIndex {
                            holder,
                            property,
                            indices,
                            span: operator_span,
                        });
                    }
                }
            }
            if indices.len() == 1 {
                return Ok(AssignTarget::ArrayIndex {
                    name,
                    index: indices.pop(),
                    span,
                });
            }
            return Ok(AssignTarget::NestedArrayIndex {
                name,
                indices,
                span,
            });
        }

        if self.match_token(|kind| matches!(kind, TokenKind::ObjectOperator)) {
            let operator_span = self.previous().span;
            if matches!(self.peek().kind, TokenKind::Variable(_) | TokenKind::LBrace) {
                let property = self.parse_dynamic_property_name_expr(operator_span)?;
                if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    return Ok(AssignTarget::Variable { name, span });
                }
                if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                    let (indices, suffix_indices, is_append) =
                        self.parse_object_property_array_indices_or_append()?;
                    if is_append {
                        return Ok(AssignTarget::DynamicObjectPropertyArrayAppend {
                            object: name,
                            property,
                            indices,
                            suffix_indices,
                            span,
                        });
                    }
                    return Ok(AssignTarget::DynamicObjectPropertyArrayIndex {
                        object: name,
                        property,
                        indices,
                        span,
                    });
                }
                return Ok(AssignTarget::DynamicProperty {
                    object: name,
                    property,
                    span,
                });
            }
            let (property, _) = self.consume_object_property_name(operator_span)?;
            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                return Ok(AssignTarget::Variable { name, span });
            }
            if self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                    let suffix_indices = self.parse_append_suffix_indices_after_append()?;
                    return Ok(AssignTarget::ObjectPropertyArrayAppend {
                        object: name,
                        property,
                        indices: Vec::new(),
                        suffix_indices,
                        span,
                    });
                }
                let mut indices = vec![self.parse_expression()?];
                self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
                while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
                    if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                        let suffix_indices = self.parse_append_suffix_indices_after_append()?;
                        return Ok(AssignTarget::ObjectPropertyArrayAppend {
                            object: name,
                            property,
                            indices,
                            suffix_indices,
                            span,
                        });
                    }
                    indices.push(self.parse_expression()?);
                    self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
                }
                return Ok(AssignTarget::ObjectPropertyArrayIndex {
                    object: name,
                    property,
                    indices,
                    span,
                });
            }
            return Ok(AssignTarget::Property {
                object: name,
                property,
                span,
            });
        }

        Ok(AssignTarget::Variable { name, span })
    }

    fn array_index_expr_from_parts(name: &str, span: Span, indices: &[Expr]) -> Expr {
        let mut expr = Expr::Variable(name.to_string(), span);
        for index in indices {
            expr = Expr::Index {
                target: Box::new(expr),
                index: Box::new(index.clone()),
                span: index.span(),
            };
        }
        expr
    }

    fn parse_object_property_array_indices_or_append(
        &mut self,
    ) -> CompileResult<(Vec<Expr>, Vec<Expr>, bool)> {
        if !self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            return Err(self.error_at(
                self.peek().span,
                unsupported_assignment_expression_target_message(),
            ));
        }
        if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
            let suffix_indices = self.parse_append_suffix_indices_after_append()?;
            return Ok((Vec::new(), suffix_indices, true));
        }
        let mut indices = vec![self.parse_expression()?];
        self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
        while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                let suffix_indices = self.parse_append_suffix_indices_after_append()?;
                return Ok((indices, suffix_indices, true));
            }
            indices.push(self.parse_expression()?);
            self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
        }
        Ok((indices, Vec::new(), false))
    }

    fn parse_append_suffix_indices_after_append(&mut self) -> CompileResult<Vec<Expr>> {
        let mut suffix_indices = Vec::new();
        while self.match_token(|kind| matches!(kind, TokenKind::LBracket)) {
            if self.match_token(|kind| matches!(kind, TokenKind::RBracket)) {
                return Err(self.error_at(
                    self.previous().span,
                    unsupported_assignment_expression_target_message(),
                ));
            }
            suffix_indices.push(self.parse_expression()?);
            self.consume_keyword(TokenKind::RBracket, "expected ']' after array index")?;
        }
        Ok(suffix_indices)
    }

    fn ensure_supported_compound_assignment_target(
        target: &AssignTarget,
    ) -> Result<(), &'static str> {
        match target {
            AssignTarget::Variable { .. }
            | AssignTarget::ArrayIndex { index: Some(_), .. }
            | AssignTarget::NestedArrayIndex { .. }
            | AssignTarget::StaticPropertyArrayIndex { .. }
            | AssignTarget::ObjectPropertyArrayIndex { .. }
            | AssignTarget::DynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::Property { .. }
            | AssignTarget::ObjectStaticProperty { .. }
            | AssignTarget::DynamicObjectStaticProperty { .. }
            | AssignTarget::StaticProperty { .. }
            | AssignTarget::DynamicStaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::DynamicSelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::DynamicParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. }
            | AssignTarget::DynamicLateStaticProperty { .. } => Ok(()),
            AssignTarget::List { .. }
            | AssignTarget::DynamicProperty { .. }
            | AssignTarget::NonDirectProperty { .. }
            | AssignTarget::NonDirectDynamicProperty { .. }
            | AssignTarget::NonDirectObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectObjectPropertyArrayAppend { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { .. }
            | AssignTarget::ObjectPropertyArrayAppend { .. }
            | AssignTarget::StaticPropertyArrayAppend { .. }
            | AssignTarget::DynamicObjectPropertyArrayAppend { .. }
            | AssignTarget::NestedArrayAppend { .. }
            | AssignTarget::ArrayIndex { index: None, .. } => {
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
            | AssignTarget::NestedArrayIndex { .. }
            | AssignTarget::StaticPropertyArrayIndex { .. }
            | AssignTarget::ArrayIndex { index: None, .. }
            | AssignTarget::Property { .. }
            | AssignTarget::ObjectPropertyArrayIndex { .. }
            | AssignTarget::DynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::ObjectStaticProperty { .. }
            | AssignTarget::DynamicObjectStaticProperty { .. }
            | AssignTarget::StaticProperty { .. }
            | AssignTarget::DynamicStaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::DynamicSelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::DynamicParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. }
            | AssignTarget::DynamicLateStaticProperty { .. } => Ok(()),
            AssignTarget::NestedArrayAppend { suffix_indices, .. } if suffix_indices.is_empty() => {
                Ok(())
            }
            AssignTarget::List { .. }
            | AssignTarget::DynamicProperty { .. }
            | AssignTarget::NonDirectProperty { .. }
            | AssignTarget::NonDirectDynamicProperty { .. }
            | AssignTarget::NonDirectObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectObjectPropertyArrayAppend { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayIndex { .. }
            | AssignTarget::NonDirectDynamicObjectPropertyArrayAppend { .. }
            | AssignTarget::ObjectPropertyArrayAppend { .. }
            | AssignTarget::StaticPropertyArrayAppend { .. }
            | AssignTarget::DynamicObjectPropertyArrayAppend { .. }
            | AssignTarget::NestedArrayAppend { .. } => {
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
            } => {
                let expr = Expr::Index {
                    target,
                    index,
                    span,
                };
                if let Some((object, property, indices, span)) =
                    Self::object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((expr, indices, span)) =
                    Self::static_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::StaticPropertyArrayIndex {
                        expr,
                        indices,
                        span,
                    });
                }

                let (name, mut indices, span) = Self::array_index_path_from_expr(expr)
                    .ok_or(unsupported_increment_decrement_target_message())?;
                if indices.len() == 1 {
                    Ok(AssignTarget::ArrayIndex {
                        name,
                        index: Some(indices.remove(0)),
                        span,
                    })
                } else {
                    Ok(AssignTarget::NestedArrayIndex {
                        name,
                        indices,
                        span,
                    })
                }
            }
            Expr::AppendIndex { target, span } => match *target {
                expr @ (Expr::StaticProperty { .. }
                | Expr::DynamicStaticProperty { .. }
                | Expr::ObjectStaticProperty { .. }
                | Expr::DynamicObjectStaticProperty { .. }
                | Expr::SelfStaticProperty { .. }
                | Expr::DynamicSelfStaticProperty { .. }
                | Expr::ParentStaticProperty { .. }
                | Expr::DynamicParentStaticProperty { .. }
                | Expr::LateStaticProperty { .. }
                | Expr::DynamicLateStaticProperty { .. }) => {
                    Ok(AssignTarget::StaticPropertyArrayAppend {
                        expr,
                        indices: Vec::new(),
                        suffix_indices: Vec::new(),
                        span,
                    })
                }
                Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                    name,
                    index: None,
                    span,
                }),
                nested @ Expr::Index { .. } => {
                    if let Some((expr, indices, _)) =
                        Self::static_property_array_index_path_from_expr(&nested)
                    {
                        return Ok(AssignTarget::StaticPropertyArrayAppend {
                            expr,
                            indices,
                            suffix_indices: Vec::new(),
                            span,
                        });
                    }
                    let (name, indices, _) = Self::array_index_path_from_expr(nested)
                        .ok_or(unsupported_increment_decrement_target_message())?;
                    Ok(AssignTarget::NestedArrayAppend {
                        name,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    })
                }
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
            Expr::ObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::ObjectStaticProperty {
                target: *target,
                property,
                span,
            }),
            Expr::DynamicObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::DynamicObjectStaticProperty {
                target: *target,
                property: *property,
                span,
            }),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::DynamicStaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::DynamicStaticProperty {
                class_name,
                property: *property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::DynamicSelfStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicSelfStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::DynamicParentStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicParentStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            Expr::DynamicLateStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicLateStaticProperty {
                    property: *property,
                    span,
                })
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
            Expr::Index { .. } => {
                if let Some((object, property, indices, suffix_indices, span)) =
                    Self::dynamic_object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((object, property, indices, suffix_indices, span)) =
                    Self::object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, suffix_indices, span)) =
                    Self::non_direct_dynamic_object_property_array_append_suffix_target_from_expr(
                        &expr,
                    )
                {
                    return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, suffix_indices, span)) =
                    Self::non_direct_object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::NonDirectObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((name, indices, suffix_indices, span)) =
                    Self::array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::NestedArrayAppend {
                        name,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((expr, indices, span)) =
                    Self::static_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::StaticPropertyArrayIndex {
                        expr,
                        indices,
                        span,
                    });
                }

                let (name, mut indices, span) = Self::array_index_path_from_expr(expr)
                    .ok_or(unsupported_compound_assignment_target_message())?;
                if indices.len() == 1 {
                    Ok(AssignTarget::ArrayIndex {
                        name,
                        index: Some(indices.remove(0)),
                        span,
                    })
                } else {
                    Ok(AssignTarget::NestedArrayIndex {
                        name,
                        indices,
                        span,
                    })
                }
            }
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
            Expr::ObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::ObjectStaticProperty {
                target: *target,
                property,
                span,
            }),
            Expr::DynamicObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::DynamicObjectStaticProperty {
                target: *target,
                property: *property,
                span,
            }),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::DynamicStaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::DynamicStaticProperty {
                class_name,
                property: *property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::DynamicSelfStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicSelfStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::DynamicParentStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicParentStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            Expr::DynamicLateStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicLateStaticProperty {
                    property: *property,
                    span,
                })
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
            Expr::Index { .. } => {
                if let Some((object, property, indices, suffix_indices, span)) =
                    Self::dynamic_object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((object, property, indices, suffix_indices, span)) =
                    Self::object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, suffix_indices, span)) =
                    Self::non_direct_dynamic_object_property_array_append_suffix_target_from_expr(
                        &expr,
                    )
                {
                    return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, suffix_indices, span)) =
                    Self::non_direct_object_property_array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::NonDirectObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((name, indices, suffix_indices, span)) =
                    Self::array_append_suffix_target_from_expr(&expr)
                {
                    return Ok(AssignTarget::NestedArrayAppend {
                        name,
                        indices,
                        suffix_indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, span)) =
                    Self::non_direct_dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayIndex {
                        holder,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((holder, property, indices, span)) =
                    Self::non_direct_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::NonDirectObjectPropertyArrayIndex {
                        holder,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((expr, indices, span)) =
                    Self::static_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::StaticPropertyArrayIndex {
                        expr,
                        indices,
                        span,
                    });
                }
                let (name, mut indices, span) = Self::array_index_path_from_expr(expr)
                    .ok_or(unsupported_assignment_expression_target_message())?;
                if indices.len() == 1 {
                    Ok(AssignTarget::ArrayIndex {
                        name,
                        index: Some(indices.remove(0)),
                        span,
                    })
                } else {
                    Ok(AssignTarget::NestedArrayIndex {
                        name,
                        indices,
                        span,
                    })
                }
            }
            Expr::AppendIndex { target, span } => {
                if let Some((expr, indices, _)) =
                    Self::static_property_array_index_path_from_expr(target.as_ref())
                {
                    return Ok(AssignTarget::StaticPropertyArrayAppend {
                        expr,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                if matches!(
                    target.as_ref(),
                    Expr::StaticProperty { .. }
                        | Expr::DynamicStaticProperty { .. }
                        | Expr::ObjectStaticProperty { .. }
                        | Expr::DynamicObjectStaticProperty { .. }
                        | Expr::SelfStaticProperty { .. }
                        | Expr::DynamicSelfStaticProperty { .. }
                        | Expr::ParentStaticProperty { .. }
                        | Expr::DynamicParentStaticProperty { .. }
                        | Expr::LateStaticProperty { .. }
                        | Expr::DynamicLateStaticProperty { .. }
                ) {
                    return Ok(AssignTarget::StaticPropertyArrayAppend {
                        expr: *target,
                        indices: Vec::new(),
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                if let Some((object, property, indices, _)) =
                    Self::dynamic_object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                if let Some((object, property, indices, _)) =
                    Self::object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(AssignTarget::ObjectPropertyArrayAppend {
                        object,
                        property,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                if let Some((holder, property, indices, _)) =
                    Self::non_direct_dynamic_object_property_array_append_target_from_expr(
                        target.as_ref(),
                    )
                {
                    return Ok(AssignTarget::NonDirectDynamicObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                if let Some((holder, property, indices, _)) =
                    Self::non_direct_object_property_array_append_target_from_expr(target.as_ref())
                {
                    return Ok(AssignTarget::NonDirectObjectPropertyArrayAppend {
                        holder,
                        property,
                        indices,
                        suffix_indices: Vec::new(),
                        span,
                    });
                }
                match *target {
                    Expr::Variable(name, _) => Ok(AssignTarget::ArrayIndex {
                        name,
                        index: None,
                        span,
                    }),
                    nested @ Expr::Index { .. } => {
                        let (name, indices, _) = Self::array_index_path_from_expr(nested)
                            .ok_or(unsupported_assignment_expression_target_message())?;
                        Ok(AssignTarget::NestedArrayAppend {
                            name,
                            indices,
                            suffix_indices: Vec::new(),
                            span,
                        })
                    }
                    _ => Err(unsupported_assignment_expression_target_message()),
                }
            }
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
                holder => Ok(AssignTarget::NonDirectProperty {
                    holder,
                    property,
                    span,
                }),
            },
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => match *target {
                Expr::Variable(object, _) => Ok(AssignTarget::DynamicProperty {
                    object,
                    property: *property,
                    span,
                }),
                holder => Ok(AssignTarget::NonDirectDynamicProperty {
                    holder,
                    property: *property,
                    span,
                }),
            },
            Expr::ObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::ObjectStaticProperty {
                target: *target,
                property,
                span,
            }),
            Expr::DynamicObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::DynamicObjectStaticProperty {
                target: *target,
                property: *property,
                span,
            }),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            }),
            Expr::DynamicStaticProperty {
                class_name,
                property,
                span,
            } => Ok(AssignTarget::DynamicStaticProperty {
                class_name,
                property: *property,
                span,
            }),
            Expr::SelfStaticProperty { property, span } => {
                Ok(AssignTarget::SelfStaticProperty { property, span })
            }
            Expr::DynamicSelfStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicSelfStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::ParentStaticProperty { property, span } => {
                Ok(AssignTarget::ParentStaticProperty { property, span })
            }
            Expr::DynamicParentStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicParentStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::LateStaticProperty { property, span } => {
                Ok(AssignTarget::LateStaticProperty { property, span })
            }
            Expr::DynamicLateStaticProperty { property, span } => {
                Ok(AssignTarget::DynamicLateStaticProperty {
                    property: *property,
                    span,
                })
            }
            Expr::Array { .. } => Err(unsupported_array_destructuring_assignment_message()),
            _ => Err(unsupported_assignment_expression_target_message()),
        }
    }

    fn array_index_path_from_expr(expr: Expr) -> Option<(String, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Variable(name, _) => Some((name, vec![*index], span)),
                nested @ Expr::Index { .. } => {
                    let (name, mut indices, span) = Self::array_index_path_from_expr(nested)?;
                    indices.push(*index);
                    Some((name, indices, span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn expression_array_index_path_from_expr(expr: Expr) -> Option<(Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match *target {
                Expr::Index { .. } => {
                    let (target, mut indices, _) =
                        Self::expression_array_index_path_from_expr(*target)?;
                    indices.push(*index);
                    Some((target, indices, span))
                }
                Expr::Variable(_, _) => None,
                target => Some((target, vec![*index], span)),
            },
            _ => None,
        }
    }

    fn static_property_array_index_path_from_expr(expr: &Expr) -> Option<(Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match target.as_ref() {
                Expr::StaticProperty { .. }
                | Expr::DynamicStaticProperty { .. }
                | Expr::ObjectStaticProperty { .. }
                | Expr::DynamicObjectStaticProperty { .. }
                | Expr::SelfStaticProperty { .. }
                | Expr::DynamicSelfStaticProperty { .. }
                | Expr::ParentStaticProperty { .. }
                | Expr::DynamicParentStaticProperty { .. }
                | Expr::LateStaticProperty { .. }
                | Expr::DynamicLateStaticProperty { .. } => {
                    Some(((**target).clone(), vec![(**index).clone()], *span))
                }
                Expr::Index { .. } => {
                    let (expr, mut indices, _) =
                        Self::static_property_array_index_path_from_expr(target.as_ref())?;
                    indices.push((**index).clone());
                    Some((expr, indices, *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn object_property_array_index_path_from_expr(
        expr: &Expr,
    ) -> Option<(String, String, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match target.as_ref() {
                Expr::Property {
                    target, property, ..
                } => match target.as_ref() {
                    Expr::Variable(object, _) => Some((
                        object.clone(),
                        property.clone(),
                        vec![(**index).clone()],
                        *span,
                    )),
                    _ => None,
                },
                Expr::Index { .. } => {
                    let (object, property, mut indices, _) =
                        Self::object_property_array_index_path_from_expr(target.as_ref())?;
                    indices.push((**index).clone());
                    Some((object, property, indices, *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn dynamic_object_property_array_index_path_from_expr(
        expr: &Expr,
    ) -> Option<(String, Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match target.as_ref() {
                Expr::DynamicProperty {
                    target, property, ..
                } => match target.as_ref() {
                    Expr::Variable(object, _) => Some((
                        object.clone(),
                        (**property).clone(),
                        vec![(**index).clone()],
                        *span,
                    )),
                    _ => None,
                },
                Expr::Index { .. } => {
                    let (object, property, mut indices, _) =
                        Self::dynamic_object_property_array_index_path_from_expr(target.as_ref())?;
                    indices.push((**index).clone());
                    Some((object, property, indices, *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn non_direct_object_property_array_index_path_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, String, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match target.as_ref() {
                Expr::Property {
                    target, property, ..
                } => match target.as_ref() {
                    Expr::Variable(_, _) => None,
                    _ => Some((
                        (**target).clone(),
                        property.clone(),
                        vec![(**index).clone()],
                        *span,
                    )),
                },
                Expr::Index { .. } => {
                    let (holder, property, mut indices, _) =
                        Self::non_direct_object_property_array_index_path_from_expr(
                            target.as_ref(),
                        )?;
                    indices.push((**index).clone());
                    Some((holder, property, indices, *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn non_direct_dynamic_object_property_array_index_path_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::Index {
                target,
                index,
                span,
            } => match target.as_ref() {
                Expr::DynamicProperty {
                    target, property, ..
                } => match target.as_ref() {
                    Expr::Variable(_, _) => None,
                    _ => Some((
                        (**target).clone(),
                        (**property).clone(),
                        vec![(**index).clone()],
                        *span,
                    )),
                },
                Expr::Index { .. } => {
                    let (holder, property, mut indices, _) =
                        Self::non_direct_dynamic_object_property_array_index_path_from_expr(
                            target.as_ref(),
                        )?;
                    indices.push((**index).clone());
                    Some((holder, property, indices, *span))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn object_property_array_append_target_from_expr(
        expr: &Expr,
    ) -> Option<(String, String, Vec<Expr>, Span)> {
        match expr {
            Expr::Property {
                target,
                property,
                span,
            } => match target.as_ref() {
                Expr::Variable(object, _) => {
                    Some((object.clone(), property.clone(), Vec::new(), *span))
                }
                _ => None,
            },
            Expr::Index { .. } => Self::object_property_array_index_path_from_expr(expr),
            _ => None,
        }
    }

    fn object_property_array_append_suffix_target_from_expr(
        expr: &Expr,
    ) -> Option<(String, String, Vec<Expr>, Vec<Expr>, Span)> {
        let (target, suffix_indices, span) = Self::append_suffix_target_from_expr(expr)?;
        let (object, property, indices, _) =
            Self::object_property_array_append_target_from_expr(target)?;
        Some((object, property, indices, suffix_indices, span))
    }

    fn dynamic_object_property_array_append_suffix_target_from_expr(
        expr: &Expr,
    ) -> Option<(String, Expr, Vec<Expr>, Vec<Expr>, Span)> {
        let (target, suffix_indices, span) = Self::append_suffix_target_from_expr(expr)?;
        let (object, property, indices, _) =
            Self::dynamic_object_property_array_append_target_from_expr(target)?;
        Some((object, property, indices, suffix_indices, span))
    }

    fn non_direct_object_property_array_append_suffix_target_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, String, Vec<Expr>, Vec<Expr>, Span)> {
        let (target, suffix_indices, span) = Self::append_suffix_target_from_expr(expr)?;
        let (holder, property, indices, _) =
            Self::non_direct_object_property_array_append_target_from_expr(target)?;
        Some((holder, property, indices, suffix_indices, span))
    }

    fn non_direct_dynamic_object_property_array_append_suffix_target_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, Expr, Vec<Expr>, Vec<Expr>, Span)> {
        let (target, suffix_indices, span) = Self::append_suffix_target_from_expr(expr)?;
        let (holder, property, indices, _) =
            Self::non_direct_dynamic_object_property_array_append_target_from_expr(target)?;
        Some((holder, property, indices, suffix_indices, span))
    }

    fn array_append_suffix_target_from_expr(
        expr: &Expr,
    ) -> Option<(String, Vec<Expr>, Vec<Expr>, Span)> {
        let (target, suffix_indices, span) = Self::append_suffix_target_from_expr(expr)?;
        match target {
            Expr::Variable(name, _) => Some((name.clone(), Vec::new(), suffix_indices, span)),
            Expr::Index { .. } => {
                let (name, indices, _) = Self::array_index_path_from_expr(target.clone())?;
                Some((name, indices, suffix_indices, span))
            }
            _ => None,
        }
    }

    fn append_suffix_target_from_expr(expr: &Expr) -> Option<(&Expr, Vec<Expr>, Span)> {
        let mut suffix_indices = Vec::new();
        let mut current = expr;
        while let Expr::Index { target, index, .. } = current {
            suffix_indices.push((**index).clone());
            current = target.as_ref();
        }
        suffix_indices.reverse();

        let Expr::AppendIndex { target, span } = current else {
            return None;
        };
        if suffix_indices.is_empty() {
            return None;
        }
        Some((target.as_ref(), suffix_indices, *span))
    }

    fn dynamic_object_property_array_append_target_from_expr(
        expr: &Expr,
    ) -> Option<(String, Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => match target.as_ref() {
                Expr::Variable(object, _) => {
                    Some((object.clone(), (**property).clone(), Vec::new(), *span))
                }
                _ => None,
            },
            Expr::Index { .. } => Self::dynamic_object_property_array_index_path_from_expr(expr),
            _ => None,
        }
    }

    fn non_direct_object_property_array_append_target_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, String, Vec<Expr>, Span)> {
        match expr {
            Expr::Property {
                target,
                property,
                span,
            } => match target.as_ref() {
                Expr::Variable(_, _) => None,
                _ => Some(((**target).clone(), property.clone(), Vec::new(), *span)),
            },
            Expr::Index { .. } => Self::non_direct_object_property_array_index_path_from_expr(expr),
            _ => None,
        }
    }

    fn non_direct_dynamic_object_property_array_append_target_from_expr(
        expr: &Expr,
    ) -> Option<(Expr, Expr, Vec<Expr>, Span)> {
        match expr {
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => match target.as_ref() {
                Expr::Variable(_, _) => None,
                _ => Some(((**target).clone(), (**property).clone(), Vec::new(), *span)),
            },
            Expr::Index { .. } => {
                Self::non_direct_dynamic_object_property_array_index_path_from_expr(expr)
            }
            _ => None,
        }
    }

    fn null_coalescing_assignment_expression_target_from_expr(
        &self,
        expr: Expr,
    ) -> Result<AssignTarget, &'static str> {
        match expr {
            Expr::Variable(name, span) => Ok(AssignTarget::Variable { name, span }),
            Expr::Index { .. } => {
                if let Some((object, property, indices, span)) =
                    Self::object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::ObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((object, property, indices, span)) =
                    Self::dynamic_object_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::DynamicObjectPropertyArrayIndex {
                        object,
                        property,
                        indices,
                        span,
                    });
                }
                if let Some((expr, indices, span)) =
                    Self::static_property_array_index_path_from_expr(&expr)
                {
                    return Ok(AssignTarget::StaticPropertyArrayIndex {
                        expr,
                        indices,
                        span,
                    });
                }
                let (name, mut indices, span) = Self::array_index_path_from_expr(expr)
                    .ok_or(unsupported_null_coalescing_assignment_message())?;
                if indices.len() == 1 {
                    Ok(AssignTarget::ArrayIndex {
                        name,
                        index: Some(indices.remove(0)),
                        span,
                    })
                } else {
                    Ok(AssignTarget::NestedArrayIndex {
                        name,
                        indices,
                        span,
                    })
                }
            }
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
            Expr::ObjectStaticProperty {
                target,
                property,
                span,
            } => Ok(AssignTarget::ObjectStaticProperty {
                target: *target,
                property,
                span,
            }),
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
                self.trace_parse("block statement");
                if self.skip_doc_comments_before(|kind| {
                    matches!(kind, TokenKind::RBrace | TokenKind::Eof)
                }) {
                    continue;
                }
                if self.check(|kind| matches!(kind, TokenKind::Interface)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_interface_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Trait)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_trait_declaration_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::Enum)) {
                    return Err(self.error_at(
                        self.peek().span,
                        unsupported_nested_enum_declaration_message(),
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
        if self.check(|kind| matches!(kind, TokenKind::Class)) {
            let error = self.error_at(
                self.peek().span,
                "unsupported unbraced nested class declaration: nested class declarations require a braced statement body in the current subset",
            );
            self.nested_statement_depth -= 1;
            return Err(error);
        }
        let result = self.parse_statement();
        self.nested_statement_depth -= 1;
        result
    }

    fn parse_expression(&mut self) -> CompileResult<Expr> {
        self.parse_expression_with_append_read(false)
    }

    fn parse_expression_with_append_read(
        &mut self,
        allow_append_read: bool,
    ) -> CompileResult<Expr> {
        let expr = self.parse_low_precedence_logical_or(allow_append_read)?;
        self.reject_assignment_expression_operator()?;
        Ok(expr)
    }

    fn parse_low_precedence_logical_or(&mut self, allow_append_read: bool) -> CompileResult<Expr> {
        let mut expr = self.parse_low_precedence_logical_xor(allow_append_read)?;
        while self.match_identifier("or") {
            let right = self.parse_low_precedence_logical_xor(allow_append_read)?;
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

    fn parse_low_precedence_logical_xor(&mut self, allow_append_read: bool) -> CompileResult<Expr> {
        let mut expr = self.parse_low_precedence_logical_and(allow_append_read)?;
        while self.match_identifier("xor") {
            let right = self.parse_low_precedence_logical_and(allow_append_read)?;
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

    fn parse_low_precedence_logical_and(&mut self, allow_append_read: bool) -> CompileResult<Expr> {
        let mut expr = self.parse_assignment_expression_with_options(true, allow_append_read)?;
        while self.match_identifier("and") {
            let right = self.parse_assignment_expression_with_options(true, allow_append_read)?;
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
        self.parse_assignment_expression_with_options(true, false)
    }

    fn parse_assignment_expression_without_unparenthesized_ternary(
        &mut self,
    ) -> CompileResult<Expr> {
        self.parse_assignment_expression_with_options(false, false)
    }

    fn parse_assignment_expression_with_options(
        &mut self,
        allow_ternary: bool,
        allow_append_read: bool,
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
            if !allow_append_read {
                if let Some(span) = Self::find_append_index_span(&expr) {
                    return Err(self.error_at(
                        span,
                        "cannot use [] for reading; append syntax is only supported in assignments",
                    ));
                }
            }
            return Ok(expr);
        }

        let operator_span = self.previous().span;
        let target = self
            .assignment_expression_target_from_expr(expr)
            .map_err(|message| self.error_at(operator_span, message))?;
        let span = target.span();

        let value = self.parse_assignment_expression_with_options(allow_ternary, false)?;
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

        if self.match_token(|kind| matches!(kind, TokenKind::At)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::ErrorControl {
                expr: Box::new(expr),
                span,
            });
        }

        if self.match_token(|kind| matches!(kind, TokenKind::Clone)) {
            let span = self.previous().span;
            let expr = self.parse_unary()?;
            return Ok(Expr::Clone {
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
            "int" | "integer" => Ok(Some(CastKind::Int)),
            "bool" | "boolean" => Ok(Some(CastKind::Bool)),
            "float" | "double" => Ok(Some(CastKind::Float)),
            "array" => Ok(Some(CastKind::Array)),
            "real" | "object" | "unset" | "binary" => Err(self.error_at(
                self.peek().span,
                "unsupported cast expression: only (string), (int), (bool), (float), and (array) casts are implemented",
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
                if matches!(self.peek().kind, TokenKind::Variable(_) | TokenKind::LBrace) {
                    let property = self.parse_dynamic_property_name_expr(operator_span)?;
                    if self.match_token(|kind| matches!(kind, TokenKind::LParen)) {
                        let span = expr.span();
                        let args = self.parse_call_arguments_after_open()?;
                        expr = Expr::DynamicMethodCall {
                            target: Box::new(expr),
                            method: Box::new(property),
                            args,
                            span,
                        };
                        continue;
                    }

                    let span = expr.span();
                    expr = Expr::DynamicProperty {
                        target: Box::new(expr),
                        property: Box::new(property),
                        span,
                    };
                    continue;
                }
                let (member, keyword_member) = self.consume_object_property_name(operator_span)?;
                if self.match_token(|kind| matches!(kind, TokenKind::LParen)) {
                    if keyword_member {
                        return Err(self.error_at(
                            operator_span,
                            "unsupported keyword method call: keyword method names after '->' are not implemented",
                        ));
                    }
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

            if self.match_token(|kind| matches!(kind, TokenKind::NullsafeObjectOperator)) {
                return Err(self.error_at(
                    self.previous().span,
                    unsupported_nullsafe_object_operator_message(),
                ));
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
                        self.advance();
                        let span = expr.span();
                        expr = Expr::ObjectClassNameConstant {
                            target: Box::new(expr),
                            span,
                        };
                        continue;
                    }
                    TokenKind::Class => {
                        self.advance();
                        let span = expr.span();
                        expr = Expr::ObjectClassNameConstant {
                            target: Box::new(expr),
                            span,
                        };
                        continue;
                    }
                    TokenKind::Identifier(constant) => {
                        self.advance();
                        let span = expr.span();
                        expr = Expr::ObjectStaticClassConstant {
                            target: Box::new(expr),
                            constant,
                            span,
                        };
                        continue;
                    }
                    TokenKind::Variable(property) => {
                        self.advance();
                        let span = expr.span();
                        expr = Expr::ObjectStaticProperty {
                            target: Box::new(expr),
                            property,
                            span,
                        };
                        continue;
                    }
                    TokenKind::LBrace | TokenKind::Dollar => {
                        let property =
                            self.parse_computed_static_property_name_expr(operator_span)?;
                        let span = expr.span();
                        expr = Expr::DynamicObjectStaticProperty {
                            target: Box::new(expr),
                            property: Box::new(property),
                            span,
                        };
                        continue;
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
            TokenKind::InterpolatedString(parts) => Ok(Expr::InterpolatedString {
                parts,
                span: token.span,
            }),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::Dollar => Err(self.error_at(
                token.span,
                "unsupported variable variable: variable variables are not implemented",
            )),
            TokenKind::LBracket => {
                self.parse_array_literal(token.span, ArrayLiteralDelimiter::Short)
            }
            TokenKind::Class => {
                Err(self.error_at(token.span, unsupported_class_expression_message()))
            }
            TokenKind::New => self.parse_new_expression(token.span),
            TokenKind::Clone => Err(self.error_at(token.span, "expected expression after clone")),
            TokenKind::Instanceof => {
                Err(self.error_at(token.span, unsupported_instanceof_message()))
            }
            TokenKind::Function => self.parse_closure_expression(token.span, false),
            TokenKind::Fn => self.parse_arrow_function_expression(token.span, false),
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
            TokenKind::Include => self.parse_include_expression(false, token.span),
            TokenKind::IncludeOnce => self.parse_include_expression(true, token.span),
            TokenKind::Require => self.parse_require_expression(false, token.span),
            TokenKind::RequireOnce => self.parse_require_expression(true, token.span),
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
                    if magic_name == "__CLASS__" {
                        return Ok(Expr::MagicClass { span: token.span });
                    }
                    if magic_name == "__METHOD__" {
                        return Ok(Expr::MagicMethod { span: token.span });
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
                    let message = if self.check(|kind| {
                        matches!(kind, TokenKind::Identifier(next) if next.eq_ignore_ascii_case("from"))
                    }) {
                        unsupported_yield_from_message()
                    } else {
                        unsupported_yield_message()
                    };
                    return Err(self.error_at(token.span, message));
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
                    let qualified = self.parse_qualified_name_after_first(name.clone())?;
                    if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                        let resolved = self.resolve_class_like_name(&qualified);
                        return self.reject_unsupported_static_member_access(Some(&resolved));
                    }
                    if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
                        return Err(self.error_at(
                            token.span,
                            unsupported_namespace_qualified_constant_name_message(),
                        ));
                    }
                    return Err(self.error_at(
                        token.span,
                        unsupported_namespace_qualified_function_name_message(),
                    ));
                }
                if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                    let receiver = if is_magic_static_receiver(&name) {
                        name
                    } else {
                        self.resolve_class_like_name(&name)
                    };
                    return self.reject_unsupported_static_member_access(Some(&receiver));
                }
                if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    let name = self.resolve_constant_read_name(&name);
                    return Ok(Expr::GlobalConstant {
                        name,
                        span: token.span,
                    });
                }
                self.consume_keyword(TokenKind::LParen, "expected '(' after function name")?;
                let args = self.parse_call_arguments_after_open()?;
                let name = self.resolve_function_call_name(&name);
                Ok(Expr::Call {
                    name,
                    args,
                    span: token.span,
                })
            }
            TokenKind::Backslash => {
                let qualified = format!(
                    "\\{}",
                    self.parse_qualified_name(false, "expected qualified name after '\\'")?
                );
                if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                    let resolved = self.resolve_class_like_name(&qualified);
                    return self.reject_unsupported_static_member_access(Some(&resolved));
                }
                if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    return Err(self.error_at(
                        token.span,
                        unsupported_fully_qualified_function_call_message(),
                    ));
                }
                Err(self.error_at(
                    token.span,
                    unsupported_fully_qualified_constant_name_message(),
                ))
            }
            TokenKind::Namespace if self.check(|kind| matches!(kind, TokenKind::Backslash)) => {
                self.advance();
                let suffix = self.parse_qualified_name(false, "expected qualified name")?;
                let resolved = self.resolve_relative_namespace_class_name(&suffix);
                if self.check(|kind| matches!(kind, TokenKind::DoubleColon)) {
                    return self.reject_unsupported_static_member_access(Some(&resolved));
                }
                if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    return Err(self.error_at(
                        token.span,
                        unsupported_namespace_qualified_constant_name_message(),
                    ));
                }
                Err(self.error_at(
                    token.span,
                    unsupported_namespace_qualified_function_name_message(),
                ))
            }
            TokenKind::Static if self.check(|kind| matches!(kind, TokenKind::Function)) => {
                self.advance();
                self.parse_closure_expression(token.span, true)
            }
            TokenKind::Static if self.check(|kind| matches!(kind, TokenKind::Fn)) => {
                self.advance();
                self.parse_arrow_function_expression(token.span, true)
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

    fn parse_closure_expression(&mut self, span: Span, is_static: bool) -> CompileResult<Expr> {
        let returns_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));

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
            returns_by_reference,
            body,
            is_static,
            is_arrow: false,
            span,
        })
    }

    fn parse_arrow_function_expression(
        &mut self,
        span: Span,
        is_static: bool,
    ) -> CompileResult<Expr> {
        if self.check(|kind| matches!(kind, TokenKind::Ampersand)) {
            let span = self.advance().span;
            return Err(self.error_at(
                span,
                "unsupported reference return: returning closures by reference is not implemented",
            ));
        }

        self.consume_keyword(TokenKind::LParen, "expected '(' after fn")?;
        let params = self.parse_function_params_after_open()?;
        let return_type = if self.match_token(|kind| matches!(kind, TokenKind::Colon)) {
            Some(self.parse_type_decl(unsupported_return_type_message())?)
        } else {
            None
        };
        self.consume_keyword(
            TokenKind::FatArrow,
            "expected '=>' after arrow function parameters",
        )?;
        self.function_body_depth += 1;
        let value = self.parse_expression();
        self.function_body_depth -= 1;
        let value = value?;

        Ok(Expr::Closure {
            params,
            captures: Vec::new(),
            return_type,
            returns_by_reference: false,
            body: vec![Stmt::Return {
                value: Some(value),
                span,
            }],
            is_static,
            is_arrow: true,
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
                TokenKind::LBrace | TokenKind::Dollar => {
                    let property = self.parse_computed_static_property_name_expr(operator_span)?;
                    Ok(Expr::DynamicParentStaticProperty {
                        property: Box::new(property),
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
                TokenKind::LBrace | TokenKind::Dollar => {
                    let property = self.parse_computed_static_property_name_expr(operator_span)?;
                    Ok(Expr::DynamicSelfStaticProperty {
                        property: Box::new(property),
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
                TokenKind::LBrace | TokenKind::Dollar => {
                    let property = self.parse_computed_static_property_name_expr(operator_span)?;
                    Ok(Expr::DynamicLateStaticProperty {
                        property: Box::new(property),
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
            TokenKind::LBrace | TokenKind::Dollar => {
                let property = self.parse_computed_static_property_name_expr(operator_span)?;
                Ok(Expr::DynamicStaticProperty {
                    class_name: receiver
                        .expect("named static receiver should exist")
                        .to_string(),
                    property: Box::new(property),
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

        let token = self.advance().clone();
        let class_name = match token.kind {
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.consume_keyword(
                    TokenKind::RParen,
                    "expected ')' after dynamic class-name expression in new",
                )?;
                NewClassName::DynamicExpression(Box::new(expr))
            }
            TokenKind::Backslash => {
                let raw = format!(
                    "\\{}",
                    self.parse_qualified_name(false, "expected class name after 'new'")?
                );
                NewClassName::Named(self.resolve_class_like_name(&raw))
            }
            TokenKind::Static => NewClassName::StaticClass,
            TokenKind::Identifier(name) => {
                if name.eq_ignore_ascii_case("self") {
                    NewClassName::SelfClass
                } else if name.eq_ignore_ascii_case("parent") {
                    NewClassName::ParentClass
                } else if name.eq_ignore_ascii_case("static") {
                    NewClassName::StaticClass
                } else if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                    let raw = self.parse_qualified_name_after_first(name)?;
                    NewClassName::Named(self.resolve_class_like_name(&raw))
                } else {
                    NewClassName::Named(self.resolve_class_like_name(&name))
                }
            }
            TokenKind::Namespace if self.check(|kind| matches!(kind, TokenKind::Backslash)) => {
                self.advance();
                let suffix = self.parse_qualified_name(false, "expected class name after 'new'")?;
                NewClassName::Named(self.resolve_relative_namespace_class_name(&suffix))
            }
            TokenKind::Variable(name) => NewClassName::DynamicVariable(name),
            _ => return Err(self.error_at(token.span, "expected class name after 'new'")),
        };
        let args = if self.match_token(|kind| matches!(kind, TokenKind::LParen)) {
            self.parse_call_arguments_after_open()?
        } else {
            Vec::new()
        };
        Ok(Expr::New {
            class_name,
            args,
            span,
        })
    }

    fn parse_include_expression(&mut self, once: bool, span: Span) -> CompileResult<Expr> {
        let path = self.parse_expression()?;
        Ok(Expr::Include {
            path: Box::new(path),
            once,
            span,
        })
    }

    fn parse_require_expression(&mut self, once: bool, span: Span) -> CompileResult<Expr> {
        let path = self.parse_expression()?;
        Ok(Expr::Require {
            path: Box::new(path),
            once,
            span,
        })
    }

    fn consume_instanceof_class_name(&mut self) -> CompileResult<String> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Backslash => {
                let raw = format!(
                    "\\{}",
                    self.parse_qualified_name(false, "expected class name after instanceof")?
                );
                Ok(self.resolve_class_like_name(&raw))
            }
            TokenKind::Identifier(name) => {
                if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
                    let raw = self.parse_qualified_name_after_first(name)?;
                    Ok(self.resolve_class_like_name(&raw))
                } else {
                    Ok(self.resolve_class_like_name(&name))
                }
            }
            TokenKind::Namespace if self.check(|kind| matches!(kind, TokenKind::Backslash)) => {
                self.advance();
                let suffix =
                    self.parse_qualified_name(false, "expected class name after instanceof")?;
                Ok(self.resolve_relative_namespace_class_name(&suffix))
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
            let first_by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
            let first = self.parse_expression()?;
            let item = if self.match_token(|kind| matches!(kind, TokenKind::FatArrow)) {
                if first_by_reference {
                    return Err(self.error_at(
                        first.span(),
                        "unsupported array reference key: reference keys are not implemented",
                    ));
                }
                let by_reference = self.match_token(|kind| matches!(kind, TokenKind::Ampersand));
                ArrayItem {
                    key: Some(first),
                    value: self.parse_expression()?,
                    by_reference,
                }
            } else {
                ArrayItem {
                    key: None,
                    value: first,
                    by_reference: first_by_reference,
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
            TokenKind::Ampersand => Ok(()),
            _ => Ok(()),
        }
    }

    fn parse_call_arguments_after_open(&mut self) -> CompileResult<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            loop {
                self.reject_unsupported_call_argument_syntax()?;
                args.push(self.parse_call_argument_after_open()?);
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

    fn parse_call_argument_after_open(&mut self) -> CompileResult<Expr> {
        if let TokenKind::Identifier(name) = self.peek().kind.clone() {
            if matches!(self.peek_next().kind, TokenKind::Colon) {
                let span = self.advance().span;
                self.consume_keyword(TokenKind::Colon, "expected ':' after named argument")?;
                let expr = self.parse_expression_with_append_read(true)?;
                return Ok(Expr::NamedArgument {
                    name,
                    expr: Box::new(expr),
                    span,
                });
            }
        }

        self.parse_expression_with_append_read(true)
    }

    fn reject_unsupported_call_argument_syntax(&self) -> CompileResult<()> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Ellipsis if matches!(self.peek_next().kind, TokenKind::RParen) => {
                Err(self.error_at(token.span, unsupported_first_class_callable_message()))
            }
            TokenKind::Ellipsis => {
                Err(self.error_at(token.span, unsupported_argument_unpacking_message()))
            }
            TokenKind::Ampersand => {
                Err(self.error_at(token.span, unsupported_reference_argument_message()))
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
            Expr::ErrorControl { .. } => Err(self.error_at(
                expr.span(),
                "default parameter values only support constant expressions in the current subset",
            )),
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
            Expr::MagicClass { .. } | Expr::MagicMethod { .. } => Ok(()),
            Expr::SelfClassConstant { .. } => Ok(()),
            Expr::Variable(_, _)
            | Expr::InterpolatedString { .. }
            | Expr::Cast { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ObjectClassNameConstant { .. }
            | Expr::ClassConstant { .. }
            | Expr::ObjectStaticClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::DynamicStaticProperty { .. }
            | Expr::ObjectStaticProperty { .. }
            | Expr::DynamicObjectStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. }
            | Expr::Index { .. }
            | Expr::AppendIndex { .. }
            | Expr::Property { .. }
            | Expr::DynamicProperty { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::InstanceOf { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. }
            | Expr::NamedArgument { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::Closure { .. }
            | Expr::Assign { .. }
            | Expr::CompoundAssign { .. }
            | Expr::NullCoalesceAssign { .. }
            | Expr::IncrementDecrement { .. }
            | Expr::Include { .. }
            | Expr::Require { .. }
            | Expr::New { .. }
            | Expr::Clone { .. } => Err(self.error_at(
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
            Expr::ErrorControl { .. } => Err(self.error_at(
                expr.span(),
                "const declaration values only support constant expressions in the current subset",
            )),
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
            Expr::MagicClass { .. } | Expr::MagicMethod { .. } => Ok(()),
            Expr::ClassConstant { .. } => Ok(()),
            Expr::Variable(_, _)
            | Expr::InterpolatedString { .. }
            | Expr::Cast { .. }
            | Expr::SelfClassNameConstant { .. }
            | Expr::ParentClassNameConstant { .. }
            | Expr::StaticClassNameConstant { .. }
            | Expr::ObjectStaticClassConstant { .. }
            | Expr::ObjectClassNameConstant { .. }
            | Expr::SelfClassConstant { .. }
            | Expr::ParentClassConstant { .. }
            | Expr::LateStaticClassConstant { .. }
            | Expr::StaticProperty { .. }
            | Expr::DynamicStaticProperty { .. }
            | Expr::ObjectStaticProperty { .. }
            | Expr::DynamicObjectStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. }
            | Expr::Index { .. }
            | Expr::AppendIndex { .. }
            | Expr::Property { .. }
            | Expr::DynamicProperty { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. }
            | Expr::InstanceOf { .. }
            | Expr::ParentMethodCall { .. }
            | Expr::StaticMethodCall { .. }
            | Expr::ObjectStaticMethodCall { .. }
            | Expr::SelfMethodCall { .. }
            | Expr::LateStaticMethodCall { .. }
            | Expr::NamedArgument { .. }
            | Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::Closure { .. }
            | Expr::Assign { .. }
            | Expr::CompoundAssign { .. }
            | Expr::NullCoalesceAssign { .. }
            | Expr::IncrementDecrement { .. }
            | Expr::Include { .. }
            | Expr::Require { .. }
            | Expr::New { .. }
            | Expr::Clone { .. } => Err(self.error_at(
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

    fn ensure_supported_instance_property_default_expr(&self, expr: &Expr) -> CompileResult<()> {
        self.ensure_supported_const_declaration_expr(expr)
            .map_err(|_error| {
                self.error_at(
                    expr.span(),
                    "instance property default values only support constant expressions in the current subset",
                )
            })
    }

    fn ensure_supported_typed_property_default_expr(
        &self,
        type_decl: &TypeDecl,
        expr: &Expr,
    ) -> CompileResult<()> {
        if type_decl.text.contains('|') {
            if type_decl.text.split('|').any(|part| {
                let part = TypeDecl {
                    text: part.trim().to_string(),
                    span: type_decl.span,
                };
                self.ensure_supported_typed_property_default_expr(&part, expr)
                    .is_ok()
            }) {
                return Ok(());
            }
            return Err(self.error_at(
                expr.span(),
                "unsupported typed property default: literal defaults must match the declared property type in the current metadata subset",
            ));
        }

        if type_decl.text.contains('&') {
            return Err(self.error_at(
                expr.span(),
                "unsupported typed property default: intersection-typed properties cannot use literal defaults in the current metadata subset",
            ));
        }

        let without_nullable = type_decl.text.strip_prefix('?').unwrap_or(&type_decl.text);
        let type_name = without_nullable
            .strip_prefix('\\')
            .unwrap_or(without_nullable)
            .to_ascii_lowercase();
        let allows_null =
            type_decl.text.starts_with('?') || type_name == "mixed" || type_name == "null";
        let compatible = match expr {
            Expr::Null(_) => allows_null,
            Expr::String(_, _) => matches!(type_name.as_str(), "string" | "mixed"),
            Expr::Int(_, _) => matches!(type_name.as_str(), "int" | "float" | "mixed"),
            Expr::Float(_, _) => matches!(type_name.as_str(), "float" | "mixed"),
            Expr::Bool(true, _) => matches!(type_name.as_str(), "bool" | "true" | "mixed"),
            Expr::Bool(false, _) => matches!(type_name.as_str(), "bool" | "false" | "mixed"),
            Expr::Array { .. } => matches!(type_name.as_str(), "array" | "mixed"),
            _ => true,
        };
        if compatible {
            Ok(())
        } else {
            Err(self.error_at(
                expr.span(),
                "unsupported typed property default: literal defaults must match the declared property type in the current metadata subset",
            ))
        }
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
            Expr::DynamicProperty {
                target, property, ..
            } => Self::expr_contains_assignment(target) || Self::expr_contains_assignment(property),
            Expr::ObjectStaticProperty { target, .. }
            | Expr::ObjectStaticClassConstant { target, .. }
            | Expr::ObjectClassNameConstant { target, .. }
            | Expr::DynamicObjectStaticProperty { target, .. } => {
                Self::expr_contains_assignment(target)
            }
            Expr::MethodCall { target, args, .. } => {
                Self::expr_contains_assignment(target)
                    || args.iter().any(Self::expr_contains_assignment)
            }
            Expr::DynamicMethodCall {
                target,
                method,
                args,
                ..
            } => {
                Self::expr_contains_assignment(target)
                    || Self::expr_contains_assignment(method)
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
            Expr::NamedArgument { expr, .. } => Self::expr_contains_assignment(expr),
            Expr::Clone { expr, .. } => Self::expr_contains_assignment(expr),
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
            Expr::Unary { expr, .. }
            | Expr::ErrorControl { expr, .. }
            | Expr::Cast { expr, .. } => Self::expr_contains_assignment(expr),
            Expr::Include { path, .. } | Expr::Require { path, .. } => {
                Self::expr_contains_assignment(path)
            }
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::InterpolatedString { .. }
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::MagicClass { .. }
            | Expr::MagicMethod { .. }
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
            | Expr::DynamicStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. }
            | Expr::IncrementDecrement { .. } => false,
        }
    }

    fn expr_contains_unsupported_assignment_rhs(expr: &Expr) -> bool {
        match expr {
            Expr::Assign { target, expr, .. } => {
                matches!(
                    target.as_ref(),
                    AssignTarget::ArrayIndex { index: None, .. }
                        | AssignTarget::NestedArrayAppend { .. }
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
            Expr::DynamicProperty {
                target, property, ..
            } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || Self::expr_contains_unsupported_assignment_rhs(property)
            }
            Expr::MethodCall { target, args, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || args
                        .iter()
                        .any(Self::expr_contains_unsupported_assignment_rhs)
            }
            Expr::DynamicMethodCall {
                target,
                method,
                args,
                ..
            } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
                    || Self::expr_contains_unsupported_assignment_rhs(method)
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
            Expr::ObjectStaticProperty { target, .. }
            | Expr::ObjectStaticClassConstant { target, .. }
            | Expr::DynamicObjectStaticProperty { target, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
            }
            Expr::ObjectClassNameConstant { target, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(target)
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
            Expr::NamedArgument { expr, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(expr)
            }
            Expr::Clone { expr, .. } => Self::expr_contains_unsupported_assignment_rhs(expr),
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
            Expr::Unary { expr, .. }
            | Expr::ErrorControl { expr, .. }
            | Expr::Cast { expr, .. } => Self::expr_contains_unsupported_assignment_rhs(expr),
            Expr::Include { path, .. } | Expr::Require { path, .. } => {
                Self::expr_contains_unsupported_assignment_rhs(path)
            }
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::InterpolatedString { .. }
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::MagicClass { .. }
            | Expr::MagicMethod { .. }
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
            | Expr::DynamicStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. }
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
            Expr::DynamicProperty {
                target, property, ..
            } => Self::find_append_index_span(target)
                .or_else(|| Self::find_append_index_span(property)),
            Expr::MethodCall { target, .. } => Self::find_append_index_span(target),
            Expr::DynamicMethodCall { target, method, .. } => Self::find_append_index_span(target)
                .or_else(|| Self::find_append_index_span(method)),
            Expr::ParentMethodCall { .. } => None,
            Expr::StaticMethodCall { .. } => None,
            Expr::ObjectStaticMethodCall { target, .. } => Self::find_append_index_span(target),
            Expr::ObjectStaticProperty { target, .. }
            | Expr::ObjectStaticClassConstant { target, .. }
            | Expr::ObjectClassNameConstant { target, .. }
            | Expr::DynamicObjectStaticProperty { target, .. } => {
                Self::find_append_index_span(target)
            }
            Expr::SelfMethodCall { .. } => None,
            Expr::LateStaticMethodCall { .. } => None,
            Expr::Call { .. } | Expr::New { .. } => None,
            Expr::NamedArgument { expr, .. } => Self::find_append_index_span(expr),
            Expr::Clone { expr, .. } => Self::find_append_index_span(expr),
            Expr::Closure { params, .. } => params
                .iter()
                .filter_map(|param| param.default.as_ref())
                .find_map(Self::find_append_index_span),
            Expr::DynamicCall { callee, .. } => Self::find_append_index_span(callee),
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
            Expr::Unary { expr, .. }
            | Expr::ErrorControl { expr, .. }
            | Expr::Cast { expr, .. } => Self::find_append_index_span(expr),
            Expr::Include { path, .. } | Expr::Require { path, .. } => {
                Self::find_append_index_span(path)
            }
            Expr::Assign { expr, .. }
            | Expr::CompoundAssign { expr, .. }
            | Expr::NullCoalesceAssign { expr, .. } => Self::find_append_index_span(expr),
            Expr::Null(_)
            | Expr::Bool(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::String(_, _)
            | Expr::InterpolatedString { .. }
            | Expr::Variable(_, _)
            | Expr::MagicLine { .. }
            | Expr::MagicFile { .. }
            | Expr::MagicDir { .. }
            | Expr::MagicFunction { .. }
            | Expr::MagicClass { .. }
            | Expr::MagicMethod { .. }
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
            | Expr::DynamicStaticProperty { .. }
            | Expr::SelfStaticProperty { .. }
            | Expr::DynamicSelfStaticProperty { .. }
            | Expr::ParentStaticProperty { .. }
            | Expr::DynamicParentStaticProperty { .. }
            | Expr::LateStaticProperty { .. }
            | Expr::DynamicLateStaticProperty { .. }
            | Expr::IncrementDecrement { .. } => None,
        }
    }

    fn consume_object_property_name(
        &mut self,
        operator_span: Span,
    ) -> CompileResult<(String, bool)> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) => Ok((name, false)),
            kind if object_property_keyword_name(&kind).is_some() => Ok((
                object_property_keyword_name(&kind)
                    .expect("checked keyword property name")
                    .to_string(),
                true,
            )),
            TokenKind::Variable(_) => unreachable!("caller handles dynamic property variables"),
            _ => Err(self.error_at(
                operator_span,
                format!(
                    "expected property name after '->', found {}",
                    token_name(&token.kind)
                ),
            )),
        }
    }

    fn parse_dynamic_property_name_expr(&mut self, _operator_span: Span) -> CompileResult<Expr> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::LBrace => {
                let expr = if self.check(|kind| matches!(kind, TokenKind::Dollar)) {
                    let dollar = self.advance().clone();
                    match self.advance().clone() {
                        Token {
                            kind: TokenKind::Variable(name),
                            span,
                        } => Expr::Variable(name, span),
                        token => {
                            return Err(self.error_at(
                                dollar.span,
                                format!(
                                    "unsupported variable variable: expected variable after '$', found {}",
                                    token_name(&token.kind)
                                ),
                            ));
                        }
                    }
                } else {
                    self.parse_expression()?
                };
                self.consume_keyword(
                    TokenKind::RBrace,
                    "expected '}' after dynamic property expression",
                )?;
                Ok(expr)
            }
            _ => unreachable!("caller checked dynamic property variable"),
        }
    }

    fn parse_computed_static_property_name_expr(
        &mut self,
        operator_span: Span,
    ) -> CompileResult<Expr> {
        if self.match_token(|kind| matches!(kind, TokenKind::Dollar)) {
            self.consume_keyword(
                TokenKind::LBrace,
                "expected '{' after '$' in computed static property name",
            )?;
            let expr = if self.check(|kind| matches!(kind, TokenKind::Dollar)) {
                let dollar = self.advance().clone();
                match self.advance().clone() {
                    Token {
                        kind: TokenKind::Variable(name),
                        span,
                    } => Expr::Variable(name, span),
                    token => {
                        return Err(self.error_at(
                            dollar.span,
                            format!(
                                "unsupported variable variable: expected variable after '$', found {}",
                                token_name(&token.kind)
                            ),
                        ));
                    }
                }
            } else {
                self.parse_expression()?
            };
            self.consume_keyword(
                TokenKind::RBrace,
                "expected '}' after computed static property name",
            )?;
            Ok(expr)
        } else {
            self.parse_dynamic_property_name_expr(operator_span)
        }
    }

    fn consume_identifier(&mut self, message: &str) -> CompileResult<String> {
        self.consume_identifier_with_span(message)
            .map(|(name, _span)| name)
    }

    fn parse_qualified_name(
        &mut self,
        allow_leading_backslash: bool,
        message: &str,
    ) -> CompileResult<String> {
        self.parse_qualified_name_with_span(allow_leading_backslash, message)
            .map(|(name, _span)| name)
    }

    fn parse_qualified_name_with_span(
        &mut self,
        allow_leading_backslash: bool,
        message: &str,
    ) -> CompileResult<(String, Span)> {
        let mut name = String::new();
        let mut leading = false;
        let span = if self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            if !allow_leading_backslash {
                return Err(self.error_at(self.previous().span, message));
            }
            leading = true;
            self.previous().span
        } else {
            self.peek().span
        };

        let first = self.consume_identifier(message)?;
        if leading {
            name.push('\\');
        }
        name.push_str(&first);

        while self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            name.push('\\');
            name.push_str(&self.consume_identifier(message)?);
        }

        Ok((name, span))
    }

    fn parse_qualified_name_after_first(&mut self, first: String) -> CompileResult<String> {
        let mut name = first;
        while self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
            name.push('\\');
            name.push_str(&self.consume_identifier("expected name segment after '\\'")?);
        }
        Ok(name)
    }

    fn consume_class_like_name(&mut self, message: &str) -> CompileResult<String> {
        if self.check(|kind| matches!(kind, TokenKind::Namespace)) {
            let span = self.advance().span;
            if !self.match_token(|kind| matches!(kind, TokenKind::Backslash)) {
                return Err(self.error_at(span, message));
            }
            let suffix = self.parse_qualified_name(false, message)?;
            return Ok(self.resolve_relative_namespace_class_name(&suffix));
        }

        if self.check(|kind| matches!(kind, TokenKind::Backslash)) {
            let raw = self.parse_qualified_name(true, message)?;
            return Ok(self.resolve_class_like_name(&raw));
        }

        let raw = self.parse_qualified_name(false, message)?;
        Ok(self.resolve_class_like_name(&raw))
    }

    fn resolve_declared_class_name(&self, name: &str) -> String {
        if self.current_namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, name)
        }
    }

    fn resolve_function_declaration_name(&self, name: &str) -> String {
        if self.current_namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, name)
        }
    }

    fn resolve_constant_declaration_name(&self, name: &str) -> String {
        if self.current_namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, name)
        }
    }

    fn resolve_constant_read_name(&self, name: &str) -> String {
        if let Some((_, imported)) = self
            .constant_imports
            .iter()
            .find(|(alias, _)| alias == name)
        {
            return imported.clone();
        }

        if self.current_namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, name)
        }
    }

    fn resolve_function_call_name(&self, name: &str) -> String {
        if let Some((_, imported)) = self
            .function_imports
            .iter()
            .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        {
            return imported.clone();
        }

        if self.current_namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, name)
        }
    }

    fn resolve_relative_namespace_class_name(&self, suffix: &str) -> String {
        if self.current_namespace.is_empty() {
            suffix.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, suffix)
        }
    }

    fn resolve_class_like_name(&self, raw: &str) -> String {
        if let Some(stripped) = raw.strip_prefix('\\') {
            return stripped.to_string();
        }

        let (first, rest) = raw.split_once('\\').unwrap_or((raw, ""));
        if let Some((_, imported)) = self
            .class_imports
            .iter()
            .find(|(alias, _)| alias.eq_ignore_ascii_case(first))
        {
            if rest.is_empty() {
                return imported.clone();
            }
            return format!("{imported}\\{rest}");
        }

        if self.current_namespace.is_empty() {
            raw.to_string()
        } else {
            format!("{}\\{}", self.current_namespace, raw)
        }
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

    fn consume_trait_adaptation_as(&mut self) -> CompileResult<()> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("as") => Ok(()),
            _ => Err(self.error_at(token.span, "expected 'as' in trait method alias adaptation")),
        }
    }

    fn consume_trait_adaptation_insteadof(&mut self) -> CompileResult<()> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("insteadof") => Ok(()),
            _ => Err(self.error_at(
                token.span,
                "expected 'insteadof' in trait method precedence adaptation",
            )),
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

    fn match_exception_keyword(&mut self, expected: &str) -> Option<Span> {
        if self.check_exception_keyword(expected) {
            Some(self.advance().span)
        } else {
            None
        }
    }

    fn check_exception_keyword(&self, expected: &str) -> bool {
        match (&self.peek().kind, expected) {
            (TokenKind::Try, "try")
            | (TokenKind::Catch, "catch")
            | (TokenKind::Finally, "finally") => true,
            (TokenKind::Identifier(name), _) => name.eq_ignore_ascii_case(expected),
            _ => false,
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

    fn skip_doc_comments_before(&mut self, terminal: fn(&TokenKind) -> bool) -> bool {
        if !matches!(self.peek().kind, TokenKind::DocComment(_)) {
            return false;
        }
        let mut offset = 0;
        while matches!(self.peek_n(offset).kind, TokenKind::DocComment(_)) {
            offset += 1;
        }
        if !terminal(&self.peek_n(offset).kind) {
            return false;
        }
        for _ in 0..offset {
            self.advance();
        }
        true
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

    fn trace_parse(&self, context: &str) {
        if !self.trace_parse {
            return;
        }
        let token = self.peek();
        eprintln!(
            "phpc trace parse: {context} at {}:{} token {}",
            token.span.line,
            token.span.column,
            token_name(&token.kind)
        );
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
        TokenKind::Dollar => "$",
        TokenKind::Variable(_) => "variable",
        TokenKind::Identifier(_) => "identifier",
        TokenKind::Int(_) => "integer literal",
        TokenKind::Float(_) => "float literal",
        TokenKind::StringLiteral(_) => "string literal",
        TokenKind::InterpolatedString(_) => "interpolated string literal",
        TokenKind::DocComment(_) => "doc comment",
        TokenKind::InlineHtml(_) => "inline HTML",
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
        TokenKind::NullsafeObjectOperator => "?->",
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
        TokenKind::At => "@",
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

fn object_property_keyword_name(kind: &TokenKind) -> Option<&'static str> {
    match kind {
        TokenKind::Echo => Some("echo"),
        TokenKind::Print => Some("print"),
        TokenKind::Function => Some("function"),
        TokenKind::Fn => Some("fn"),
        TokenKind::Class => Some("class"),
        TokenKind::Interface => Some("interface"),
        TokenKind::Trait => Some("trait"),
        TokenKind::Enum => Some("enum"),
        TokenKind::Abstract => Some("abstract"),
        TokenKind::Final => Some("final"),
        TokenKind::Readonly => Some("readonly"),
        TokenKind::New => Some("new"),
        TokenKind::Public => Some("public"),
        TokenKind::Protected => Some("protected"),
        TokenKind::Private => Some("private"),
        TokenKind::Static => Some("static"),
        TokenKind::Extends => Some("extends"),
        TokenKind::Implements => Some("implements"),
        TokenKind::Clone => Some("clone"),
        TokenKind::Instanceof => Some("instanceof"),
        TokenKind::Return => Some("return"),
        TokenKind::Global => Some("global"),
        TokenKind::Namespace => Some("namespace"),
        TokenKind::Use => Some("use"),
        TokenKind::Declare => Some("declare"),
        TokenKind::Eval => Some("eval"),
        TokenKind::Include => Some("include"),
        TokenKind::IncludeOnce => Some("include_once"),
        TokenKind::Require => Some("require"),
        TokenKind::RequireOnce => Some("require_once"),
        TokenKind::If => Some("if"),
        TokenKind::Else => Some("else"),
        TokenKind::ElseIf => Some("elseif"),
        TokenKind::While => Some("while"),
        TokenKind::Do => Some("do"),
        TokenKind::Foreach => Some("foreach"),
        TokenKind::For => Some("for"),
        TokenKind::Switch => Some("switch"),
        TokenKind::Match => Some("match"),
        TokenKind::Break => Some("break"),
        TokenKind::Continue => Some("continue"),
        TokenKind::Throw => Some("throw"),
        TokenKind::Try => Some("try"),
        TokenKind::Catch => Some("catch"),
        TokenKind::Finally => Some("finally"),
        TokenKind::Null => Some("null"),
        TokenKind::True => Some("true"),
        TokenKind::False => Some("false"),
        _ => None,
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
            | TokenKind::LParen
            | TokenKind::Backslash
            | TokenKind::Static
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False
    )
}

fn is_promoted_property_parameter_start(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Public | TokenKind::Protected | TokenKind::Private | TokenKind::Readonly
    )
}

fn unsupported_promoted_property_parameter_message() -> &'static str {
    "unsupported promoted property parameter: constructor property promotion is not implemented"
}

fn unsupported_parameter_type_message() -> &'static str {
    "unsupported parameter type declaration: parameter type enforcement is not implemented"
}

fn unsupported_return_type_message() -> &'static str {
    "unsupported return type declaration: return type enforcement is not implemented"
}

fn unsupported_property_type_message() -> &'static str {
    "unsupported property type declaration: property type metadata supports only simple named property types in the current subset"
}

fn unsupported_readonly_property_message() -> &'static str {
    "unsupported readonly property declaration: readonly property metadata, initialization rules, write-once enforcement, reflection, and native lowering are not implemented"
}

fn unsupported_readonly_class_message() -> &'static str {
    "unsupported readonly class declaration: readonly class metadata, typed-property enforcement, initialization and write rules, reflection, and native lowering are not implemented"
}

fn unsupported_dnf_type_message() -> &'static str {
    "unsupported DNF type declaration: parenthesized union/intersection type declarations are not implemented"
}

fn unsupported_multiple_properties_message() -> &'static str {
    "unsupported property declaration: multiple properties in one declaration are not implemented"
}

fn unsupported_property_hook_message() -> &'static str {
    "unsupported property hook declaration: PHP property get/set hooks require hook metadata, backing/virtual property behavior, typed-property storage and enforcement, references, reflection, and native lowering"
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
    if name == "__TRAIT__" {
        return "unsupported magic constant __TRAIT__: trait context evaluation requires original trait method context tracking through class composition, which is not implemented".to_string();
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

fn unsupported_yield_from_message() -> &'static str {
    "unsupported yield from expression: generator delegation requires Traversable iteration, yielded key/value forwarding, send/throw propagation, generator return values, references/copy-on-write, and native lowering"
}

fn unsupported_match_expression_message() -> &'static str {
    "unsupported match expression: strict arm matching, default/exhaustiveness handling, throw arms, value evaluation order, references/copy-on-write, and native lowering are not implemented"
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

fn unsupported_nullsafe_object_operator_message() -> &'static str {
    "unsupported nullsafe object operator: ?-> property and method access is not implemented"
}

fn unsupported_exponentiation_message() -> &'static str {
    "unsupported exponentiation operator: ** and **= are not implemented"
}

fn unsupported_null_coalescing_assignment_message() -> &'static str {
    "unsupported null coalescing assignment: only direct variable, direct or nested array-offset, direct object-property, static-property, and direct object-property array-offset targets are implemented"
}

fn unsupported_assignment_expression_message() -> &'static str {
    "unsupported assignment expression: this assignment form is not implemented in the current expression context"
}

fn unsupported_assignment_expression_target_message() -> &'static str {
    "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented"
}

fn unsupported_chained_assignment_expression_message() -> &'static str {
    "unsupported assignment expression: this chained assignment form is not implemented in the current subset"
}

fn unsupported_compound_assignment_target_message() -> &'static str {
    "unsupported compound assignment target: only direct static variables, direct array offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented; append offsets and nested variable targets are not implemented"
}

fn unsupported_increment_decrement_expression_message() -> &'static str {
    "unsupported increment/decrement expression: chained increment/decrement expressions are not implemented"
}

fn unsupported_increment_decrement_target_message() -> &'static str {
    "unsupported increment/decrement target: only direct static variables, direct array/object offsets, append offsets, direct object properties, direct object-property array offsets, and supported static properties are implemented for integer, float, and append-null values; append suffixes and nested variable targets are not implemented"
}

fn unsupported_bracketed_namespace_message() -> &'static str {
    "unsupported namespace declaration: bracketed namespace blocks are not implemented"
}

fn unsupported_multiple_namespace_message() -> &'static str {
    "unsupported namespace declaration: multiple namespace declarations are not implemented"
}

fn unsupported_nested_namespace_message() -> &'static str {
    "unsupported namespace declaration: namespace declarations are only implemented at file scope"
}

fn unsupported_use_message() -> &'static str {
    "unsupported use declaration: only simple class imports are implemented"
}

fn unsupported_multiple_class_use_message() -> &'static str {
    "unsupported multiple class use declaration: multiple simple class imports in one use declaration require import-list metadata, alias handling, namespace resolution, and native lowering"
}

fn function_import_alias_conflict_message() -> &'static str {
    "unsupported function use declaration: imported function alias conflicts with an existing function declaration or import in the same namespace"
}

fn function_declaration_import_conflict_message() -> &'static str {
    "unsupported function declaration: function name conflicts with an imported function alias in the same namespace"
}

fn constant_import_alias_conflict_message() -> &'static str {
    "unsupported const use declaration: imported constant alias conflicts with an existing constant declaration or import in the same namespace"
}

fn constant_declaration_import_conflict_message() -> &'static str {
    "unsupported const declaration: constant name conflicts with an imported constant alias in the same namespace"
}

fn unsupported_grouped_use_message() -> &'static str {
    "unsupported grouped use declaration: grouped class, function, and const imports are not implemented"
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

fn unsupported_fully_qualified_function_call_message() -> &'static str {
    "unsupported fully-qualified function call: leading global namespace function calls require exact function-table lookup, namespace fallback bypass, builtin/user dispatch, and native lowering"
}

fn unsupported_namespace_qualified_constant_name_message() -> &'static str {
    "unsupported namespace-qualified constant name: namespace-aware constant lookup, fallback behavior, constant imports, and native lowering are not implemented"
}

fn unsupported_fully_qualified_constant_name_message() -> &'static str {
    "unsupported fully-qualified constant name: leading global namespace constant reads require exact constant-table lookup, namespace fallback bypass, import interaction, and native lowering"
}

fn unsupported_array_spread_message() -> &'static str {
    "unsupported array spread: spread elements are not implemented"
}

fn unsupported_array_destructuring_assignment_message() -> &'static str {
    "unsupported array destructuring: only positional statement-form list($a, $b) = expr and [$a, $b] = expr targets with variable or skipped slots are implemented; expression-position list(...), nested, keyed, reference, and non-variable targets are not implemented"
}

fn unsupported_reference_assignment_source_message() -> &'static str {
    "unsupported reference assignment: only direct variable, direct/nested/append array-offset, expression-root array-offset/append, direct/nested/append object-property array-offset, bounded non-direct object-property array-offset, object-property, static property, direct/dynamic function-call, and method-call reference sources are parsed before reference semantics exist"
}

fn unsupported_first_class_callable_message() -> &'static str {
    "unsupported first-class callable syntax: Closure creation with ... is not implemented"
}

fn unsupported_argument_unpacking_message() -> &'static str {
    "unsupported argument unpacking: call-site ... expansion requires iterable unpacking order, string-keyed named-argument interaction, by-reference argument propagation, variadic collection, duplicate argument diagnostics, and native lowering"
}

fn unsupported_reference_argument_message() -> &'static str {
    "unsupported call-time by-reference argument: passing & at a call site requires legacy syntax handling, by-reference parameter metadata, alias setup, default handling, variadic/unpacking interaction, references/copy-on-write, and native lowering"
}

fn unsupported_named_argument_message() -> &'static str {
    "unsupported named argument: call argument names require parameter-name metadata, duplicate and unknown-name diagnostics, positional/named ordering, by-reference binding, variadic collection, unpacking interaction, and native lowering"
}

fn unsupported_unset_message() -> &'static str {
    "unsupported unset: supported operands are direct variables, direct/nested array offsets, direct or bounded non-direct object properties and object-property array offsets, and direct static properties; append unset, object operators after array offsets, and broader dynamic expression roots are not implemented"
}

fn unsupported_foreach_expression_message() -> &'static str {
    "unsupported foreach: foreach is only supported as a statement in the current subset"
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

fn unsupported_switch_expression_message() -> &'static str {
    "unsupported switch: switch is only supported as a statement in the current subset"
}

fn unsupported_if_alternate_message() -> &'static str {
    "unsupported if: alternate if/elseif/else colon/endif syntax is not implemented; use brace blocks or single-statement bodies"
}

fn unsupported_break_depth_message() -> &'static str {
    "unsupported break: only positive integer loop-depth literals are implemented in the current subset"
}

fn unsupported_break_expression_message() -> &'static str {
    "unsupported break: break is only supported as a statement in the current subset"
}

fn unsupported_continue_depth_message() -> &'static str {
    "unsupported continue: only positive integer loop-depth literals are implemented in the current subset"
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

fn unsupported_nested_trait_declaration_message() -> &'static str {
    "unsupported trait declaration: only top-level trait declarations are implemented"
}

fn unsupported_trait_member_declaration_message() -> &'static str {
    "unsupported trait member declaration: nested trait use adaptations and unsupported trait members are not implemented"
}

fn unsupported_trait_method_message() -> &'static str {
    "unsupported trait method declaration: only simple public instance and public static trait methods are implemented; abstract, final, non-public methods, __TRAIT__ context, references/copy-on-write, and native lowering remain unsupported"
}

fn unsupported_interface_declaration_message() -> &'static str {
    "unsupported interface declaration: interface parsing and implementation execution are not implemented"
}

fn unsupported_nested_interface_declaration_message() -> &'static str {
    "unsupported interface declaration: only top-level interface declarations are implemented"
}

fn unsupported_interface_method_visibility_message() -> &'static str {
    "unsupported interface method declaration: only public interface methods are implemented"
}

fn unsupported_interface_method_body_message() -> &'static str {
    "unsupported interface method declaration: interface methods cannot have bodies in the current subset"
}

fn unsupported_enum_declaration_message() -> &'static str {
    "unsupported enum declaration: enum parsing and case/value execution are not implemented"
}

fn unsupported_nested_enum_declaration_message() -> &'static str {
    "unsupported enum declaration: only top-level enum declarations are implemented"
}

fn unsupported_backed_enum_message() -> &'static str {
    "unsupported backed enum declaration: backed enum values and scalar backing types are not implemented"
}

fn unsupported_enum_implementation_message() -> &'static str {
    "unsupported enum interface implementation: enum implements clauses are not implemented"
}

fn unsupported_enum_member_message() -> &'static str {
    "unsupported enum member declaration: only unbacked enum case declarations are implemented"
}

fn unsupported_enum_case_value_message() -> &'static str {
    "unsupported enum case value: backed enum case values are not implemented"
}

fn unsupported_clone_message() -> &'static str {
    "unsupported clone expression: object handle copying and __clone dispatch are not implemented"
}

fn unsupported_instanceof_message() -> &'static str {
    "unsupported instanceof expression: class/interface relationship checks are not implemented"
}

fn is_magic_static_receiver(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "self" | "parent" | "static"
    )
}

fn same_token_kind(left: &TokenKind, right: &TokenKind) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
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
        TokenKind::Readonly => unsupported_readonly_class_member_modifier_message().to_string(),
        TokenKind::Abstract | TokenKind::Final => {
            unsupported_class_member_modifier_message().to_string()
        }
        _ => format!("expected class member, found {}", token_name(kind)),
    }
}

fn unsupported_class_member_modifier_message() -> &'static str {
    "unsupported class member modifier: abstract, final, and readonly member modifiers are not implemented"
}

fn unsupported_abstract_final_property_message() -> &'static str {
    "unsupported abstract/final property declaration: abstract and final property modifiers are not implemented"
}

fn unsupported_abstract_final_class_constant_message() -> &'static str {
    "unsupported abstract/final class constant declaration: abstract and final class constant modifiers are not implemented"
}

fn unsupported_readonly_class_member_modifier_message() -> &'static str {
    "unsupported readonly class member modifier: readonly methods and readonly class constants are not implemented"
}

fn unsupported_asymmetric_property_visibility_message() -> &'static str {
    "unsupported asymmetric property visibility: PHP 8 set-visibility modifiers such as private(set) and protected(set) require property visibility metadata, typed-property storage and enforcement, reflection behavior, and native lowering"
}

fn unsupported_trait_use_message() -> &'static str {
    "unsupported trait use: class-body trait use is implemented only for already-declared traits with public instance methods and simple method aliases"
}

impl Parser {
    fn check_trait_method_declaration(&self) -> bool {
        for token in self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
        {
            match &token.kind {
                TokenKind::Function => return true,
                TokenKind::Variable(_) => return false,
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return false,
                _ => {}
            }
        }
        false
    }

    fn check_trait_constant_declaration(&self) -> bool {
        self.check_class_like_constant_declaration()
    }

    fn check_trait_property_declaration(&self) -> bool {
        for token in self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
        {
            match &token.kind {
                TokenKind::Variable(_) => return true,
                TokenKind::Function => return false,
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return false,
                _ => {}
            }
        }
        false
    }

    fn check_interface_constant_declaration(&self) -> bool {
        self.check_class_like_constant_declaration()
    }

    fn check_class_like_constant_declaration(&self) -> bool {
        let mut index = self.current;
        while index < self.tokens.len() {
            match &self.tokens[index].kind {
                TokenKind::Public
                | TokenKind::Protected
                | TokenKind::Private
                | TokenKind::Static
                | TokenKind::Abstract
                | TokenKind::Final => {
                    index += 1;
                }
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return true,
                _ => return false,
            }
        }
        false
    }

    fn match_trait_visibility_adaptation(&mut self) -> Option<ClassVisibility> {
        let visibility = match &self.peek().kind {
            TokenKind::Public => ClassVisibility::Public,
            TokenKind::Protected => ClassVisibility::Protected,
            TokenKind::Private => ClassVisibility::Private,
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("public") => {
                ClassVisibility::Public
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("protected") => {
                ClassVisibility::Protected
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("private") => {
                ClassVisibility::Private
            }
            _ => return None,
        };
        self.advance();
        Some(visibility)
    }

    fn check_asymmetric_property_visibility_modifier(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Public | TokenKind::Protected | TokenKind::Private
        ) && matches!(self.peek_next().kind, TokenKind::LParen)
            && matches!(
                &self.peek_n(2).kind,
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("set")
            )
            && matches!(self.peek_n(3).kind, TokenKind::RParen)
    }

    fn check_readonly_property_declaration(&self) -> bool {
        for token in self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
        {
            match &token.kind {
                TokenKind::Variable(_) => return true,
                TokenKind::Function => return false,
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return false,
                _ => {}
            }
        }
        false
    }

    fn check_unsupported_property_type_declaration(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("const") => return false,
            TokenKind::Identifier(_)
            | TokenKind::Question
            | TokenKind::LParen
            | TokenKind::Backslash
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False => {}
            _ => return false,
        }

        self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
            .any(|token| matches!(token.kind, TokenKind::Variable(_)))
    }

    fn property_hook_span_before_member_end(&self) -> Option<Span> {
        let mut saw_variable = false;
        for token in self.tokens[self.current..]
            .iter()
            .take_while(|token| !matches!(token.kind, TokenKind::Semicolon | TokenKind::RBrace))
        {
            match &token.kind {
                TokenKind::Variable(_) => saw_variable = true,
                TokenKind::LBrace if saw_variable => return Some(token.span),
                _ => {}
            }
        }
        None
    }
}
