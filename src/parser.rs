use std::collections::{HashMap, HashSet};

use crate::ast::{
    AnonymousFunction, ArrayDimTarget, ArrayElement, ArrayElementValue, AssignmentOp,
    AssignmentTarget, AttributeMetadata, BinaryOp, CastKind, CatchClause, ClassConstantDecl,
    ClassDecl, ClosureUseCapture, ConstDeclaration, Expr, FunctionDecl, FunctionParameter,
    IncDecOp, IncDecResult, IncDecTarget, IncludeKind, InstanceOfTarget, ListAssignmentElement,
    ListAssignmentElementTarget, ListAssignmentTarget, ListExpr, ListExprElement,
    ListExprElementTarget, MagicConstantKind, MatchArm, MethodDecl, Program, PromotedProperty,
    PropertyDecl, PropertyTypeHint, PropertyTypeKind, PropertyVisibility, ReferenceTarget,
    Statement, StaticLocalDeclaration, StaticPropertyDecl, StringInterpolationIndex, StringPart,
    SwitchCase, TraitDecl, TraitUseDecl, TypeHint, UnaryOp, UnsetTarget,
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
const PIPE_PRECEDENCE: u8 = 4;
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
        source,
        tokens,
        index: 0,
        block_depth: 0,
        function_depth: 0,
        current_namespace: None,
        seen_namespace_declaration: false,
        namespace_declaration_style: None,
        class_aliases: HashMap::new(),
        function_aliases: HashMap::new(),
        constant_aliases: HashMap::new(),
        runtime_class_aliases: HashMap::new(),
        declared_functions: HashSet::new(),
        declared_class_names: HashMap::new(),
        anonymous_classes: Vec::new(),
        nested_functions: Vec::new(),
        anonymous_class_name_counts: HashMap::new(),
        allow_append_array_read: false,
        active_type_scope: None,
        allow_unscoped_relative_types: 0,
        return_by_ref_stack: Vec::new(),
        strict_types: false,
    }
    .parse_program()
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    index: usize,
    block_depth: usize,
    function_depth: usize,
    current_namespace: Option<String>,
    seen_namespace_declaration: bool,
    namespace_declaration_style: Option<NamespaceDeclarationStyle>,
    class_aliases: HashMap<String, String>,
    function_aliases: HashMap<String, String>,
    constant_aliases: HashMap<String, String>,
    runtime_class_aliases: HashMap<String, String>,
    declared_functions: HashSet<String>,
    declared_class_names: HashMap<String, SourceSpan>,
    anonymous_classes: Vec<ClassDecl>,
    nested_functions: Vec<FunctionDecl>,
    anonymous_class_name_counts: HashMap<String, usize>,
    allow_append_array_read: bool,
    active_type_scope: Option<ActiveTypeScope>,
    allow_unscoped_relative_types: usize,
    return_by_ref_stack: Vec<bool>,
    strict_types: bool,
}

#[derive(Debug, Clone)]
struct ActiveTypeScope {
    class_name: String,
    parent_name: Option<String>,
    allow_unresolved_parent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamespaceDeclarationStyle {
    Bracketed,
    Unbracketed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopLevelScope {
    Program,
    NamespaceBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameResolution {
    Unqualified,
    Qualified,
    FullyQualified,
    NamespaceRelative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedName {
    name: String,
    span: SourceSpan,
    resolution: NameResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UseDeclarationKind {
    Class,
    Function,
    Constant,
}

struct ForeachVariable {
    target: AssignmentTarget,
    by_ref: bool,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
struct ParsedTypeHint {
    type_hint: TypeHint,
    span: SourceSpan,
}

#[derive(Debug, Clone, Copy)]
enum TypeHintContext {
    Parameter,
    Return,
}

impl TypeHintContext {
    fn allows_void(self) -> bool {
        matches!(self, Self::Return)
    }

    fn allows_never(self) -> bool {
        matches!(self, Self::Return)
    }
}

type ParsedAttributes = AttributeMetadata;

#[derive(Clone, Copy)]
struct ClassModifiers {
    is_static: bool,
    is_abstract: bool,
    is_final: bool,
    is_readonly: bool,
    static_span: Option<SourceSpan>,
    abstract_span: Option<SourceSpan>,
    final_span: Option<SourceSpan>,
    readonly_span: Option<SourceSpan>,
    visibility: PropertyVisibility,
    visibility_span: Option<SourceSpan>,
    set_visibility: Option<PropertyVisibility>,
    set_visibility_span: Option<SourceSpan>,
}

impl Default for ClassModifiers {
    fn default() -> Self {
        Self {
            is_static: false,
            is_abstract: false,
            is_final: false,
            static_span: None,
            abstract_span: None,
            final_span: None,
            is_readonly: false,
            readonly_span: None,
            visibility: PropertyVisibility::Public,
            visibility_span: None,
            set_visibility: None,
            set_visibility_span: None,
        }
    }
}

impl ClassModifiers {
    fn has_promoted_property_modifier(&self) -> bool {
        self.visibility_span.is_some() || self.set_visibility_span.is_some() || self.is_readonly
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VirtualPropertyHookKind {
    GetOnly,
    SetOnly,
    Other,
}

enum ParsedClassMember {
    Method(MethodDecl),
    Properties(Vec<PropertyDecl>),
    StaticProperties(Vec<StaticPropertyDecl>),
    Constants(Vec<ClassConstantDecl>),
    TraitUses(Vec<TraitUseDecl>),
}

impl Parser<'_> {
    fn parse_program(&mut self) -> Result<Program> {
        if matches!(self.peek().kind, TokenKind::OpenTag) {
            self.expect_open_tag()?;
        } else if !matches!(self.peek().kind, TokenKind::InlineHtml(_) | TokenKind::Eof) {
            return Err(Diagnostic::new(
                "expected <?php open tag",
                Some(self.peek().span),
            ));
        }
        let mut classes = Vec::new();
        let mut traits = Vec::new();
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        self.parse_top_level_items(
            &mut classes,
            &mut traits,
            &mut functions,
            &mut statements,
            TopLevelScope::Program,
        )?;
        functions.append(&mut self.nested_functions);
        classes.append(&mut self.anonymous_classes);
        compose_traits(&mut traits)?;
        compose_class_traits(&mut classes, &traits)?;
        validate_class_names(&classes, &traits)?;
        validate_trait_names(&traits)?;
        validate_builtin_attribute_targets(&classes, &traits, &functions)?;
        validate_parent_class_names(&classes)?;
        validate_interface_references(&classes)?;
        validate_override_attributes(&classes, &traits)?;
        validate_traversable_implementations(&classes)?;
        validate_method_signature_compatibility(&classes)?;
        validate_abstract_methods(&classes)?;
        validate_final_class_inheritance(&classes)?;
        validate_readonly_class_inheritance(&classes)?;
        validate_property_override_set_visibility(&classes)?;
        validate_class_scoped_constant_exprs(&classes)?;
        for class in &classes {
            validate_method_names(class)?;
            validate_property_names(class)?;
            validate_class_constant_names(class)?;
            for method in &class.methods {
                if method.return_by_ref {
                    validate_by_reference_returns_in_statements(
                        &method.body,
                        &format!("{}::{}", class.name, method.name),
                    )?;
                }
                if matches!(&method.return_type, Some(TypeHint::Void)) {
                    validate_void_returns_in_statements(&method.body)?;
                }
                validate_anonymous_functions_in_statements(&method.body, &functions)?;
                validate_reference_assignment_sources(&method.body, &functions)?;
                validate_control_transfers_in_statements(&method.body, 0)?;
                validate_goto_labels(&method.body)?;
            }
        }
        validate_function_names(&functions)?;
        validate_by_reference_returns(&functions)?;
        validate_void_returns(&functions)?;
        validate_anonymous_functions_in_statements(&statements, &functions)?;
        validate_reference_assignment_sources(&statements, &functions)?;
        validate_control_transfers_in_statements(&statements, 0)?;
        for function in &functions {
            validate_anonymous_functions_in_statements(&function.body, &functions)?;
            validate_reference_assignment_sources(&function.body, &functions)?;
            validate_control_transfers_in_statements(&function.body, 0)?;
            validate_goto_labels(&function.body)?;
        }
        validate_goto_labels(&statements)?;
        Ok(Program {
            classes,
            traits,
            functions,
            statements,
            strict_types: self.strict_types,
        })
    }

    fn parse_top_level_items(
        &mut self,
        classes: &mut Vec<ClassDecl>,
        traits: &mut Vec<TraitDecl>,
        functions: &mut Vec<FunctionDecl>,
        statements: &mut Vec<Statement>,
        scope: TopLevelScope,
    ) -> Result<()> {
        while !matches!(self.peek().kind, TokenKind::Eof) {
            self.skip_php_tags();
            if matches!(self.peek().kind, TokenKind::Eof)
                || (scope == TopLevelScope::NamespaceBlock
                    && matches!(self.peek().kind, TokenKind::RightBrace))
            {
                break;
            }
            let attributes = self.parse_attribute_groups()?;
            if token_is_identifier_named(self.peek(), "declare") {
                let statement = self.parse_declare_statement()?;
                if !matches!(statement, Statement::Empty { .. }) {
                    self.reject_code_outside_bracketed_namespace(scope)?;
                    statements.push(statement);
                }
            } else if token_is_identifier_named(self.peek(), "namespace") {
                if scope == TopLevelScope::NamespaceBlock {
                    return Err(Diagnostic::new(
                        "Namespace declarations cannot be nested",
                        Some(self.peek().span),
                    ));
                }
                if !self.seen_namespace_declaration
                    && (!classes.is_empty()
                        || !functions.is_empty()
                        || statements
                            .iter()
                            .any(|statement| !matches!(statement, Statement::Empty { .. })))
                {
                    return Err(Diagnostic::new(
                        "Namespace declaration statement has to be the very first statement or after any declare call in the script",
                        Some(self.peek().span),
                    ));
                }
                self.parse_namespace_declaration(classes, traits, functions, statements)?;
            } else if token_is_identifier_named(self.peek(), "use") {
                self.reject_code_outside_bracketed_namespace(scope)?;
                self.parse_use_declarations()?;
            } else if self.peek_starts_function_decl() {
                self.reject_code_outside_bracketed_namespace(scope)?;
                functions.push(self.parse_function_decl(attributes)?);
            } else if token_is_identifier_named(self.peek(), "trait") {
                self.reject_code_outside_bracketed_namespace(scope)?;
                traits.push(self.parse_trait_decl(attributes)?);
            } else if self.peek_starts_class_decl() {
                self.reject_code_outside_bracketed_namespace(scope)?;
                let class = self.parse_class_decl(attributes)?;
                self.register_declared_class_name(&class)?;
                classes.push(class);
            } else {
                self.reject_code_outside_bracketed_namespace(scope)?;
                let statement = self.parse_statement()?;
                self.note_runtime_class_alias_statement(&statement);
                statements.push(statement);
            }
        }
        Ok(())
    }

    fn reject_code_outside_bracketed_namespace(&self, scope: TopLevelScope) -> Result<()> {
        if scope == TopLevelScope::Program
            && self.namespace_declaration_style == Some(NamespaceDeclarationStyle::Bracketed)
        {
            return Err(Diagnostic::new(
                "No code may exist outside of namespace {}",
                Some(self.peek().span),
            ));
        }
        Ok(())
    }

    fn parse_namespace_declaration(
        &mut self,
        classes: &mut Vec<ClassDecl>,
        traits: &mut Vec<TraitDecl>,
        functions: &mut Vec<FunctionDecl>,
        statements: &mut Vec<Statement>,
    ) -> Result<()> {
        let namespace_span = self.advance().span;
        let namespace = if matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::LeftBrace
        ) {
            None
        } else {
            Some(self.parse_namespace_name()?)
        };
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            self.note_namespace_declaration_style(
                NamespaceDeclarationStyle::Bracketed,
                namespace_span,
            )?;
            self.seen_namespace_declaration = true;
            return self.parse_bracketed_namespace_block(
                namespace, classes, traits, functions, statements,
            );
        }
        self.note_namespace_declaration_style(
            NamespaceDeclarationStyle::Unbracketed,
            namespace_span,
        )?;
        self.expect_semicolon()?;
        self.current_namespace = namespace;
        self.seen_namespace_declaration = true;
        self.clear_namespace_imports();
        Ok(())
    }

    fn note_namespace_declaration_style(
        &mut self,
        style: NamespaceDeclarationStyle,
        span: SourceSpan,
    ) -> Result<()> {
        if let Some(existing) = self.namespace_declaration_style {
            if existing != style {
                return Err(Diagnostic::new(
                    "Cannot mix bracketed namespace declarations with unbracketed namespace declarations",
                    Some(span),
                ));
            }
        } else {
            self.namespace_declaration_style = Some(style);
        }
        Ok(())
    }

    fn parse_bracketed_namespace_block(
        &mut self,
        namespace: Option<String>,
        classes: &mut Vec<ClassDecl>,
        traits: &mut Vec<TraitDecl>,
        functions: &mut Vec<FunctionDecl>,
        statements: &mut Vec<Statement>,
    ) -> Result<()> {
        self.expect_left_brace()?;

        let saved_namespace = self.current_namespace.clone();
        let saved_class_aliases = self.class_aliases.clone();
        let saved_function_aliases = self.function_aliases.clone();
        let saved_constant_aliases = self.constant_aliases.clone();

        self.current_namespace = namespace;
        self.clear_namespace_imports();
        let result = (|| {
            self.parse_top_level_items(
                classes,
                traits,
                functions,
                statements,
                TopLevelScope::NamespaceBlock,
            )?;
            self.expect_right_brace()?;
            Ok(())
        })();

        self.current_namespace = saved_namespace;
        self.class_aliases = saved_class_aliases;
        self.function_aliases = saved_function_aliases;
        self.constant_aliases = saved_constant_aliases;

        result
    }

    fn parse_namespace_name(&mut self) -> Result<String> {
        let parsed = self.parse_name("expected namespace name")?;
        match parsed.resolution {
            NameResolution::Unqualified | NameResolution::Qualified => Ok(parsed.name),
            NameResolution::FullyQualified => Err(Diagnostic::new(
                "namespace declarations cannot use fully qualified names",
                Some(parsed.span),
            )),
            NameResolution::NamespaceRelative => {
                let text = self
                    .source
                    .get(parsed.span.byte_start..parsed.span.byte_end)
                    .unwrap_or("namespace");
                if parsed.name.is_empty() {
                    Err(Diagnostic::new(
                        format!("Cannot use '{text}' as namespace name"),
                        Some(parsed.span),
                    ))
                } else {
                    Err(Diagnostic::parse_error(
                        format!(
                            "syntax error, unexpected namespace-relative name \"{text}\", expecting \"{{\""
                        ),
                        Some(parsed.span),
                    ))
                }
            }
        }
    }

    fn parse_use_declarations(&mut self) -> Result<()> {
        let use_span = self.advance().span;
        if self.block_depth != 0 || self.function_depth != 0 {
            return Err(Diagnostic::new(
                "use declarations must be at top level",
                Some(use_span),
            ));
        }
        let kind = if matches!(self.peek().kind, TokenKind::Function) {
            self.advance();
            UseDeclarationKind::Function
        } else if matches!(self.peek().kind, TokenKind::Const) {
            self.advance();
            UseDeclarationKind::Constant
        } else {
            UseDeclarationKind::Class
        };
        loop {
            self.parse_use_import(kind)?;
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_semicolon()?;
        Ok(())
    }

    fn parse_use_import(&mut self, kind: UseDeclarationKind) -> Result<()> {
        let target = self.parse_use_name("expected imported name")?;
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            return self.parse_grouped_use_imports(kind, target);
        }
        self.parse_single_use_import(kind, target)
    }

    fn parse_single_use_import(
        &mut self,
        kind: UseDeclarationKind,
        target: ParsedName,
    ) -> Result<()> {
        let (alias, alias_span) = if matches!(self.peek().kind, TokenKind::As) {
            self.advance();
            let token = self.advance().clone();
            let TokenKind::Identifier(alias) = token.kind else {
                return Err(Diagnostic::new("expected import alias", Some(token.span)));
            };
            (alias, token.span)
        } else {
            (
                target
                    .name
                    .rsplit('\\')
                    .next()
                    .unwrap_or(&target.name)
                    .to_string(),
                target.span,
            )
        };
        self.register_use_import(kind, target, alias, alias_span)?;
        Ok(())
    }

    fn parse_grouped_use_imports(
        &mut self,
        outer_kind: UseDeclarationKind,
        prefix: ParsedName,
    ) -> Result<()> {
        if prefix.resolution == NameResolution::NamespaceRelative {
            return Err(Diagnostic::new(
                "namespace-relative grouped use prefixes are unsupported",
                Some(prefix.span),
            ));
        }
        self.expect_left_brace()?;
        loop {
            let item_kind = if outer_kind == UseDeclarationKind::Class
                && matches!(self.peek().kind, TokenKind::Function)
            {
                self.advance();
                UseDeclarationKind::Function
            } else if outer_kind == UseDeclarationKind::Class
                && matches!(self.peek().kind, TokenKind::Const)
            {
                self.advance();
                UseDeclarationKind::Constant
            } else {
                outer_kind
            };
            let item = self.parse_name("expected imported name")?;
            if item.resolution == NameResolution::FullyQualified {
                return Err(Diagnostic::new(
                    "fully qualified grouped use items are unsupported",
                    Some(item.span),
                ));
            }
            if item.resolution == NameResolution::NamespaceRelative {
                return Err(Diagnostic::new(
                    "namespace-relative grouped use items are unsupported",
                    Some(item.span),
                ));
            }
            let (alias, alias_span) = if matches!(self.peek().kind, TokenKind::As) {
                self.advance();
                let token = self.advance().clone();
                let TokenKind::Identifier(alias) = token.kind else {
                    return Err(Diagnostic::new("expected import alias", Some(token.span)));
                };
                (alias, token.span)
            } else {
                (
                    item.name
                        .rsplit('\\')
                        .next()
                        .unwrap_or(&item.name)
                        .to_string(),
                    item.span,
                )
            };
            let grouped_target = ParsedName {
                name: format!("{}\\{}", prefix.name, item.name),
                span: combine_spans(prefix.span, item.span),
                resolution: prefix.resolution,
            };
            self.register_use_import(item_kind, grouped_target, alias, alias_span)?;
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_right_brace()?;
        Ok(())
    }

    fn register_use_import(
        &mut self,
        kind: UseDeclarationKind,
        target: ParsedName,
        alias: String,
        alias_span: SourceSpan,
    ) -> Result<()> {
        let target_name = match target.resolution {
            NameResolution::FullyQualified => target.name,
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&target.name),
            NameResolution::Unqualified | NameResolution::Qualified => target.name,
        };
        let alias_key = alias.to_ascii_lowercase();
        match kind {
            UseDeclarationKind::Class => {
                let declared_key = self.qualify_current_namespace(&alias).to_ascii_lowercase();
                if self.declared_class_names.contains_key(&declared_key) {
                    return Err(Diagnostic::new(
                        format!(
                            "Cannot use {target_name} as {alias} because the name is already in use"
                        ),
                        Some(alias_span),
                    ));
                }
                self.class_aliases.insert(alias_key, target_name);
            }
            UseDeclarationKind::Function => {
                self.function_aliases.insert(alias_key, target_name);
            }
            UseDeclarationKind::Constant => {
                self.constant_aliases.insert(alias_key, target_name);
            }
        }
        Ok(())
    }

    fn register_declared_class_name(&mut self, class: &ClassDecl) -> Result<()> {
        let local_name = class
            .name
            .rsplit('\\')
            .next()
            .unwrap_or(&class.name)
            .to_string();
        if self
            .class_aliases
            .contains_key(&local_name.to_ascii_lowercase())
        {
            return Err(Diagnostic::new(
                format!(
                    "Cannot redeclare class {local_name} (previously declared as local import)"
                ),
                Some(class.span),
            ));
        }
        self.declared_class_names
            .insert(class.name.to_ascii_lowercase(), class.span);
        Ok(())
    }

    fn parse_use_name(&mut self, expected: &str) -> Result<ParsedName> {
        let leading_backslash = matches!(self.peek().kind, TokenKind::Backslash);
        let start_span = if leading_backslash {
            Some(self.advance().span)
        } else {
            None
        };
        let first_token = self.advance().clone();
        let TokenKind::Identifier(first_segment) = first_token.kind else {
            return Err(Diagnostic::new(expected, Some(first_token.span)));
        };
        let mut span = start_span
            .map(|span| combine_spans(span, first_token.span))
            .unwrap_or(first_token.span);
        let mut segments = vec![first_segment];
        while matches!(self.peek().kind, TokenKind::Backslash) {
            if matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::LeftBrace)
            ) {
                self.advance();
                break;
            }
            self.advance();
            let segment_token = self.advance().clone();
            let TokenKind::Identifier(segment) = segment_token.kind else {
                return Err(Diagnostic::new(expected, Some(segment_token.span)));
            };
            span = combine_spans(span, segment_token.span);
            segments.push(segment);
        }
        let leading_backslash = start_span.is_some();
        let namespace_relative = !leading_backslash
            && segments
                .first()
                .is_some_and(|segment| segment.eq_ignore_ascii_case("namespace"));
        let resolution = if leading_backslash {
            NameResolution::FullyQualified
        } else if namespace_relative {
            NameResolution::NamespaceRelative
        } else if segments.len() == 1 {
            NameResolution::Unqualified
        } else {
            NameResolution::Qualified
        };
        let name_segments = if namespace_relative {
            &segments[1..]
        } else {
            &segments[..]
        };
        Ok(ParsedName {
            name: name_segments.join("\\"),
            span,
            resolution,
        })
    }

    fn clear_namespace_imports(&mut self) {
        self.class_aliases.clear();
        self.function_aliases.clear();
        self.constant_aliases.clear();
    }

    fn parse_declaration_name(&mut self, expected: &str) -> Result<(String, SourceSpan)> {
        let parsed = self.parse_name(expected)?;
        match parsed.resolution {
            NameResolution::Unqualified => {
                let name = self.qualify_current_namespace(&parsed.name);
                Ok((name, parsed.span))
            }
            _ => Err(Diagnostic::new(
                "declarations must use unqualified names",
                Some(parsed.span),
            )),
        }
    }

    fn parse_resolved_class_name(&mut self, expected: &str) -> Result<(String, SourceSpan)> {
        let parsed = self.parse_name(expected)?;
        let span = parsed.span;
        let name = self.resolve_class_name(&parsed);
        Ok((self.resolve_runtime_class_alias_name(&name), span))
    }

    fn parse_resolved_function_name(&mut self, expected: &str) -> Result<(String, SourceSpan)> {
        let parsed = self.parse_name(expected)?;
        let span = parsed.span;
        Ok((self.resolve_function_name(&parsed), span))
    }

    fn parse_name(&mut self, expected: &str) -> Result<ParsedName> {
        let leading_backslash = matches!(self.peek().kind, TokenKind::Backslash);
        let start_span = if leading_backslash {
            Some(self.advance().span)
        } else {
            None
        };
        let first_token = self.advance().clone();
        let TokenKind::Identifier(first_segment) = first_token.kind else {
            return Err(Diagnostic::new(expected, Some(first_token.span)));
        };
        self.parse_name_from_first(first_segment, first_token.span, start_span, expected)
    }

    fn parse_name_from_first(
        &mut self,
        first_segment: String,
        first_span: SourceSpan,
        leading_span: Option<SourceSpan>,
        expected: &str,
    ) -> Result<ParsedName> {
        let mut span = leading_span
            .map(|span| combine_spans(span, first_span))
            .unwrap_or(first_span);
        let mut segments = vec![first_segment];
        while matches!(self.peek().kind, TokenKind::Backslash) {
            self.advance();
            let segment_token = self.advance().clone();
            let TokenKind::Identifier(segment) = segment_token.kind else {
                return Err(Diagnostic::new(expected, Some(segment_token.span)));
            };
            span = combine_spans(span, segment_token.span);
            segments.push(segment);
        }
        let leading_backslash = leading_span.is_some();
        let namespace_relative = !leading_backslash
            && segments
                .first()
                .is_some_and(|segment| segment.eq_ignore_ascii_case("namespace"));
        let resolution = if leading_backslash {
            NameResolution::FullyQualified
        } else if namespace_relative {
            NameResolution::NamespaceRelative
        } else if segments.len() == 1 {
            NameResolution::Unqualified
        } else {
            NameResolution::Qualified
        };
        let name_segments = if namespace_relative {
            &segments[1..]
        } else {
            &segments[..]
        };
        Ok(ParsedName {
            name: name_segments.join("\\"),
            span,
            resolution,
        })
    }

    fn qualify_current_namespace(&self, name: &str) -> String {
        match self.current_namespace.as_deref() {
            Some(namespace) if !namespace.is_empty() && !name.is_empty() => {
                format!("{namespace}\\{name}")
            }
            _ => name.to_string(),
        }
    }

    fn resolve_class_name(&self, parsed: &ParsedName) -> String {
        if parsed.resolution == NameResolution::Unqualified {
            let lowered = parsed.name.to_ascii_lowercase();
            if matches!(lowered.as_str(), "self" | "static" | "parent") {
                return lowered;
            }
        }
        match parsed.resolution {
            NameResolution::FullyQualified => parsed.name.clone(),
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&parsed.name),
            NameResolution::Unqualified | NameResolution::Qualified => {
                self.resolve_aliasable_name(&parsed.name, &self.class_aliases)
            }
        }
    }

    fn resolve_function_name(&self, parsed: &ParsedName) -> String {
        let resolved = match parsed.resolution {
            NameResolution::FullyQualified => parsed.name.clone(),
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&parsed.name),
            NameResolution::Unqualified => {
                let alias_key = parsed.name.to_ascii_lowercase();
                let namespaced = self.qualify_current_namespace(&parsed.name);
                if let Some(target) = self.function_aliases.get(&alias_key) {
                    target.clone()
                } else if namespaced != parsed.name
                    && self
                        .declared_functions
                        .contains(&namespaced.to_ascii_lowercase())
                {
                    namespaced
                } else if is_modeled_internal_function_name(&alias_key) {
                    alias_key
                } else {
                    namespaced
                }
            }
            NameResolution::Qualified => {
                self.resolve_aliasable_name(&parsed.name, &self.class_aliases)
            }
        };
        resolved.to_ascii_lowercase()
    }

    fn resolve_constant_name(&self, parsed: &ParsedName) -> String {
        match parsed.resolution {
            NameResolution::FullyQualified => parsed.name.clone(),
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&parsed.name),
            NameResolution::Unqualified => {
                let alias_key = parsed.name.to_ascii_lowercase();
                if let Some(target) = self.constant_aliases.get(&alias_key) {
                    target.clone()
                } else if is_modeled_global_constant_name(&parsed.name) {
                    parsed.name.clone()
                } else {
                    self.qualify_current_namespace(&parsed.name)
                }
            }
            NameResolution::Qualified => {
                self.resolve_aliasable_name(&parsed.name, &self.class_aliases)
            }
        }
    }

    fn resolve_aliasable_name(&self, name: &str, aliases: &HashMap<String, String>) -> String {
        let mut parts = name.split('\\');
        let Some(first) = parts.next() else {
            return self.qualify_current_namespace(name);
        };
        let suffix = parts.collect::<Vec<_>>().join("\\");
        if let Some(target) = aliases.get(&first.to_ascii_lowercase()) {
            if suffix.is_empty() {
                target.clone()
            } else {
                format!("{target}\\{suffix}")
            }
        } else {
            self.qualify_current_namespace(name)
        }
    }

    fn note_runtime_class_alias_statement(&mut self, statement: &Statement) {
        let Some((name, arguments, argument_names, argument_unpacks)) = (match statement {
            Statement::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                ..
            } => Some((name, arguments, argument_names, argument_unpacks)),
            Statement::Expression {
                expression:
                    Expr::Call {
                        name,
                        arguments,
                        argument_names,
                        argument_unpacks,
                        ..
                    },
                ..
            } => Some((name, arguments, argument_names, argument_unpacks)),
            _ => None,
        }) else {
            return;
        };
        if !name.eq_ignore_ascii_case("class_alias")
            || arguments.len() < 2
            || argument_names.iter().take(2).any(Option::is_some)
            || argument_unpacks.iter().take(2).any(|unpack| *unpack)
        {
            return;
        }
        let Some(target) = self.compile_time_class_name_string(&arguments[0]) else {
            return;
        };
        let Some(alias) = self.compile_time_class_name_string(&arguments[1]) else {
            return;
        };
        self.runtime_class_aliases.insert(
            normalize_runtime_class_alias_key(&alias),
            normalize_runtime_class_alias_target(&target),
        );
    }

    fn compile_time_class_name_string(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::String(value, _) => Some(value.clone()),
            Expr::MagicConstant(MagicConstantKind::Namespace, _) => {
                Some(self.current_namespace.clone().unwrap_or_default())
            }
            Expr::ClassConstantFetch {
                class_name, name, ..
            } if name.eq_ignore_ascii_case("class") => Some(class_name.clone()),
            Expr::Binary {
                op: BinaryOp::Concat,
                left,
                right,
                ..
            } => {
                let mut value = self.compile_time_class_name_string(left)?;
                value.push_str(&self.compile_time_class_name_string(right)?);
                Some(value)
            }
            Expr::Grouped { expr, .. } => self.compile_time_class_name_string(expr),
            _ => None,
        }
    }

    fn resolve_runtime_class_alias_name(&self, name: &str) -> String {
        if matches!(
            name.to_ascii_lowercase().as_str(),
            "self" | "static" | "parent"
        ) {
            return name.to_string();
        }
        let mut resolved = normalize_runtime_class_alias_target(name);
        let mut seen = HashSet::new();
        loop {
            let key = normalize_runtime_class_alias_key(&resolved);
            if !seen.insert(key.clone()) {
                break;
            }
            let Some(target) = self.runtime_class_aliases.get(&key) else {
                break;
            };
            resolved = normalize_runtime_class_alias_target(target);
        }
        resolved
    }

    fn parse_class_decl(&mut self, attributes: ParsedAttributes) -> Result<ClassDecl> {
        let mut is_abstract = false;
        let mut abstract_span = None;
        let mut is_final = false;
        let mut final_span = None;
        loop {
            if token_is_identifier_named(self.peek(), "abstract")
                && (token_is_identifier_named(self.peek_next(), "class")
                    || token_is_identifier_named(self.peek_next(), "final")
                    || token_is_identifier_named(self.peek_next(), "readonly"))
            {
                if is_abstract {
                    return Err(Diagnostic::new(
                        "Multiple abstract modifiers are not allowed",
                        Some(self.peek().span),
                    ));
                }
                is_abstract = true;
                abstract_span = Some(self.advance().span);
                continue;
            }
            if token_is_identifier_named(self.peek(), "final")
                && (token_is_identifier_named(self.peek_next(), "class")
                    || token_is_identifier_named(self.peek_next(), "abstract")
                    || token_is_identifier_named(self.peek_next(), "final"))
            {
                if is_final {
                    return Err(Diagnostic::new(
                        "Multiple final modifiers are not allowed",
                        Some(self.peek().span),
                    ));
                }
                is_final = true;
                final_span = Some(self.advance().span);
                continue;
            }
            break;
        }
        if is_abstract && is_final {
            return Err(Diagnostic::new(
                "Cannot use the final modifier on an abstract class",
                final_span.or(abstract_span),
            ));
        }
        let readonly_span = if token_is_identifier_named(self.peek(), "readonly")
            && token_is_identifier_named(self.peek_next(), "class")
        {
            Some(self.advance().span)
        } else {
            None
        };
        let class_token = self.advance().clone();
        let TokenKind::Identifier(keyword) = &class_token.kind else {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        };
        let is_interface = keyword.eq_ignore_ascii_case("interface");
        if !keyword.eq_ignore_ascii_case("class") && !is_interface {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        }
        if is_interface && (is_abstract || readonly_span.is_some()) {
            return Err(Diagnostic::new(
                "interface declarations cannot be abstract or readonly",
                abstract_span.or(readonly_span).or(Some(class_token.span)),
            ));
        }
        if is_final && is_interface {
            return Err(Diagnostic::new(
                "interface declarations cannot be final",
                final_span.or(Some(class_token.span)),
            ));
        }
        let is_readonly = readonly_span.is_some();

        let (class_name, _) = self.parse_declaration_name("expected class name")?;
        let parent_name = if !is_interface && token_is_identifier_named(self.peek(), "extends") {
            self.advance();
            Some(
                self.parse_resolved_class_name("expected parent class name")?
                    .0,
            )
        } else {
            None
        };
        let mut interfaces = Vec::new();
        if is_interface && token_is_identifier_named(self.peek(), "extends") {
            self.advance();
            interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            }
        }
        if !is_interface && token_is_identifier_named(self.peek(), "implements") {
            self.advance();
            interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            }
        }

        self.expect_left_brace()?;
        let previous_type_scope = self.active_type_scope.replace(ActiveTypeScope {
            class_name: class_name.clone(),
            parent_name: parent_name.clone(),
            allow_unresolved_parent: false,
        });
        let mut properties = Vec::new();
        let mut static_properties = Vec::new();
        let mut constants = Vec::new();
        let mut methods = Vec::new();
        let mut trait_uses = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            match self.parse_class_member(is_readonly, is_interface, &class_name)? {
                ParsedClassMember::Method(method) => {
                    if method.name.eq_ignore_ascii_case("__construct") {
                        properties
                            .extend(promoted_properties_from_constructor(&method, is_readonly));
                    }
                    methods.push(method);
                }
                ParsedClassMember::Properties(parsed_properties) => {
                    properties.extend(parsed_properties);
                }
                ParsedClassMember::StaticProperties(properties) => {
                    static_properties.extend(properties);
                }
                ParsedClassMember::Constants(parsed_constants) => {
                    constants.extend(parsed_constants);
                }
                ParsedClassMember::TraitUses(parsed_trait_uses) => {
                    trait_uses.extend(parsed_trait_uses);
                }
            }
        }
        self.active_type_scope = previous_type_scope;
        let right_span = self.expect_right_brace()?;
        let span = combine_spans(
            abstract_span.or(readonly_span).unwrap_or(class_token.span),
            right_span,
        );
        Ok(ClassDecl {
            name: class_name,
            parent_name,
            interfaces,
            trait_uses,
            attributes,
            is_abstract: is_abstract || is_interface,
            is_final,
            is_interface,
            is_readonly,
            properties,
            static_properties,
            constants,
            methods,
            span,
        })
    }

    fn parse_trait_decl(&mut self, attributes: ParsedAttributes) -> Result<TraitDecl> {
        let trait_token = self.advance().clone();
        if !token_is_identifier_named(&trait_token, "trait") {
            return Err(Diagnostic::new("expected trait", Some(trait_token.span)));
        }
        let (trait_name, _) = self.parse_declaration_name("expected trait name")?;
        self.expect_left_brace()?;
        let previous_type_scope = self.active_type_scope.replace(ActiveTypeScope {
            class_name: trait_name.clone(),
            parent_name: None,
            allow_unresolved_parent: true,
        });

        let mut properties = Vec::new();
        let mut static_properties = Vec::new();
        let mut constants = Vec::new();
        let mut methods = Vec::new();
        let mut trait_uses = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            match self.parse_class_member(false, false, &trait_name)? {
                ParsedClassMember::Method(mut method) => {
                    method.trait_name = Some(trait_name.clone());
                    methods.push(method);
                }
                ParsedClassMember::Properties(parsed_properties) => {
                    properties.extend(parsed_properties);
                }
                ParsedClassMember::StaticProperties(parsed_properties) => {
                    static_properties.extend(parsed_properties);
                }
                ParsedClassMember::Constants(parsed_constants) => {
                    constants.extend(parsed_constants);
                }
                ParsedClassMember::TraitUses(parsed_trait_uses) => {
                    trait_uses.extend(parsed_trait_uses);
                }
            }
        }
        self.active_type_scope = previous_type_scope;
        let right_span = self.expect_right_brace()?;
        Ok(TraitDecl {
            name: trait_name,
            trait_uses,
            attributes,
            properties,
            static_properties,
            constants,
            methods,
            span: combine_spans(trait_token.span, right_span),
        })
    }

    fn parse_trait_use_declarations(&mut self) -> Result<Vec<TraitUseDecl>> {
        self.advance();
        let mut trait_uses = Vec::new();
        loop {
            let (name, span) = self.parse_resolved_class_name("expected trait name")?;
            trait_uses.push(TraitUseDecl { name, span });
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            return Err(Diagnostic::new(
                "trait adaptations are unsupported",
                Some(self.peek().span),
            ));
        }
        self.expect_semicolon()?;
        Ok(trait_uses)
    }

    fn parse_class_member(
        &mut self,
        class_is_readonly: bool,
        class_is_interface: bool,
        class_name: &str,
    ) -> Result<ParsedClassMember> {
        let attributes = self.parse_attribute_groups()?;
        let mut modifiers = self.parse_class_modifiers()?;
        if token_is_identifier_named(self.peek(), "use") {
            if class_is_interface {
                return Err(Diagnostic::new(
                    "interfaces may not use traits",
                    Some(self.peek().span),
                ));
            }
            if modifiers.visibility_span.is_some()
                || modifiers.static_span.is_some()
                || modifiers.abstract_span.is_some()
                || modifiers.final_span.is_some()
                || modifiers.readonly_span.is_some()
                || modifiers.set_visibility_span.is_some()
            {
                return Err(Diagnostic::new(
                    "trait use declarations cannot have modifiers",
                    modifiers
                        .visibility_span
                        .or(modifiers.static_span)
                        .or(modifiers.abstract_span)
                        .or(modifiers.final_span)
                        .or(modifiers.readonly_span)
                        .or(modifiers.set_visibility_span),
                ));
            }
            return Ok(ParsedClassMember::TraitUses(
                self.parse_trait_use_declarations()?,
            ));
        }
        if modifiers.is_abstract && modifiers.is_final {
            return Err(Diagnostic::new(
                "Cannot use the final modifier on an abstract method",
                modifiers.final_span.or(modifiers.abstract_span),
            ));
        }
        if matches!(self.peek().kind, TokenKind::Const) {
            if modifiers.is_abstract {
                return Err(Diagnostic::new(
                    "Cannot use the abstract modifier on a class constant",
                    modifiers.abstract_span,
                ));
            }
            if modifiers.is_static {
                return Err(Diagnostic::new(
                    "static class constants are unsupported",
                    Some(self.peek().span),
                ));
            }
            if modifiers.visibility != PropertyVisibility::Public {
                return Err(Diagnostic::new(
                    "non-public class constants are unsupported",
                    modifiers.visibility_span,
                ));
            }
            if class_is_interface && modifiers.visibility != PropertyVisibility::Public {
                return Err(Diagnostic::new(
                    format!("Access type for interface constant {class_name} must be public"),
                    modifiers.visibility_span,
                ));
            }
            return Ok(ParsedClassMember::Constants(
                self.parse_class_constant_declarations(modifiers.visibility)?,
            ));
        }
        if class_is_interface && matches!(self.peek().kind, TokenKind::Variable(_)) {
            return Err(Diagnostic::new(
                "interfaces may not include properties",
                Some(self.peek().span),
            ));
        }
        let member_is_readonly = class_is_readonly || modifiers.is_readonly;
        let set_visibility = modifiers
            .set_visibility
            .unwrap_or_else(|| default_set_visibility(modifiers.visibility, member_is_readonly));
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            if modifiers.is_static {
                if member_is_readonly {
                    return Err(Diagnostic::new(
                        "readonly static properties are unsupported",
                        modifiers.readonly_span.or(Some(self.peek().span)),
                    ));
                }
                return Ok(ParsedClassMember::StaticProperties(
                    self.parse_static_property_declarations(
                        modifiers.visibility,
                        set_visibility,
                        modifiers.set_visibility_span,
                        attributes,
                        class_name,
                    )?,
                ));
            }
            return Ok(ParsedClassMember::Properties(
                self.parse_property_declarations(
                    modifiers.visibility,
                    set_visibility,
                    modifiers.set_visibility_span,
                    member_is_readonly,
                    attributes,
                    class_is_interface,
                    class_name,
                )?,
            ));
        }
        if self.peek_starts_property_type_hint() {
            if modifiers.is_static {
                if member_is_readonly {
                    return Err(Diagnostic::new(
                        "readonly static properties are unsupported",
                        modifiers.readonly_span.or(Some(self.peek().span)),
                    ));
                }
                return Ok(ParsedClassMember::StaticProperties(
                    self.parse_static_property_declarations(
                        modifiers.visibility,
                        set_visibility,
                        modifiers.set_visibility_span,
                        attributes,
                        class_name,
                    )?,
                ));
            }
            return Ok(ParsedClassMember::Properties(
                self.parse_property_declarations(
                    modifiers.visibility,
                    set_visibility,
                    modifiers.set_visibility_span,
                    member_is_readonly,
                    attributes,
                    class_is_interface,
                    class_name,
                )?,
            ));
        }
        if !matches!(self.peek().kind, TokenKind::Function) {
            return Err(Diagnostic::new(
                "unsupported class member",
                Some(self.peek().span),
            ));
        }
        if class_is_interface {
            modifiers.is_abstract = true;
        }
        let method =
            self.parse_method_decl(attributes, modifiers, class_is_readonly, class_name)?;
        if class_is_interface && method.visibility != PropertyVisibility::Public {
            return Err(Diagnostic::new(
                format!(
                    "Access type for interface method {class_name}::{}() must be public",
                    method.name
                ),
                modifiers.visibility_span,
            ));
        }
        Ok(ParsedClassMember::Method(method))
    }

    fn parse_class_modifiers(&mut self) -> Result<ClassModifiers> {
        let mut modifiers = ClassModifiers::default();
        loop {
            let TokenKind::Identifier(modifier) = &self.peek().kind else {
                break;
            };
            match modifier.to_ascii_lowercase().as_str() {
                "public" => {
                    if self.peek_starts_set_visibility_modifier() {
                        self.parse_set_visibility_modifier(
                            &mut modifiers,
                            PropertyVisibility::Public,
                        )?;
                        continue;
                    }
                    if modifiers.visibility_span.is_some() {
                        return Err(Diagnostic::new(
                            "Multiple access type modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.visibility = PropertyVisibility::Public;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "var" => {
                    if modifiers.visibility_span.is_some() {
                        return Err(Diagnostic::new(
                            "Multiple access type modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.visibility = PropertyVisibility::Public;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "static" => {
                    if modifiers.is_static {
                        return Err(Diagnostic::new(
                            "Multiple static modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.is_static = true;
                    modifiers.static_span = Some(self.peek().span);
                    self.advance();
                }
                "private" => {
                    if self.peek_starts_set_visibility_modifier() {
                        self.parse_set_visibility_modifier(
                            &mut modifiers,
                            PropertyVisibility::Private,
                        )?;
                        continue;
                    }
                    if modifiers.visibility_span.is_some() {
                        return Err(Diagnostic::new(
                            "Multiple access type modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.visibility = PropertyVisibility::Private;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "protected" => {
                    if self.peek_starts_set_visibility_modifier() {
                        self.parse_set_visibility_modifier(
                            &mut modifiers,
                            PropertyVisibility::Protected,
                        )?;
                        continue;
                    }
                    if modifiers.visibility_span.is_some() {
                        return Err(Diagnostic::new(
                            "Multiple access type modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.visibility = PropertyVisibility::Protected;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "abstract" => {
                    if modifiers.is_abstract {
                        return Err(Diagnostic::new(
                            "Multiple abstract modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.is_abstract = true;
                    modifiers.abstract_span = Some(self.peek().span);
                    self.advance();
                }
                "final" => {
                    if modifiers.is_final {
                        return Err(Diagnostic::new(
                            "Multiple final modifiers are not allowed",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.is_final = true;
                    modifiers.final_span = Some(self.peek().span);
                    self.advance();
                }
                "readonly" => {
                    if modifiers.is_readonly {
                        return Err(Diagnostic::new(
                            "duplicate readonly modifier",
                            Some(self.peek().span),
                        ));
                    }
                    modifiers.is_readonly = true;
                    modifiers.readonly_span = Some(self.peek().span);
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(modifiers)
    }

    fn parse_set_visibility_modifier(
        &mut self,
        modifiers: &mut ClassModifiers,
        visibility: PropertyVisibility,
    ) -> Result<()> {
        let visibility_span = self.advance().span;
        self.expect_left_paren()?;
        if !self.peek_is_identifier("set") {
            return Err(Diagnostic::new(
                "expected set visibility operation",
                Some(self.peek().span),
            ));
        }
        self.advance();
        self.expect_right_paren()?;
        if modifiers.set_visibility.is_some() {
            return Err(Diagnostic::new(
                "Multiple access type modifiers are not allowed",
                Some(visibility_span),
            ));
        }
        modifiers.set_visibility = Some(visibility);
        modifiers.set_visibility_span = Some(visibility_span);
        Ok(())
    }

    fn parse_static_property_declarations(
        &mut self,
        visibility: PropertyVisibility,
        set_visibility: PropertyVisibility,
        set_visibility_span: Option<SourceSpan>,
        attributes: ParsedAttributes,
        class_name: &str,
    ) -> Result<Vec<StaticPropertyDecl>> {
        let type_hint = self.parse_optional_property_type_hint()?;
        let has_type = type_hint.is_some();
        if set_visibility_span.is_some() && !has_type {
            let property_name = match &self.peek().kind {
                TokenKind::Variable(name) => name.as_str(),
                _ => "",
            };
            return Err(Diagnostic::new(
                format!(
                    "Property with asymmetric visibility {class_name}::${property_name} must have type"
                ),
                set_visibility_span,
            ));
        }
        let mut properties = vec![self.parse_static_property_declaration(
            visibility,
            set_visibility,
            type_hint.clone(),
            attributes.clone(),
            class_name,
        )?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            properties.push(self.parse_static_property_declaration(
                visibility,
                set_visibility,
                type_hint.clone(),
                attributes.clone(),
                class_name,
            )?);
        }
        self.expect_semicolon()?;
        Ok(properties)
    }

    fn parse_class_constant_declarations(
        &mut self,
        visibility: PropertyVisibility,
    ) -> Result<Vec<ClassConstantDecl>> {
        self.expect_const()?;
        let mut constants = vec![self.parse_class_constant_declaration(visibility)?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            constants.push(self.parse_class_constant_declaration(visibility)?);
        }
        self.expect_const_statement_terminator()?;
        Ok(constants)
    }

    fn parse_class_constant_declaration(
        &mut self,
        visibility: PropertyVisibility,
    ) -> Result<ClassConstantDecl> {
        let looks_like_typed_constant = (self.peek_is_type_hint()
            || matches!(self.peek().kind, TokenKind::Identifier(_)))
            && matches!(self.peek_next().kind, TokenKind::Identifier(_));
        if looks_like_typed_constant {
            return Err(Diagnostic::new(
                "typed class constants are unsupported",
                Some(self.peek().span),
            ));
        }
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected class constant name",
                Some(token.span),
            ));
        };
        self.expect_equal()?;
        let value = self.parse_expr()?;
        if !is_supported_const_declaration_expr(&value) {
            return Err(Diagnostic::new(
                "class constant value must be a supported constant expression",
                Some(value.span()),
            ));
        }
        Ok(ClassConstantDecl {
            name,
            visibility,
            value,
            span: token.span,
        })
    }

    fn parse_property_declarations(
        &mut self,
        visibility: PropertyVisibility,
        set_visibility: PropertyVisibility,
        set_visibility_span: Option<SourceSpan>,
        is_readonly: bool,
        attributes: ParsedAttributes,
        allow_property_hooks: bool,
        class_name: &str,
    ) -> Result<Vec<PropertyDecl>> {
        let type_hint = self.parse_optional_property_type_hint()?;
        let has_type = type_hint.is_some();
        if is_readonly && !has_type {
            let property_name = match &self.peek().kind {
                TokenKind::Variable(name) => name.as_str(),
                _ => "",
            };
            return Err(Diagnostic::new(
                format!("Readonly property {class_name}::${property_name} must have type"),
                Some(self.peek().span),
            ));
        }
        if set_visibility_span.is_some() && !has_type {
            let property_name = match &self.peek().kind {
                TokenKind::Variable(name) => name.as_str(),
                _ => "",
            };
            return Err(Diagnostic::new(
                format!(
                    "Property with asymmetric visibility {class_name}::${property_name} must have type"
                ),
                set_visibility_span,
            ));
        }
        let (first_property, first_had_hooks) = self.parse_property_declaration(
            visibility,
            set_visibility,
            is_readonly,
            type_hint.clone(),
            attributes.clone(),
            allow_property_hooks,
            class_name,
        )?;
        let mut properties = vec![first_property];
        self.reject_asymmetric_virtual_property_hook(
            &properties[0],
            set_visibility_span,
            class_name,
        )?;
        if first_had_hooks {
            return Ok(properties);
        }
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            let (property, had_hooks) = self.parse_property_declaration(
                visibility,
                set_visibility,
                is_readonly,
                type_hint.clone(),
                attributes.clone(),
                allow_property_hooks,
                class_name,
            )?;
            properties.push(property);
            let property = properties
                .last()
                .expect("property was just pushed for virtual hook validation");
            self.reject_asymmetric_virtual_property_hook(
                property,
                set_visibility_span,
                class_name,
            )?;
            if had_hooks {
                return Ok(properties);
            }
        }
        self.expect_semicolon()?;
        Ok(properties)
    }

    fn reject_asymmetric_virtual_property_hook(
        &self,
        property: &PropertyDecl,
        set_visibility_span: Option<SourceSpan>,
        class_name: &str,
    ) -> Result<()> {
        if set_visibility_span.is_none() || !matches!(self.peek().kind, TokenKind::LeftBrace) {
            return Ok(());
        }
        let hook_kind = self.peek_virtual_property_hook_kind();
        let description = match hook_kind {
            VirtualPropertyHookKind::GetOnly => "get-only",
            VirtualPropertyHookKind::SetOnly => "set-only",
            VirtualPropertyHookKind::Other => "virtual",
        };
        Err(Diagnostic::new(
            format!(
                "{description} virtual property {class_name}::${} must not specify asymmetric visibility",
                property.name
            ),
            Some(property.span),
        ))
    }

    fn peek_virtual_property_hook_kind(&self) -> VirtualPropertyHookKind {
        let mut depth = 0usize;
        let mut has_get = false;
        let mut has_set = false;
        for token in self.tokens.iter().skip(self.index) {
            match &token.kind {
                TokenKind::LeftBrace => {
                    depth += 1;
                }
                TokenKind::RightBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Identifier(name) if depth == 1 && name.eq_ignore_ascii_case("get") => {
                    has_get = true;
                }
                TokenKind::Identifier(name) if depth == 1 && name.eq_ignore_ascii_case("set") => {
                    has_set = true;
                }
                TokenKind::Eof => break,
                _ => {}
            }
        }
        match (has_get, has_set) {
            (true, false) => VirtualPropertyHookKind::GetOnly,
            (false, true) => VirtualPropertyHookKind::SetOnly,
            _ => VirtualPropertyHookKind::Other,
        }
    }

    fn parse_property_declaration(
        &mut self,
        visibility: PropertyVisibility,
        set_visibility: PropertyVisibility,
        is_readonly: bool,
        type_hint: Option<PropertyTypeHint>,
        attributes: ParsedAttributes,
        allow_property_hooks: bool,
        class_name: &str,
    ) -> Result<(PropertyDecl, bool)> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected property name", Some(token.span)));
        };
        validate_asymmetric_property_visibility(
            class_name,
            &name,
            visibility,
            set_visibility,
            token.span,
        )?;
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            if set_visibility != visibility {
                let hook_kind = self
                    .tokens
                    .get(self.index + 1)
                    .and_then(|token| match &token.kind {
                        TokenKind::Identifier(identifier)
                            if identifier.eq_ignore_ascii_case("get") =>
                        {
                            Some("get-only")
                        }
                        TokenKind::Identifier(identifier)
                            if identifier.eq_ignore_ascii_case("set") =>
                        {
                            Some("set-only")
                        }
                        _ => None,
                    })
                    .unwrap_or("virtual");
                return Err(Diagnostic::new(
                    format!(
                        "{hook_kind} virtual property {class_name}::${name} must not specify asymmetric visibility"
                    ),
                    Some(self.peek().span),
                ));
            }
            if !allow_property_hooks {
                return Err(Diagnostic::new(
                    "property hooks are unsupported",
                    Some(self.peek().span),
                ));
            }
            self.parse_property_hook_block()?;
            return Ok((
                PropertyDecl {
                    name,
                    visibility,
                    set_visibility,
                    is_readonly,
                    type_hint,
                    has_override_attribute: attributes.has_override,
                    value: None,
                    span: token.span,
                },
                true,
            ));
        }
        let value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            let value = self.parse_expr()?;
            if !is_supported_property_default_expr(&value) {
                return Err(Diagnostic::new(
                    "property default value must be a supported constant expression",
                    Some(value.span()),
                ));
            }
            if is_readonly {
                return Err(Diagnostic::new(
                    format!("Readonly property {class_name}::${name} cannot have default value"),
                    Some(value.span()),
                ));
            }
            Some(value)
        } else {
            None
        };
        Ok((
            PropertyDecl {
                name,
                visibility,
                set_visibility,
                is_readonly,
                type_hint,
                has_override_attribute: attributes.has_override,
                value,
                span: token.span,
            },
            false,
        ))
    }

    fn parse_static_property_declaration(
        &mut self,
        visibility: PropertyVisibility,
        set_visibility: PropertyVisibility,
        type_hint: Option<PropertyTypeHint>,
        attributes: ParsedAttributes,
        class_name: &str,
    ) -> Result<StaticPropertyDecl> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected static property name",
                Some(token.span),
            ));
        };
        validate_asymmetric_property_visibility(
            class_name,
            &name,
            visibility,
            set_visibility,
            token.span,
        )?;
        let value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            let value = self.parse_expr()?;
            if !is_supported_property_default_expr(&value) {
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
            visibility,
            set_visibility,
            type_hint,
            has_override_attribute: attributes.has_override,
            value,
            span: token.span,
        })
    }

    fn parse_property_hook_block(&mut self) -> Result<()> {
        self.expect_left_brace()?;
        let mut depth = 1usize;
        while depth > 0 {
            let token = self.advance().clone();
            match token.kind {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => depth -= 1,
                TokenKind::Eof => {
                    return Err(Diagnostic::new(
                        "unterminated property hook block",
                        Some(token.span),
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_optional_property_type_hint(&mut self) -> Result<Option<PropertyTypeHint>> {
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            return Ok(None);
        }
        if !self.peek_starts_property_type_hint() {
            return Ok(None);
        }
        let mut type_hint = self.parse_property_type_atom()?;
        let mut complex = false;
        while matches!(self.peek().kind, TokenKind::Pipe | TokenKind::Ampersand) {
            let separator = match self.advance().kind {
                TokenKind::Pipe => "|",
                TokenKind::Ampersand => "&",
                _ => unreachable!("property type separator checked"),
            };
            let atom = self.parse_property_type_atom()?;
            complex = true;
            type_hint.text.push_str(separator);
            type_hint.text.push_str(&atom.text);
        }
        if complex {
            type_hint.kind = PropertyTypeKind::Unsupported;
            type_hint.allows_null = false;
        }
        Ok(Some(type_hint))
    }

    fn parse_property_type_atom(&mut self) -> Result<PropertyTypeHint> {
        let allows_null = if matches!(self.peek().kind, TokenKind::Question) {
            self.advance();
            true
        } else {
            false
        };
        let nullable_prefix = if allows_null { "?" } else { "" };
        match &self.peek().kind {
            TokenKind::Null => {
                self.advance();
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}null"),
                    kind: PropertyTypeKind::Null,
                    allows_null,
                })
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("array") => {
                self.advance();
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}array"),
                    kind: PropertyTypeKind::Array,
                    allows_null,
                })
            }
            TokenKind::IntType | TokenKind::IntegerType => {
                let text = property_type_token_text(&self.advance().kind);
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{text}"),
                    kind: PropertyTypeKind::Int,
                    allows_null,
                })
            }
            TokenKind::FloatType | TokenKind::DoubleType => {
                let text = property_type_token_text(&self.advance().kind);
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{text}"),
                    kind: PropertyTypeKind::Float,
                    allows_null,
                })
            }
            TokenKind::StringType | TokenKind::BinaryType => {
                let text = property_type_token_text(&self.advance().kind);
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{text}"),
                    kind: PropertyTypeKind::String,
                    allows_null,
                })
            }
            TokenKind::BoolType | TokenKind::BooleanType => {
                let text = property_type_token_text(&self.advance().kind);
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{text}"),
                    kind: PropertyTypeKind::Bool,
                    allows_null,
                })
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("mixed") => {
                if allows_null {
                    return Err(Diagnostic::new(
                        "Type mixed cannot be marked as nullable since mixed already includes null",
                        Some(self.peek().span),
                    ));
                }
                self.advance();
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}mixed"),
                    kind: PropertyTypeKind::Mixed,
                    allows_null,
                })
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("object") => {
                self.advance();
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}object"),
                    kind: PropertyTypeKind::Object,
                    allows_null,
                })
            }
            TokenKind::Identifier(name)
                if matches!(
                    name.to_ascii_lowercase().as_str(),
                    "callable" | "false" | "iterable" | "never" | "static" | "true" | "void"
                ) =>
            {
                let name = name.clone();
                self.advance();
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{name}"),
                    kind: PropertyTypeKind::Unsupported,
                    allows_null,
                })
            }
            TokenKind::Backslash | TokenKind::Identifier(_) => {
                let (class_name, _) = self.parse_resolved_class_name("expected property type")?;
                Ok(PropertyTypeHint {
                    text: format!("{nullable_prefix}{class_name}"),
                    kind: PropertyTypeKind::Class(class_name),
                    allows_null,
                })
            }
            _ => Err(Diagnostic::new(
                "expected property type",
                Some(self.peek().span),
            )),
        }
    }

    fn parse_method_decl(
        &mut self,
        attributes: ParsedAttributes,
        modifiers: ClassModifiers,
        class_is_readonly: bool,
        class_name: &str,
    ) -> Result<MethodDecl> {
        let span = self.expect_function()?;
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        };
        let name_token = self.advance().clone();
        let name = method_name_from_token(&name_token.kind)
            .ok_or_else(|| Diagnostic::new("expected method name", Some(name_token.span)))?;
        let allow_promoted_properties = name.eq_ignore_ascii_case("__construct");
        let parameters = self.parse_function_parameters_with_promotions(
            if allow_promoted_properties {
                Some(class_name)
            } else {
                None
            },
            class_is_readonly,
        )?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
        } else {
            None
        };
        let mut body = if modifiers.is_abstract {
            self.expect_semicolon()?;
            Vec::new()
        } else {
            self.return_by_ref_stack.push(return_by_ref);
            self.function_depth += 1;
            let body = self.parse_block();
            self.function_depth -= 1;
            self.return_by_ref_stack.pop();
            body?
        };
        if allow_promoted_properties && !modifiers.is_abstract {
            let mut promoted_assignments = constructor_promoted_property_assignments(&parameters);
            if !promoted_assignments.is_empty() {
                promoted_assignments.extend(body);
                body = promoted_assignments;
            }
        }
        Ok(MethodDecl {
            name,
            visibility: modifiers.visibility,
            trait_name: None,
            attributes: attributes.clone(),
            has_override_attribute: attributes.has_override,
            parameters,
            return_type,
            return_by_ref,
            is_static: modifiers.is_static,
            is_abstract: modifiers.is_abstract,
            body,
            span,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let attributes = self.parse_attribute_groups()?;
        match self.peek().kind {
            TokenKind::Semicolon => self.parse_empty_statement(),
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
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Goto => self.parse_goto(),
            TokenKind::Const => self.parse_const(),
            TokenKind::Global => self.parse_global(),
            TokenKind::Function if matches!(self.peek_next().kind, TokenKind::Identifier(_)) => {
                self.parse_nested_function_decl_statement(attributes)
            }
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("static")
                    && matches!(self.peek_next().kind, TokenKind::Variable(_)) =>
            {
                self.parse_static_local()
            }
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("declare")
                    && matches!(self.peek_next().kind, TokenKind::LeftParen) =>
            {
                self.parse_declare_statement()
            }
            TokenKind::LeftBrace => self.parse_compound_block(),
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_increment_statement(),
            _ if self.peek_starts_class_decl() => self.parse_local_class_decl_statement(attributes),
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
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("match")
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

    fn parse_empty_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        Ok(Statement::Empty { span: token.span })
    }

    fn parse_local_class_decl_statement(
        &mut self,
        attributes: ParsedAttributes,
    ) -> Result<Statement> {
        let class = self.parse_class_decl(attributes)?;
        let span = class.span;
        let source = self
            .source
            .get(span.byte_start..span.byte_end)
            .unwrap_or("")
            .to_string();
        self.anonymous_classes.push(class);
        Ok(Statement::ClassDeclaration { source, span })
    }

    fn parse_nested_function_decl_statement(
        &mut self,
        attributes: ParsedAttributes,
    ) -> Result<Statement> {
        let mut function = self.parse_function_decl(attributes)?;
        function.is_conditionally_declared = true;
        let name = function.name.clone();
        let span = function.span;
        self.nested_functions.push(function);
        Ok(Statement::FunctionDeclaration { name, span })
    }

    fn parse_function_decl(&mut self, attributes: ParsedAttributes) -> Result<FunctionDecl> {
        let span = self.expect_function()?;
        let mut return_by_ref_span = None;
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            return_by_ref_span = Some(self.advance().span);
            true
        } else {
            false
        };
        let (name, _) = self.parse_declaration_name("expected function name")?;
        self.declared_functions.insert(name.to_ascii_lowercase());
        let parameters = self.parse_function_parameters()?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
        } else {
            None
        };
        let _ = return_by_ref_span;
        self.return_by_ref_stack.push(return_by_ref);
        self.function_depth += 1;
        let body = self.parse_block();
        self.function_depth -= 1;
        self.return_by_ref_stack.pop();
        let body = body?;
        Ok(FunctionDecl {
            name,
            attributes,
            parameters,
            return_type,
            return_by_ref,
            is_conditionally_declared: false,
            body,
            span,
        })
    }

    fn parse_anonymous_function_expr(
        &mut self,
        span: SourceSpan,
        is_static: bool,
        attributes: ParsedAttributes,
    ) -> Result<Expr> {
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        };
        self.allow_unscoped_relative_types += 1;
        let parameters = self.parse_function_parameters()?;
        let captures = self.parse_closure_use_captures()?;
        validate_closure_use_parameter_names(&parameters, &captures)?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
        } else {
            None
        };
        self.allow_unscoped_relative_types -= 1;
        self.return_by_ref_stack.push(return_by_ref);
        self.function_depth += 1;
        let body = self.parse_block();
        self.function_depth -= 1;
        self.return_by_ref_stack.pop();
        let body = body?;
        Ok(Expr::AnonymousFunction(AnonymousFunction {
            attributes,
            parameters,
            captures,
            return_type,
            return_by_ref,
            is_static,
            is_arrow: false,
            body,
            span,
        }))
    }

    fn parse_closure_use_captures(&mut self) -> Result<Vec<ClosureUseCapture>> {
        if !self.peek_is_identifier("use") {
            return Ok(Vec::new());
        }

        self.advance();
        self.expect_left_paren()?;
        let mut captures = Vec::new();
        let mut seen = std::collections::HashSet::new();
        loop {
            if matches!(self.peek().kind, TokenKind::RightParen) {
                break;
            }
            let by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
                self.advance();
                true
            } else {
                false
            };
            let token = self.advance();
            let TokenKind::Variable(name) = &token.kind else {
                return Err(syntax_error_unexpected(token, Some("variable")));
            };
            validate_closure_use_name(name, token.span)?;
            if !seen.insert(name.clone()) {
                return Err(Diagnostic::new(
                    format!("Cannot use variable ${name} twice"),
                    Some(token.span),
                ));
            }
            captures.push(ClosureUseCapture {
                name: name.clone(),
                by_ref,
                warn_if_missing: true,
                span: token.span,
            });

            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_right_paren()?;
        Ok(captures)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<FunctionParameter>> {
        self.parse_function_parameters_with_promotions(None, false)
    }

    fn parse_function_parameters_with_promotions(
        &mut self,
        class_name_for_promotions: Option<&str>,
        class_is_readonly: bool,
    ) -> Result<Vec<FunctionParameter>> {
        self.expect_left_paren()?;
        let mut parameters = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            parameters
                .push(self.parse_function_parameter(class_name_for_promotions, class_is_readonly)?);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                if matches!(self.peek().kind, TokenKind::RightParen) {
                    break;
                }
                parameters.push(
                    self.parse_function_parameter(class_name_for_promotions, class_is_readonly)?,
                );
            }
        }
        self.expect_right_paren()?;
        if let Some((index, parameter)) = parameters
            .iter()
            .enumerate()
            .find(|(_, parameter)| parameter.is_variadic)
        {
            if index + 1 != parameters.len() {
                return Err(Diagnostic::new(
                    "Only the last parameter can be variadic",
                    Some(parameter.span),
                ));
            }
        }
        validate_function_parameter_defaults(&parameters)?;
        Ok(parameters)
    }

    fn parse_function_parameter(
        &mut self,
        class_name_for_promotions: Option<&str>,
        class_is_readonly: bool,
    ) -> Result<FunctionParameter> {
        let attributes = self.parse_attribute_groups()?;
        let promotion_modifiers = if class_name_for_promotions.is_some() {
            let modifiers = self.parse_class_modifiers()?;
            if modifiers.has_promoted_property_modifier() {
                Some(modifiers)
            } else {
                None
            }
        } else {
            None
        };
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
        let is_variadic = if matches!(self.peek().kind, TokenKind::Ellipsis) {
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
        let promoted_property = if let Some(modifiers) = promotion_modifiers {
            let class_name = class_name_for_promotions.expect("promotion modifiers require class");
            let is_readonly = class_is_readonly || modifiers.is_readonly;
            let set_visibility = modifiers
                .set_visibility
                .unwrap_or_else(|| default_set_visibility(modifiers.visibility, is_readonly));
            if modifiers.set_visibility_span.is_some() && type_hint.is_none() {
                return Err(Diagnostic::new(
                    format!(
                        "Property with asymmetric visibility {class_name}::${name} must have type"
                    ),
                    modifiers.set_visibility_span,
                ));
            }
            validate_asymmetric_property_visibility(
                class_name,
                &name,
                modifiers.visibility,
                set_visibility,
                modifiers.set_visibility_span.unwrap_or(token.span),
            )?;
            Some(PromotedProperty {
                visibility: modifiers.visibility,
                set_visibility,
                is_readonly,
                has_override_attribute: attributes.has_override,
                span: modifiers.visibility_span.unwrap_or(token.span),
            })
        } else {
            None
        };
        let default_value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            let value = self.parse_expr()?;
            if !is_supported_parameter_default_expr(&value) {
                return Err(Diagnostic::new(
                    "function parameter default value must be a supported constant expression",
                    Some(value.span()),
                ));
            }
            Some(value)
        } else {
            None
        };
        Ok(FunctionParameter {
            name,
            type_hint,
            by_ref,
            is_variadic,
            default_value,
            promoted_property,
            span: token.span,
        })
    }

    fn parse_type_hint(&mut self) -> Result<TypeHint> {
        self.parse_type_hint_with_context(TypeHintContext::Parameter)
            .map(|parsed| parsed.type_hint)
    }

    fn parse_return_type_hint(&mut self) -> Result<TypeHint> {
        self.parse_type_hint_with_context(TypeHintContext::Return)
            .map(|parsed| parsed.type_hint)
    }

    fn parse_type_hint_with_context(&mut self, context: TypeHintContext) -> Result<ParsedTypeHint> {
        if matches!(self.peek().kind, TokenKind::Question) {
            let span = self.advance().span;
            let inner = self.parse_type_hint_with_context(context)?;
            let nullable_span = combine_spans(span, inner.span);
            return Ok(ParsedTypeHint {
                type_hint: nullable_type_hint(inner.type_hint, span)?,
                span: nullable_span,
            });
        }

        let first = self.parse_intersection_type_hint(context)?;
        let mut span = first.span;
        let mut types = vec![first.type_hint];
        while matches!(self.peek().kind, TokenKind::Pipe) {
            self.advance();
            let next = self.parse_intersection_type_hint(context)?;
            span = combine_spans(span, next.span);
            types.push(next.type_hint);
        }

        let type_hint = if types.len() == 1 {
            types.remove(0)
        } else {
            validate_union_type_hint(&types, span)?;
            TypeHint::Union(types)
        };
        Ok(ParsedTypeHint { type_hint, span })
    }

    fn parse_intersection_type_hint(&mut self, context: TypeHintContext) -> Result<ParsedTypeHint> {
        let first = self.parse_type_hint_atom(context)?;
        let mut span = first.span;
        let mut types = vec![first.type_hint];
        while matches!(self.peek().kind, TokenKind::Ampersand)
            && token_starts_type_hint_atom(self.peek_next())
        {
            self.advance();
            let next = self.parse_type_hint_atom(context)?;
            span = combine_spans(span, next.span);
            types.push(next.type_hint);
        }

        let type_hint = if types.len() == 1 {
            types.remove(0)
        } else {
            validate_intersection_type_hint(&types, span)?;
            TypeHint::Intersection(types)
        };
        Ok(ParsedTypeHint { type_hint, span })
    }

    fn parse_type_hint_atom(&mut self, context: TypeHintContext) -> Result<ParsedTypeHint> {
        if matches!(self.peek().kind, TokenKind::LeftParen) {
            let open_span = self.advance().span;
            let inner = self.parse_intersection_type_hint(context)?;
            let close_span = self.expect_right_paren()?;
            return Ok(ParsedTypeHint {
                type_hint: inner.type_hint,
                span: combine_spans(open_span, close_span),
            });
        }

        let token = self.peek().clone();
        let type_hint = match &token.kind {
            TokenKind::Null => {
                self.advance();
                TypeHint::Null
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("array") => {
                self.advance();
                TypeHint::Array
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("callable") => {
                self.advance();
                TypeHint::Callable
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("object") => {
                self.advance();
                TypeHint::Object
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("iterable") => {
                self.advance();
                TypeHint::Iterable
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("mixed") => {
                self.advance();
                TypeHint::Mixed
            }
            TokenKind::Identifier(name)
                if context.allows_void() && name.eq_ignore_ascii_case("void") =>
            {
                self.advance();
                TypeHint::Void
            }
            TokenKind::Identifier(name)
                if context.allows_never() && name.eq_ignore_ascii_case("never") =>
            {
                self.advance();
                TypeHint::Never
            }
            TokenKind::IntType | TokenKind::IntegerType => {
                self.advance();
                TypeHint::Int
            }
            TokenKind::FloatType | TokenKind::DoubleType => {
                self.advance();
                TypeHint::Float
            }
            TokenKind::StringType | TokenKind::BinaryType => {
                self.advance();
                TypeHint::String
            }
            TokenKind::BoolType | TokenKind::BooleanType => {
                self.advance();
                TypeHint::Bool
            }
            TokenKind::True => {
                self.advance();
                TypeHint::True
            }
            TokenKind::False => {
                self.advance();
                TypeHint::False
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("self") => {
                self.advance();
                self.relative_type_hint("self", token.span)?
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("parent") => {
                self.advance();
                self.relative_type_hint("parent", token.span)?
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("static") => {
                self.advance();
                self.relative_type_hint("static", token.span)?
            }
            TokenKind::Backslash => {
                let parsed = self.parse_name("expected type hint")?;
                if is_unqualified_only_builtin_type_hint_name(&parsed.name) {
                    return Err(Diagnostic::new(
                        format!("Type declaration '{}' must be unqualified", parsed.name),
                        Some(parsed.span),
                    ));
                }
                TypeHint::Class(self.resolve_class_name(&parsed))
            }
            TokenKind::Identifier(name) if !is_unsupported_builtin_type_hint_name(name) => {
                let parsed = self.parse_name("expected type hint")?;
                TypeHint::Class(self.resolve_class_name(&parsed))
            }
            _ => {
                let token = self.advance();
                return Err(Diagnostic::new("expected type hint", Some(token.span)));
            }
        };
        Ok(ParsedTypeHint {
            type_hint,
            span: token.span,
        })
    }

    fn relative_type_hint(&self, keyword: &str, span: SourceSpan) -> Result<TypeHint> {
        match keyword.to_ascii_lowercase().as_str() {
            "self" => {
                if let Some(scope) = &self.active_type_scope {
                    Ok(TypeHint::Class(scope.class_name.clone()))
                } else if self.allow_unscoped_relative_types > 0 {
                    Ok(TypeHint::Class("self".to_string()))
                } else {
                    Err(Diagnostic::new(
                        "Cannot use \"self\" when no class scope is active",
                        Some(span),
                    ))
                }
            }
            "parent" => {
                if let Some(scope) = &self.active_type_scope {
                    if let Some(parent_name) = &scope.parent_name {
                        Ok(TypeHint::Class(parent_name.clone()))
                    } else if scope.allow_unresolved_parent {
                        Ok(TypeHint::Class("parent".to_string()))
                    } else {
                        Err(Diagnostic::new(
                            "Cannot use \"parent\" when current class scope has no parent",
                            Some(span),
                        ))
                    }
                } else if self.allow_unscoped_relative_types > 0 {
                    Ok(TypeHint::Class("parent".to_string()))
                } else {
                    Err(Diagnostic::new(
                        "Cannot use \"parent\" when no class scope is active",
                        Some(span),
                    ))
                }
            }
            "static" => {
                if self.active_type_scope.is_some() || self.allow_unscoped_relative_types > 0 {
                    Ok(TypeHint::Static)
                } else {
                    Err(Diagnostic::new(
                        "Cannot use \"static\" when no class scope is active",
                        Some(span),
                    ))
                }
            }
            _ => unreachable!("relative type keyword checked by caller"),
        }
    }

    fn peek_is_type_hint(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Question
            | TokenKind::LeftParen
            | TokenKind::Backslash
            | TokenKind::Null
            | TokenKind::IntType
            | TokenKind::IntegerType
            | TokenKind::FloatType
            | TokenKind::DoubleType
            | TokenKind::StringType
            | TokenKind::BinaryType
            | TokenKind::BoolType
            | TokenKind::BooleanType
            | TokenKind::True
            | TokenKind::False => true,
            TokenKind::Identifier(name) => {
                name.eq_ignore_ascii_case("array")
                    || name.eq_ignore_ascii_case("callable")
                    || name.eq_ignore_ascii_case("mixed")
                    || name.eq_ignore_ascii_case("object")
                    || name.eq_ignore_ascii_case("iterable")
                    || name.eq_ignore_ascii_case("self")
                    || name.eq_ignore_ascii_case("parent")
                    || name.eq_ignore_ascii_case("static")
                    || !is_unsupported_builtin_type_hint_name(name)
            }
            _ => false,
        }
    }

    fn peek_starts_class_name(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenKind::Backslash | TokenKind::Identifier(_)
        )
    }

    fn peek_starts_property_type_hint(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Question
            | TokenKind::Backslash
            | TokenKind::Null
            | TokenKind::IntType
            | TokenKind::IntegerType
            | TokenKind::FloatType
            | TokenKind::DoubleType
            | TokenKind::StringType
            | TokenKind::BinaryType
            | TokenKind::BoolType
            | TokenKind::BooleanType => true,
            TokenKind::Identifier(_) => true,
            _ => false,
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
                    target: IncDecTarget::Variable {
                        name,
                        span: token.span,
                    },
                    op: IncDecOp::Increment,
                    span: token.span,
                });
            }
            TokenKind::MinusMinus => {
                self.advance();
                self.expect_statement_terminator()?;
                return Ok(Statement::Increment {
                    target: IncDecTarget::Variable {
                        name,
                        span: token.span,
                    },
                    op: IncDecOp::Decrement,
                    span: token.span,
                });
            }
            _ => {}
        }
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            let target = self.parse_array_dim_target(name, token.span)?;
            match self.peek().kind {
                TokenKind::PlusPlus => {
                    self.advance();
                    self.expect_statement_terminator()?;
                    let target = IncDecTarget::ArrayDim(target);
                    reject_append_array_read_in_inc_dec_target(&target)?;
                    return Ok(Statement::Increment {
                        target,
                        op: IncDecOp::Increment,
                        span: token.span,
                    });
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    self.expect_statement_terminator()?;
                    let target = IncDecTarget::ArrayDim(target);
                    reject_append_array_read_in_inc_dec_target(&target)?;
                    return Ok(Statement::Increment {
                        target,
                        op: IncDecOp::Decrement,
                        span: token.span,
                    });
                }
                _ => {}
            }
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
                let value = self.parse_assignment_expr_without_keyword_boolean()?;
                if let Some(statement) = self.parse_keyword_boolean_assignment_tail_statement(
                    AssignmentTarget::ArrayDim(target.clone()),
                    op,
                    value.clone(),
                )? {
                    return Ok(statement);
                }
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
                self.expect_statement_terminator()?;
                return Ok(Statement::ArrayAssignRef {
                    target,
                    source,
                    span: token.span,
                });
            }
            let value = self.parse_assignment_expr_without_keyword_boolean()?;
            if matches!(op, AssignmentOp::Assign) {
                validate_recursive_reference_assignment_value(
                    &AssignmentTarget::ArrayDim(target.clone()),
                    &value,
                )?;
            }
            if let Some(statement) = self.parse_keyword_boolean_assignment_tail_statement(
                AssignmentTarget::ArrayDim(target.clone()),
                op,
                value.clone(),
            )? {
                return Ok(statement);
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
            self.expect_statement_terminator()?;
            return Ok(Statement::AssignRef {
                name,
                source,
                span: token.span,
            });
        }
        let value = self.parse_assignment_expr_without_keyword_boolean()?;
        let target = AssignmentTarget::Variable {
            name: name.clone(),
            span: token.span,
        };
        if matches!(op, AssignmentOp::Assign) {
            validate_recursive_reference_assignment_value(&target, &value)?;
        }
        if let Some(statement) =
            self.parse_keyword_boolean_assignment_tail_statement(target, op, value.clone())?
        {
            return Ok(statement);
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
        let target = inc_dec_target_from_expr(self.parse_postfix_expr()?, op_token.span)?;
        reject_append_array_read_in_inc_dec_target(&target)?;
        self.expect_statement_terminator()?;
        Ok(Statement::Increment {
            target,
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

    fn parse_declare_statement(&mut self) -> Result<Statement> {
        let start_span = self.advance().span;
        self.expect_left_paren()?;
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            loop {
                let name_token = self.advance().clone();
                let TokenKind::Identifier(name) = name_token.kind else {
                    return Err(syntax_error_unexpected(&name_token, Some("identifier")));
                };
                self.expect_equal()?;
                let value = self.parse_declare_literal_value()?;
                if name.eq_ignore_ascii_case("strict_types") {
                    self.strict_types = value != 0;
                }
                if matches!(self.peek().kind, TokenKind::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        self.expect_right_paren()?;
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            return self.parse_compound_block();
        }
        self.expect_statement_terminator()?;
        Ok(Statement::Empty { span: start_span })
    }

    fn parse_declare_literal_value(&mut self) -> Result<i64> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Int(value) => Ok(value),
            TokenKind::True => Ok(1),
            TokenKind::False => Ok(0),
            _ => Err(syntax_error_unexpected(&token, Some("literal"))),
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        if let Some(void_span) = self.try_parse_void_cast_statement_prefix() {
            let expression = self.parse_expr()?;
            let span = combine_spans(void_span, expression.span());
            let expression = Expr::Cast {
                kind: CastKind::Void,
                expr: Box::new(expression),
                span,
            };
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
            return Ok(Statement::Expression { expression, span });
        }

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

    fn try_parse_void_cast_statement_prefix(&mut self) -> Option<SourceSpan> {
        if !matches!(self.peek().kind, TokenKind::LeftParen) || !self.peek_next_is_void_cast_name()
        {
            return None;
        }
        if !matches!(
            self.tokens.get(self.index + 2).map(|token| &token.kind),
            Some(TokenKind::RightParen)
        ) {
            return None;
        }

        let left = self.advance().span;
        self.advance();
        let right = self.advance().span;
        Some(combine_spans(left, right))
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

    fn parse_global(&mut self) -> Result<Statement> {
        let span = self.advance().span;
        let mut names = Vec::new();
        loop {
            let token = self.advance().clone();
            let TokenKind::Variable(name) = token.kind else {
                return Err(Diagnostic::new(
                    "expected global variable",
                    Some(token.span),
                ));
            };
            names.push(name);
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_statement_terminator()?;
        Ok(Statement::Global { names, span })
    }

    fn parse_static_local(&mut self) -> Result<Statement> {
        let span = self.advance().span;
        if self.function_depth == 0 {
            return Err(Diagnostic::new(
                "static local variables must be declared inside a function",
                Some(span),
            ));
        }

        let mut declarations = vec![self.parse_static_local_declaration()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            declarations.push(self.parse_static_local_declaration()?);
        }
        self.expect_statement_terminator()?;
        Ok(Statement::Static { declarations, span })
    }

    fn parse_static_local_declaration(&mut self) -> Result<StaticLocalDeclaration> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new(
                "expected static local variable",
                Some(token.span),
            ));
        };
        let value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        Ok(StaticLocalDeclaration {
            name,
            value,
            span: token.span,
        })
    }

    fn parse_const_declaration(&mut self) -> Result<ConstDeclaration> {
        let (name, token_span) = self.parse_declaration_name("expected constant name")?;
        self.expect_equal()?;
        let value = self.parse_expr()?;
        if !is_supported_const_declaration_expr(&value) {
            return Err(Diagnostic::new(
                "constant expression contains invalid operation",
                Some(value.span()),
            ));
        }
        Ok(ConstDeclaration {
            name,
            value,
            span: token_span,
        })
    }

    fn parse_if(&mut self) -> Result<Statement> {
        let span = self.expect_if_like()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        let then_body = self.parse_statement_body()?;
        self.skip_php_tags();
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
        self.skip_php_tags();
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
            if matches!(first.target, AssignmentTarget::List(_)) {
                return Err(Diagnostic::new(
                    "Cannot use list as key element",
                    Some(assignment_target_span(&first.target)),
                ));
            }
            self.advance();
            let value = self.parse_foreach_variable()?;
            (Some(first.target), value.target, value.by_ref)
        } else {
            (None, first.target, first.by_ref)
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
        let target_expr = self.parse_expr()?;
        let target_span = target_expr.span();
        let target = assignment_target_from_expr(target_expr).map_err(|diagnostic| {
            if diagnostic.message == "Cannot use empty list" {
                diagnostic
            } else {
                Diagnostic::new("expected foreach variable", Some(target_span))
            }
        })?;
        if by_ref {
            validate_foreach_by_reference_target(&target, span)?;
        }
        Ok(ForeachVariable {
            target,
            by_ref,
            span,
        })
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
        let start = self.index;
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected variable", Some(token.span)));
        };
        if matches!(self.peek().kind, TokenKind::LeftBracket) {
            let target = self.parse_array_dim_target(name, token.span)?;
            return match self.peek().kind {
                TokenKind::PlusPlus => {
                    self.advance();
                    let target = IncDecTarget::ArrayDim(target);
                    reject_append_array_read_in_inc_dec_target(&target)?;
                    Ok(Statement::Increment {
                        target,
                        op: IncDecOp::Increment,
                        span: token.span,
                    })
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    let target = IncDecTarget::ArrayDim(target);
                    reject_append_array_read_in_inc_dec_target(&target)?;
                    Ok(Statement::Increment {
                        target,
                        op: IncDecOp::Decrement,
                        span: token.span,
                    })
                }
                _ => Err(Diagnostic::new(
                    "array offset for clauses currently support increment/decrement updates",
                    Some(target.span),
                )),
            };
        }
        match self.peek().kind {
            TokenKind::PlusPlus => {
                self.advance();
                Ok(Statement::Increment {
                    target: IncDecTarget::Variable {
                        name,
                        span: token.span,
                    },
                    op: IncDecOp::Increment,
                    span: token.span,
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                Ok(Statement::Increment {
                    target: IncDecTarget::Variable {
                        name,
                        span: token.span,
                    },
                    op: IncDecOp::Decrement,
                    span: token.span,
                })
            }
            _ => {
                if !self.peek_is_assignment_op() {
                    self.index = start;
                    let expression = self.parse_expr()?;
                    let span = expression.span();
                    return Ok(Statement::Expression { expression, span });
                }
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
        let target = inc_dec_target_from_expr(self.parse_postfix_expr()?, op_token.span)?;
        reject_append_array_read_in_inc_dec_target(&target)?;
        Ok(Statement::Increment {
            target,
            op,
            span: op_token.span,
        })
    }

    fn parse_call_clause(&mut self) -> Result<Statement> {
        let (name, token_span) = self.parse_resolved_function_name("expected function name")?;
        let (arguments, argument_names, argument_unpacks, _) = self.parse_call_arguments()?;
        validate_mutating_array_internal_call(&name, &arguments, token_span)?;
        Ok(Statement::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            span: token_span,
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
            self.skip_php_tags();
            if matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
                break;
            }
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
                self.skip_php_tags();
                if matches!(
                    self.peek().kind,
                    TokenKind::Case | TokenKind::Default | TokenKind::RightBrace | TokenKind::Eof
                ) {
                    break;
                }
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
        let level = self.parse_control_transfer_level("break")?;
        self.expect_statement_terminator()?;
        Ok(Statement::Break { level, span })
    }

    fn parse_continue(&mut self) -> Result<Statement> {
        let span = self.expect_continue()?;
        let level = self.parse_control_transfer_level("continue")?;
        self.expect_statement_terminator()?;
        Ok(Statement::Continue { level, span })
    }

    fn parse_control_transfer_level(&mut self, keyword: &str) -> Result<usize> {
        let level = match self.peek().kind {
            TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof => 1,
            TokenKind::Int(value) => {
                let span = self.advance().span;
                if value <= 0 {
                    return Err(Diagnostic::new(
                        format!("'{keyword}' operator accepts only positive integers"),
                        Some(span),
                    ));
                }
                if !matches!(
                    self.peek().kind,
                    TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof
                ) {
                    return Err(Diagnostic::new(
                        format!(
                            "'{keyword}' operator with non-integer operand is no longer supported"
                        ),
                        Some(self.peek().span),
                    ));
                }
                value as usize
            }
            _ => {
                return Err(Diagnostic::new(
                    format!("'{keyword}' operator with non-integer operand is no longer supported"),
                    Some(self.peek().span),
                ));
            }
        };
        Ok(level)
    }

    fn parse_return(&mut self) -> Result<Statement> {
        let span = self.expect_return()?;
        let value = if matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof
        ) {
            None
        } else {
            let previous = self.allow_append_array_read;
            if self.return_by_ref_stack.last().copied().unwrap_or(false) {
                self.allow_append_array_read = true;
            }
            let value = self.parse_expr();
            self.allow_append_array_read = previous;
            Some(value?)
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Return { value, span })
    }

    fn parse_throw_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let value = self.parse_expr()?;
        let span = combine_spans(token.span, value.span());
        self.expect_statement_terminator()?;
        Ok(Statement::Throw { value, span })
    }

    fn parse_try(&mut self) -> Result<Statement> {
        let span = self.expect_try()?;
        let body = self.parse_block()?;
        let mut catches = Vec::new();
        while matches!(self.peek().kind, TokenKind::Catch) {
            catches.push(self.parse_catch_clause()?);
        }
        let mut has_finally = false;
        let finally_body = if token_is_identifier_named(self.peek(), "finally") {
            has_finally = true;
            self.advance();
            self.parse_block()?
        } else {
            Vec::new()
        };
        if catches.is_empty() && !has_finally {
            return Err(Diagnostic::new(
                "try without catch or finally is unsupported",
                Some(span),
            ));
        }
        Ok(Statement::Try {
            body,
            catches,
            finally_body,
            span,
        })
    }

    fn parse_catch_clause(&mut self) -> Result<CatchClause> {
        let span = self.expect_catch()?;
        self.expect_left_paren()?;
        let type_names = self.parse_catch_type_names()?;
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
            type_names,
            variable,
            body,
            span,
        })
    }

    fn parse_catch_type_names(&mut self) -> Result<Vec<String>> {
        let mut names = vec![
            self.parse_resolved_class_name("expected catch type name")?
                .0,
        ];
        while matches!(self.peek().kind, TokenKind::Pipe) {
            self.advance();
            names.push(
                self.parse_resolved_class_name("expected catch type name")?
                    .0,
            );
        }
        Ok(names)
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
        let start_index = self.index;
        let (name, span) = self.parse_resolved_function_name("expected function name")?;
        let (arguments, argument_names, argument_unpacks, _) = self.parse_call_arguments()?;
        if !matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof
        ) {
            self.index = start_index;
            return self.parse_expression_statement();
        }
        validate_mutating_array_internal_call(&name, &arguments, span)?;
        self.expect_statement_terminator()?;
        Ok(Statement::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            span,
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
            Expr::DynamicVariable { name, span } => Ok(UnsetTarget::DynamicVariable { name, span }),
            Expr::ArrayAccess { .. } => unset_array_dim_target_from_expr(target),
            Expr::PropertyFetch {
                receiver,
                name,
                span,
            } => Ok(UnsetTarget::Property {
                receiver,
                name,
                span,
            }),
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
            self.skip_php_tags();
            if matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_nested_statement()?);
        }
        self.expect_right_brace()?;
        Ok(statements)
    }

    fn parse_statement_body(&mut self) -> Result<Vec<Statement>> {
        self.skip_php_tags();
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            self.parse_block()
        } else {
            Ok(vec![self.parse_nested_statement()?])
        }
    }

    fn parse_nested_statement(&mut self) -> Result<Statement> {
        self.skip_php_tags();
        self.block_depth += 1;
        let statement = self.parse_statement();
        self.block_depth -= 1;
        statement
    }

    fn skip_php_tags(&mut self) {
        while matches!(self.peek().kind, TokenKind::OpenTag | TokenKind::CloseTag) {
            self.advance();
        }
    }

    fn parse_attribute_groups(&mut self) -> Result<ParsedAttributes> {
        let mut attributes = ParsedAttributes::default();
        while matches!(self.peek().kind, TokenKind::AttributeStart) {
            let group = self.parse_attribute_group()?;
            merge_parsed_attributes(&mut attributes, group);
        }
        Ok(attributes)
    }

    fn parse_attribute_group(&mut self) -> Result<ParsedAttributes> {
        let start = self.advance().span;
        let mut bracket_depth = 1usize;
        let mut paren_depth = 0usize;
        let mut name_segments = Vec::new();
        let mut arguments = ParsedAttributeArguments::default();
        let mut pending_argument_name = None;
        let mut collecting_name = true;
        let mut attributes = ParsedAttributes::default();
        while bracket_depth > 0 {
            let token = self.advance().clone();
            match token.kind {
                TokenKind::Identifier(name)
                    if bracket_depth == 1 && paren_depth == 0 && collecting_name =>
                {
                    name_segments.push(name);
                }
                TokenKind::Backslash
                    if bracket_depth == 1 && paren_depth == 0 && collecting_name => {}
                TokenKind::LeftParen if bracket_depth == 1 => {
                    collecting_name = false;
                    paren_depth += 1;
                }
                TokenKind::Identifier(name)
                    if bracket_depth == 1
                        && paren_depth == 1
                        && matches!(self.peek().kind, TokenKind::Colon) =>
                {
                    pending_argument_name = Some(name);
                }
                TokenKind::String(value) if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record(pending_argument_name.take(), value);
                }
                TokenKind::RightParen if bracket_depth == 1 && paren_depth > 0 => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        pending_argument_name = None;
                    }
                }
                TokenKind::Comma if bracket_depth == 1 && paren_depth == 1 => {
                    pending_argument_name = None;
                }
                TokenKind::Comma if bracket_depth == 1 && paren_depth == 0 => {
                    apply_parsed_attribute(&mut attributes, &name_segments, &arguments);
                    name_segments.clear();
                    arguments = ParsedAttributeArguments::default();
                    pending_argument_name = None;
                    collecting_name = true;
                }
                TokenKind::LeftBracket => bracket_depth += 1,
                TokenKind::RightBracket => {
                    if bracket_depth == 1 && paren_depth == 0 {
                        apply_parsed_attribute(&mut attributes, &name_segments, &arguments);
                    }
                    bracket_depth -= 1;
                }
                TokenKind::Eof => {
                    return Err(Diagnostic::new("unterminated attribute", Some(start)));
                }
                _ => {}
            }
        }
        Ok(attributes)
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> Result<Expr> {
        let left = self.parse_ternary_expr(0)?;
        self.parse_assignment_expr_from_left(left)
    }

    fn parse_assignment_expr_without_keyword_boolean(&mut self) -> Result<Expr> {
        let left = self.parse_ternary_expr(SYMBOL_OR_PRECEDENCE)?;
        self.parse_assignment_expr_from_left(left)
    }

    fn parse_assignment_expr_from_left(&mut self, left: Expr) -> Result<Expr> {
        if !self.peek_is_expression_assignment_op() {
            if !self.allow_append_array_read {
                reject_append_array_read(&left)?;
            }
            reject_array_literal_holes(&left)?;
            return Ok(left);
        }

        if let Expr::Unary {
            op: UnaryOp::ErrorSuppress,
            expr,
            span: suppress_span,
        } = left
        {
            let assignment = self.parse_assignment_expr_from_left(*expr)?;
            let span = combine_spans(suppress_span, assignment.span());
            return Ok(Expr::Unary {
                op: UnaryOp::ErrorSuppress,
                expr: Box::new(assignment),
                span,
            });
        }

        let operator = self.peek().clone();
        let op = self.expect_assignment_op()?;
        let left_span = left.span();
        let target = match assignment_target_from_expr(left) {
            Ok(target) => target,
            Err(diagnostic)
                if diagnostic.message == "Spread operator is not supported in assignments" =>
            {
                return Err(diagnostic);
            }
            Err(_) => {
                return Err(Diagnostic::new(
                    "assignment expression target must be a variable, array dimension, or list",
                    Some(operator.span),
                ));
            }
        };
        validate_expression_assignment_target(op, &target, operator.span)?;
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
        let value = self.parse_assignment_expr_without_keyword_boolean()?;
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
        let value = self.parse_assignment_expr_without_keyword_boolean()?;
        if self.peek_is_keyword_boolean_operator() {
            return Err(Diagnostic::new(
                "assignment expressions with keyword boolean operators are unsupported",
                Some(self.peek().span),
            ));
        }
        Ok(value)
    }

    fn parse_keyword_boolean_tail_from_left(
        &mut self,
        mut left: Expr,
        min_precedence: u8,
    ) -> Result<Expr> {
        while let Some((op, precedence)) = self.peek_keyword_boolean_op() {
            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_assignment_expr_without_keyword_boolean()?;
            let right = self.parse_keyword_boolean_tail_from_left(right, precedence + 1)?;
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

    fn parse_keyword_boolean_assignment_tail_statement(
        &mut self,
        target: AssignmentTarget,
        op: AssignmentOp,
        value: Expr,
    ) -> Result<Option<Statement>> {
        if !self.peek_is_keyword_boolean_operator() {
            return Ok(None);
        }

        let span = combine_spans(assignment_target_span(&target), value.span());
        let assignment = Expr::Assign {
            target,
            op,
            value: Box::new(value),
            span,
        };
        let expression =
            self.parse_keyword_boolean_tail_from_left(assignment, KEYWORD_OR_PRECEDENCE)?;
        let span = expression.span();
        self.expect_statement_terminator()?;
        Ok(Some(Statement::Expression { expression, span }))
    }

    fn parse_assignment_expr_without_ternary(&mut self, binary_min_precedence: u8) -> Result<Expr> {
        let left = self.parse_binary_expr(binary_min_precedence)?;
        self.parse_assignment_expr_from_left(left)
    }

    fn parse_ternary_expr(&mut self, binary_min_precedence: u8) -> Result<Expr> {
        let condition = self.parse_binary_expr(binary_min_precedence)?;
        if !matches!(self.peek().kind, TokenKind::Question) {
            return Ok(condition);
        }

        let question = self.advance().clone();
        let (if_true, first_is_short) = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            (None, true)
        } else {
            let value = self.parse_assignment_expr_without_ternary(0)?;
            if matches!(self.peek().kind, TokenKind::Question) {
                return Err(Diagnostic::new(
                    nested_ternary_message(false, self.peek_next_is_colon()),
                    Some(question.span),
                ));
            }
            self.expect_colon()?;
            (Some(Box::new(value)), false)
        };
        let if_false = self.parse_assignment_expr_without_ternary(0)?;
        if matches!(self.peek().kind, TokenKind::Question) {
            return Err(Diagnostic::new(
                nested_ternary_message(first_is_short, self.peek_next_is_colon()),
                Some(question.span),
            ));
        }

        let span = combine_spans(condition.span(), if_false.span());
        Ok(Expr::Ternary {
            condition: Box::new(condition),
            if_true,
            if_false: Box::new(if_false),
            span,
        })
    }

    fn parse_binary_expr(&mut self, min_precedence: u8) -> Result<Expr> {
        let mut left = self.parse_unary_expr()?;
        loop {
            if token_is_identifier_named(self.peek(), "instanceof") {
                if COMPARISON_PRECEDENCE < min_precedence {
                    break;
                }
                self.advance();
                let target = if self.peek_starts_class_name() {
                    let (name, span) = self.parse_resolved_class_name("expected class name")?;
                    InstanceOfTarget::ClassName { name, span }
                } else {
                    InstanceOfTarget::Expr(Box::new(self.parse_unary_expr()?))
                };
                let span = combine_spans(left.span(), target.span());
                left = Expr::InstanceOf {
                    expr: Box::new(left),
                    target,
                    span,
                };
                continue;
            }

            if matches!(self.peek().kind, TokenKind::PipeGreater) {
                if PIPE_PRECEDENCE < min_precedence {
                    break;
                }
                self.advance();
                let input_span = left.span();
                let next_min_precedence = PIPE_PRECEDENCE + 1;
                let callee = self.parse_binary_expr(next_min_precedence)?;
                let span = combine_spans(input_span, callee.span());
                left = Expr::DynamicCall {
                    callee: Box::new(callee),
                    arguments: vec![Expr::PipeValue {
                        expr: Box::new(left),
                        span: input_span,
                    }],
                    argument_names: vec![None],
                    argument_unpacks: vec![false],
                    span,
                };
                continue;
            }

            let Some((op, precedence, right_associative)) = self.peek_binary_op() else {
                break;
            };
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
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_increment_expr(),
            TokenKind::Print => self.parse_print_expr(),
            TokenKind::Include => self.parse_include_expr(IncludeKind::Include),
            TokenKind::IncludeOnce => self.parse_include_expr(IncludeKind::IncludeOnce),
            TokenKind::Require => self.parse_include_expr(IncludeKind::Require),
            TokenKind::RequireOnce => self.parse_include_expr(IncludeKind::RequireOnce),
            TokenKind::Throw => self.parse_throw_expr(),
            TokenKind::Match => {
                let token = self.advance().clone();
                self.parse_match_expr(token.span)
            }
            TokenKind::Clone => self.parse_clone_expr(),
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

    fn parse_clone_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        let expr = self.parse_assignment_expr()?;
        let span = combine_spans(token.span, expr.span());
        Ok(Expr::Clone {
            expr: Box::new(expr),
            span,
        })
    }

    fn parse_throw_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        let value = self.parse_assignment_expr()?;
        let span = combine_spans(token.span, value.span());
        Ok(Expr::Throw {
            value: Box::new(value),
            span,
        })
    }

    fn parse_include_expr(&mut self, kind: IncludeKind) -> Result<Expr> {
        let token = self.advance().clone();
        let path = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();
            let path = self.parse_expr()?;
            let right_span = self.expect_right_paren()?;
            let span = combine_spans(token.span, right_span);
            return Ok(Expr::Include {
                kind,
                path: Box::new(path),
                span,
            });
        } else {
            self.parse_assignment_expr()?
        };
        let span = combine_spans(token.span, path.span());
        Ok(Expr::Include {
            kind,
            path: Box::new(path),
            span,
        })
    }

    fn parse_print_expr(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        let expression = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();
            let expression = self.parse_expr()?;
            let right_span = self.expect_right_paren()?;
            let span = combine_spans(token.span, right_span);
            return Ok(Expr::Print {
                expression: Box::new(expression),
                span,
            });
        } else {
            self.parse_assignment_expr()?
        };
        let span = combine_spans(token.span, expression.span());
        Ok(Expr::Print {
            expression: Box::new(expression),
            span,
        })
    }

    fn parse_prefix_increment_expr(&mut self) -> Result<Expr> {
        let op_token = self.advance().clone();
        let op = match op_token.kind {
            TokenKind::PlusPlus => IncDecOp::Increment,
            TokenKind::MinusMinus => IncDecOp::Decrement,
            _ => return Err(Diagnostic::new("expected increment", Some(op_token.span))),
        };
        let target = inc_dec_target_from_expr(self.parse_postfix_expr()?, op_token.span)?;
        reject_append_array_read_in_inc_dec_target(&target)?;
        let target_span = inc_dec_target_span(&target);
        Ok(Expr::IncDec {
            target,
            op,
            result: IncDecResult::Pre,
            span: combine_spans(op_token.span, target_span),
        })
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
                TokenKind::ObjectOperator | TokenKind::NullsafeObjectOperator => {
                    let nullsafe = matches!(self.peek().kind, TokenKind::NullsafeObjectOperator);
                    let start_span = expr.span();
                    self.advance();
                    let member = self.advance().clone();
                    let (literal_name, dynamic_name, member_span) = match member.kind {
                        TokenKind::Identifier(name) => (Some(name), None, member.span),
                        TokenKind::Variable(name) => {
                            (None, Some(Expr::Variable(name, member.span)), member.span)
                        }
                        TokenKind::LeftBrace => {
                            let name_expr = self.parse_expr()?;
                            let right_span = self.expect_right_brace()?;
                            let member_span = combine_spans(member.span, right_span);
                            match literal_member_name_from_expr(&name_expr) {
                                Some(name) => (Some(name), None, member_span),
                                None => (None, Some(name_expr), member_span),
                            }
                        }
                        kind => match object_member_name_from_token(&kind) {
                            Some(name) => (Some(name), None, member.span),
                            None => {
                                return Err(Diagnostic::new(
                                    "expected member name",
                                    Some(member.span),
                                ));
                            }
                        },
                    };
                    if !matches!(self.peek().kind, TokenKind::LeftParen) {
                        let span = combine_spans(start_span, member_span);
                        expr = if let Some(name) = literal_name {
                            if nullsafe {
                                Expr::NullsafePropertyFetch {
                                    receiver: Box::new(expr),
                                    name,
                                    span,
                                }
                            } else {
                                Expr::PropertyFetch {
                                    receiver: Box::new(expr),
                                    name,
                                    span,
                                }
                            }
                        } else {
                            if nullsafe {
                                return Err(Diagnostic::new(
                                    "dynamic nullsafe property fetches are unsupported",
                                    Some(member_span),
                                ));
                            }
                            Expr::DynamicPropertyFetch {
                                receiver: Box::new(expr),
                                name: Box::new(dynamic_name.expect("dynamic member name")),
                                span,
                            }
                        };
                        continue;
                    }
                    if nullsafe {
                        return Err(Diagnostic::new(
                            "nullsafe method calls are unsupported",
                            Some(member_span),
                        ));
                    }
                    if self.peek_is_first_class_callable_arguments() {
                        let right_span = self.parse_first_class_callable_arguments()?;
                        let callable_span = combine_spans(start_span, member_span);
                        let callable = Expr::Array {
                            elements: vec![
                                ArrayElement {
                                    key: None,
                                    value: ArrayElementValue::Value(expr),
                                    line: start_span.line,
                                },
                                ArrayElement {
                                    key: None,
                                    value: ArrayElementValue::Value(literal_name.map_or_else(
                                        || dynamic_name.clone().expect("dynamic member name"),
                                        |name| Expr::String(name, member_span),
                                    )),
                                    line: member_span.line,
                                },
                            ],
                            span: callable_span,
                        };
                        expr = Expr::FirstClassCallable {
                            callable: Box::new(callable),
                            span: combine_spans(start_span, right_span),
                        };
                        continue;
                    }
                    let (arguments, argument_names, argument_unpacks, right_span) =
                        self.parse_call_arguments()?;
                    let span = combine_spans(start_span, right_span);
                    expr = if let Some(name) = literal_name {
                        Expr::MethodCall {
                            receiver: Box::new(expr),
                            name,
                            arguments,
                            argument_names,
                            argument_unpacks,
                            span,
                        }
                    } else {
                        Expr::DynamicMethodCall {
                            receiver: Box::new(expr),
                            name: Box::new(dynamic_name.expect("dynamic member expression")),
                            arguments,
                            argument_names,
                            argument_unpacks,
                            span,
                        }
                    };
                }
                TokenKind::LeftParen => {
                    let start_span = expr.span();
                    if self.peek_is_first_class_callable_arguments() {
                        let right_span = self.parse_first_class_callable_arguments()?;
                        expr = Expr::FirstClassCallable {
                            callable: Box::new(expr),
                            span: combine_spans(start_span, right_span),
                        };
                        continue;
                    }
                    let (arguments, argument_names, argument_unpacks, right_span) =
                        self.parse_call_arguments()?;
                    expr = Expr::DynamicCall {
                        callee: Box::new(expr),
                        arguments,
                        argument_names,
                        argument_unpacks,
                        span: combine_spans(start_span, right_span),
                    };
                }
                TokenKind::DoubleColon => {
                    let start_span = expr.span();
                    let scope_span = self.advance().span;
                    let member = self.advance().clone();
                    let (literal_name, dynamic_name, member_span) = match member.kind {
                        TokenKind::Identifier(member_name) => {
                            if member_name.eq_ignore_ascii_case("class")
                                && !matches!(self.peek().kind, TokenKind::LeftParen)
                            {
                                if dynamic_class_name_fetch_has_illegal_literal_receiver(&expr) {
                                    return Err(Diagnostic::new(
                                        "Illegal class name",
                                        Some(start_span),
                                    ));
                                }
                                expr = Expr::DynamicClassNameFetch {
                                    receiver: Box::new(expr),
                                    span: combine_spans(start_span, member.span),
                                };
                                continue;
                            }
                            (Some(member_name), None, member.span)
                        }
                        TokenKind::Variable(name) => {
                            (None, Some(Expr::Variable(name, member.span)), member.span)
                        }
                        TokenKind::LeftBrace => {
                            let name_expr = self.parse_expr()?;
                            let right_span = self.expect_right_brace()?;
                            let member_span = combine_spans(member.span, right_span);
                            match literal_member_name_from_expr(&name_expr) {
                                Some(name) => (Some(name), None, member_span),
                                None => (None, Some(name_expr), member_span),
                            }
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                CLASS_CONSTANT_FETCH_UNSUPPORTED,
                                Some(scope_span),
                            ));
                        }
                    };
                    if !matches!(self.peek().kind, TokenKind::LeftParen) {
                        return Err(Diagnostic::new(
                            CLASS_CONSTANT_FETCH_UNSUPPORTED,
                            Some(scope_span),
                        ));
                    }
                    if self.peek_is_first_class_callable_arguments() {
                        return Err(Diagnostic::new(
                            "dynamic first-class static method callables are unsupported",
                            Some(member_span),
                        ));
                    }
                    let receiver_span = expr.span();
                    let class_name_fetch = Expr::DynamicClassNameFetch {
                        receiver: Box::new(expr),
                        span: combine_spans(start_span, receiver_span),
                    };
                    let method_name = literal_name.map_or_else(
                        || dynamic_name.expect("dynamic static member expression"),
                        |name| Expr::String(name, member_span),
                    );
                    expr = self.parse_dynamic_static_method_call_expr(
                        class_name_fetch,
                        start_span,
                        method_name,
                    )?;
                }
                TokenKind::PlusPlus | TokenKind::MinusMinus => {
                    let op_token = self.advance().clone();
                    let op = match op_token.kind {
                        TokenKind::PlusPlus => IncDecOp::Increment,
                        TokenKind::MinusMinus => IncDecOp::Decrement,
                        _ => {
                            return Err(Diagnostic::new("expected increment", Some(op_token.span)));
                        }
                    };
                    let target = inc_dec_target_from_expr(expr, op_token.span)?;
                    reject_append_array_read_in_inc_dec_target(&target)?;
                    let target_span = inc_dec_target_span(&target);
                    expr = Expr::IncDec {
                        target,
                        op,
                        result: IncDecResult::Post,
                        span: combine_spans(target_span, op_token.span),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let attributes = self.parse_attribute_groups()?;
        let token = self.advance().clone();
        match token.kind {
            TokenKind::String(value) => Ok(Expr::String(value, token.span)),
            TokenKind::InterpolatedString(parts) => Ok(Expr::InterpolatedString(
                parts.into_iter().map(lower_string_part).collect(),
                token.span,
            )),
            TokenKind::BacktickString(command) => Ok(Expr::ShellExec {
                command,
                span: token.span,
            }),
            TokenKind::Int(value) => Ok(Expr::Int(value, token.span)),
            TokenKind::Float(value) => Ok(Expr::Float(value, token.span)),
            TokenKind::True => Ok(Expr::Bool(true, token.span)),
            TokenKind::False => Ok(Expr::Bool(false, token.span)),
            TokenKind::Null => Ok(Expr::Null(token.span)),
            TokenKind::Variable(name) => Ok(Expr::Variable(name, token.span)),
            TokenKind::Dollar => self.parse_dynamic_variable_expr(token.span),
            TokenKind::Function => {
                self.parse_anonymous_function_expr(token.span, false, attributes)
            }
            TokenKind::New => self.parse_new_object_expr(token.span),
            TokenKind::Yield => self.parse_yield_expr(token.span),
            TokenKind::Identifier(name) => {
                if name.eq_ignore_ascii_case("fn")
                    && matches!(
                        self.peek().kind,
                        TokenKind::LeftParen | TokenKind::Ampersand
                    )
                {
                    return self.parse_arrow_function_expr(token.span, false, attributes);
                }
                if name.eq_ignore_ascii_case("static") && self.peek_is_identifier("fn") {
                    let fn_span = self.advance().span;
                    return self.parse_arrow_function_expr(
                        combine_spans(token.span, fn_span),
                        true,
                        attributes,
                    );
                }
                if name.eq_ignore_ascii_case("static")
                    && matches!(self.peek().kind, TokenKind::Function)
                {
                    let function_span = self.advance().span;
                    return self.parse_anonymous_function_expr(
                        combine_spans(token.span, function_span),
                        true,
                        attributes,
                    );
                }
                if name.eq_ignore_ascii_case("match") {
                    return self.parse_match_expr(token.span);
                }
                let parsed_name =
                    self.parse_name_from_first(name, token.span, None, "expected name")?;
                let unqualified = matches!(parsed_name.resolution, NameResolution::Unqualified);
                let lowercase = parsed_name.name.to_ascii_lowercase();
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    let class_name = self.resolve_class_name(&parsed_name);
                    self.parse_static_member_expr(class_name, parsed_name.span)
                } else if matches!(self.peek().kind, TokenKind::LeftParen) {
                    if self.peek_is_first_class_callable_arguments() {
                        let right_span = self.parse_first_class_callable_arguments()?;
                        let resolved_name = self.resolve_function_name(&parsed_name);
                        return Ok(Expr::FirstClassCallable {
                            callable: Box::new(Expr::String(resolved_name, parsed_name.span)),
                            span: combine_spans(parsed_name.span, right_span),
                        });
                    }
                    match (unqualified, lowercase.as_str()) {
                        (true, "array") => self.parse_long_array_literal(parsed_name.span),
                        (true, "list") => self.parse_long_list_expr(parsed_name.span),
                        (true, "isset") => self.parse_isset_expr(parsed_name.span),
                        (true, "empty") => self.parse_empty_expr(parsed_name.span),
                        _ => {
                            let (arguments, argument_names, argument_unpacks, right_span) =
                                self.parse_call_arguments()?;
                            let resolved_name = self.resolve_function_name(&parsed_name);
                            validate_mutating_array_internal_call(
                                &resolved_name,
                                &arguments,
                                parsed_name.span,
                            )?;
                            Ok(Expr::Call {
                                name: resolved_name,
                                arguments,
                                argument_names,
                                argument_unpacks,
                                span: combine_spans(parsed_name.span, right_span),
                            })
                        }
                    }
                } else if unqualified
                    && lowercase == "__namespace__"
                    && self.current_namespace.is_some()
                {
                    Ok(Expr::String(
                        self.current_namespace.clone().unwrap_or_default(),
                        parsed_name.span,
                    ))
                } else if unqualified {
                    if let Some(kind) = magic_constant_kind(&parsed_name.name) {
                        Ok(Expr::MagicConstant(kind, parsed_name.span))
                    } else {
                        Ok(Expr::Constant(
                            self.resolve_constant_name(&parsed_name),
                            parsed_name.span,
                        ))
                    }
                } else {
                    Ok(Expr::Constant(
                        self.resolve_constant_name(&parsed_name),
                        parsed_name.span,
                    ))
                }
            }
            TokenKind::Backslash => {
                let first_token = self.advance().clone();
                let TokenKind::Identifier(first_segment) = first_token.kind else {
                    return Err(Diagnostic::new(
                        "expected fully qualified name",
                        Some(first_token.span),
                    ));
                };
                let parsed_name = self.parse_name_from_first(
                    first_segment,
                    first_token.span,
                    Some(token.span),
                    "expected fully qualified name",
                )?;
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    return self.parse_static_member_expr(
                        self.resolve_class_name(&parsed_name),
                        parsed_name.span,
                    );
                }
                if matches!(self.peek().kind, TokenKind::LeftParen) {
                    if self.peek_is_first_class_callable_arguments() {
                        let right_span = self.parse_first_class_callable_arguments()?;
                        let resolved_name = self.resolve_function_name(&parsed_name);
                        return Ok(Expr::FirstClassCallable {
                            callable: Box::new(Expr::String(resolved_name, parsed_name.span)),
                            span: combine_spans(parsed_name.span, right_span),
                        });
                    }
                    let (arguments, argument_names, argument_unpacks, right_span) =
                        self.parse_call_arguments()?;
                    let resolved_name = self.resolve_function_name(&parsed_name);
                    validate_mutating_array_internal_call(
                        &resolved_name,
                        &arguments,
                        parsed_name.span,
                    )?;
                    return Ok(Expr::Call {
                        name: resolved_name,
                        arguments,
                        argument_names,
                        argument_unpacks,
                        span: combine_spans(parsed_name.span, right_span),
                    });
                }
                Ok(Expr::Constant(
                    self.resolve_constant_name(&parsed_name),
                    parsed_name.span,
                ))
            }
            TokenKind::LeftParen => {
                if self.peek_is_unset_cast_name()
                    && matches!(
                        self.tokens.get(self.index + 1).map(|token| &token.kind),
                        Some(TokenKind::RightParen)
                    )
                {
                    return Err(Diagnostic::new(
                        "The (unset) cast is no longer supported",
                        Some(token.span),
                    ));
                }
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

    fn parse_match_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        self.expect_left_paren()?;
        let subject = self.parse_expr()?;
        self.expect_right_paren()?;
        self.expect_left_brace()?;

        let mut arms = Vec::new();
        let mut seen_default = false;
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            let arm_start = self.peek().span;
            let (conditions, is_default) = if matches!(self.peek().kind, TokenKind::Default) {
                if seen_default {
                    return Err(Diagnostic::new(
                        "Match expressions may only contain one default arm",
                        Some(self.peek().span),
                    ));
                }
                seen_default = true;
                self.advance();
                if matches!(self.peek().kind, TokenKind::Comma)
                    && matches!(self.peek_next().kind, TokenKind::DoubleArrow)
                {
                    self.advance();
                }
                (Vec::new(), true)
            } else {
                (self.parse_match_arm_conditions()?, false)
            };
            self.expect_double_arrow()?;
            let value = self.parse_expr()?;
            let arm_span = combine_spans(arm_start, value.span());
            arms.push(MatchArm {
                conditions,
                value,
                is_default,
                span: arm_span,
            });
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                continue;
            }
            if !matches!(self.peek().kind, TokenKind::RightBrace) {
                return Err(syntax_error_unexpected(self.peek(), Some(",")));
            }
        }

        let right_span = self.expect_right_brace()?;
        Ok(Expr::Match {
            subject: Box::new(subject),
            arms,
            span: combine_spans(start_span, right_span),
        })
    }

    fn parse_match_arm_conditions(&mut self) -> Result<Vec<Expr>> {
        let mut conditions = vec![self.parse_expr()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            if matches!(self.peek().kind, TokenKind::DoubleArrow) {
                break;
            }
            conditions.push(self.parse_expr()?);
        }
        Ok(conditions)
    }

    fn parse_yield_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        if matches!(
            self.peek().kind,
            TokenKind::Semicolon
                | TokenKind::CloseTag
                | TokenKind::RightParen
                | TokenKind::RightBracket
                | TokenKind::RightBrace
                | TokenKind::Comma
                | TokenKind::Colon
        ) {
            return Ok(Expr::Yield {
                key: None,
                value: None,
                span: start_span,
            });
        }
        if self.peek_is_identifier("from") {
            return Err(Diagnostic::new(
                "yield from is unsupported",
                Some(self.peek().span),
            ));
        }

        let first = self.parse_expr()?;
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let value = self.parse_expr()?;
            let span = combine_spans(start_span, value.span());
            return Ok(Expr::Yield {
                key: Some(Box::new(first)),
                value: Some(Box::new(value)),
                span,
            });
        }
        let span = combine_spans(start_span, first.span());
        Ok(Expr::Yield {
            key: None,
            value: Some(Box::new(first)),
            span,
        })
    }

    fn parse_arrow_function_expr(
        &mut self,
        start_span: SourceSpan,
        is_static: bool,
        attributes: ParsedAttributes,
    ) -> Result<Expr> {
        let return_by_ref = if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        };
        self.allow_unscoped_relative_types += 1;
        let parameters = self.parse_function_parameters()?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
        } else {
            None
        };
        self.allow_unscoped_relative_types -= 1;
        self.expect_double_arrow()?;
        let previous_allow_append_array_read = self.allow_append_array_read;
        if return_by_ref {
            self.allow_append_array_read = true;
        }
        self.return_by_ref_stack.push(return_by_ref);
        self.function_depth += 1;
        let expression = self.parse_expr();
        self.function_depth -= 1;
        self.return_by_ref_stack.pop();
        self.allow_append_array_read = previous_allow_append_array_read;
        let expression = expression?;
        let captures = arrow_function_captures(&parameters, &expression, is_static);
        let expression_span = expression.span();
        let span = combine_spans(start_span, expression_span);
        Ok(Expr::AnonymousFunction(AnonymousFunction {
            attributes,
            parameters,
            captures,
            return_type,
            return_by_ref,
            is_static,
            is_arrow: true,
            body: vec![Statement::Return {
                value: Some(expression),
                span: expression_span,
            }],
            span,
        }))
    }

    fn parse_dynamic_variable_expr(&mut self, dollar_span: SourceSpan) -> Result<Expr> {
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            self.advance();
            let name = self.parse_expr()?;
            let right_span = self.expect_right_brace()?;
            return Ok(Expr::DynamicVariable {
                name: Box::new(name),
                span: combine_spans(dollar_span, right_span),
            });
        }

        if matches!(self.peek().kind, TokenKind::Variable(_) | TokenKind::Dollar) {
            let name = self.parse_primary_expr()?;
            return Ok(Expr::DynamicVariable {
                span: combine_spans(dollar_span, name.span()),
                name: Box::new(name),
            });
        }

        Err(Diagnostic::new(
            "expected variable name or braced expression after `$`",
            Some(self.peek().span),
        ))
    }

    fn parse_static_member_expr(
        &mut self,
        class_name: String,
        class_span: SourceSpan,
    ) -> Result<Expr> {
        let scope_span = self.advance().span;
        let member = self.advance().clone();
        if let TokenKind::Variable(member_name) = member.kind {
            if matches!(self.peek().kind, TokenKind::LeftParen) {
                let name_expr = Expr::Variable(member_name, member.span);
                return self.parse_dynamic_static_method_call(class_name, class_span, name_expr);
            }
            return Ok(Expr::StaticPropertyFetch {
                class_name,
                name: member_name,
                span: combine_spans(class_span, member.span),
            });
        }
        if let TokenKind::LeftBrace = member.kind {
            let name_expr = self.parse_expr()?;
            let right_brace_span = self.expect_right_brace()?;
            let member_span = combine_spans(member.span, right_brace_span);
            if !matches!(self.peek().kind, TokenKind::LeftParen) {
                return Err(Diagnostic::new(
                    CLASS_CONSTANT_FETCH_UNSUPPORTED,
                    Some(scope_span),
                ));
            }
            if self.peek_is_first_class_callable_arguments() {
                return Err(Diagnostic::new(
                    "dynamic first-class static method callables are unsupported",
                    Some(member_span),
                ));
            }
            return self.parse_dynamic_static_method_call(class_name, class_span, name_expr);
        }
        let TokenKind::Identifier(member_name) = member.kind else {
            return Err(Diagnostic::new(
                CLASS_CONSTANT_FETCH_UNSUPPORTED,
                Some(scope_span),
            ));
        };
        if !matches!(self.peek().kind, TokenKind::LeftParen) {
            return Ok(Expr::ClassConstantFetch {
                class_name,
                name: member_name,
                span: combine_spans(class_span, member.span),
            });
        }
        if self.peek_is_first_class_callable_arguments() {
            let right_span = self.parse_first_class_callable_arguments()?;
            return Ok(Expr::FirstClassCallable {
                callable: Box::new(Expr::String(
                    format!("{}::{}", class_name, member_name),
                    combine_spans(class_span, member.span),
                )),
                span: combine_spans(class_span, right_span),
            });
        }
        let (arguments, argument_names, argument_unpacks, right_span) =
            self.parse_call_arguments()?;
        Ok(Expr::Call {
            name: format!("{}::{}", class_name, member_name),
            arguments,
            argument_names,
            argument_unpacks,
            span: combine_spans(class_span, right_span),
        })
    }

    fn parse_dynamic_static_method_call(
        &mut self,
        class_name: String,
        class_span: SourceSpan,
        method_name: Expr,
    ) -> Result<Expr> {
        self.parse_dynamic_static_method_call_expr(
            Expr::String(class_name, class_span),
            class_span,
            method_name,
        )
    }

    fn parse_dynamic_static_method_call_expr(
        &mut self,
        class_name: Expr,
        class_span: SourceSpan,
        method_name: Expr,
    ) -> Result<Expr> {
        let (arguments, argument_names, argument_unpacks, right_span) =
            self.parse_call_arguments()?;
        let callable = Expr::Array {
            elements: vec![
                ArrayElement {
                    key: None,
                    value: ArrayElementValue::Value(class_name),
                    line: class_span.line,
                },
                ArrayElement {
                    key: None,
                    value: ArrayElementValue::Value(method_name),
                    line: class_span.line,
                },
            ],
            span: class_span,
        };
        Ok(Expr::DynamicCall {
            callee: Box::new(callable),
            arguments,
            argument_names,
            argument_unpacks,
            span: combine_spans(class_span, right_span),
        })
    }

    fn parse_new_object_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let _ = self.parse_attribute_groups()?;
        if token_is_identifier_named(self.peek(), "class") {
            return self.parse_anonymous_class_expr(start_span);
        }
        if matches!(
            self.peek().kind,
            TokenKind::Variable(_) | TokenKind::Dollar | TokenKind::LeftParen
        ) {
            let class_name = self.parse_postfix_expr()?;
            let mut span = combine_spans(start_span, class_name.span());
            let (arguments, argument_names, argument_unpacks) =
                if matches!(self.peek().kind, TokenKind::LeftParen) {
                    let (arguments, argument_names, argument_unpacks, right_span) =
                        self.parse_call_arguments()?;
                    span = combine_spans(start_span, right_span);
                    (arguments, argument_names, argument_unpacks)
                } else {
                    (Vec::new(), Vec::new(), Vec::new())
                };
            return Ok(Expr::DynamicNewObject {
                class_name: Box::new(class_name),
                arguments,
                argument_names,
                argument_unpacks,
                span,
            });
        }

        let (class_name, class_span) = self.parse_new_object_class_name()?;
        let mut span = combine_spans(start_span, class_span);
        let (arguments, argument_names, argument_unpacks) =
            if matches!(self.peek().kind, TokenKind::LeftParen) {
                let (arguments, argument_names, argument_unpacks, right_span) =
                    self.parse_call_arguments()?;
                span = combine_spans(start_span, right_span);
                (arguments, argument_names, argument_unpacks)
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            };
        Ok(Expr::NewObject {
            class_name,
            arguments,
            argument_names,
            argument_unpacks,
            anonymous_class_source: None,
            span,
        })
    }

    fn parse_new_object_class_name(&mut self) -> Result<(String, SourceSpan)> {
        self.parse_resolved_class_name("expected class name")
    }

    fn parse_anonymous_class_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let class_span = self.advance().span;
        let (arguments, argument_names, argument_unpacks) =
            if matches!(self.peek().kind, TokenKind::LeftParen) {
                let (arguments, argument_names, argument_unpacks, _) =
                    self.parse_call_arguments()?;
                (arguments, argument_names, argument_unpacks)
            } else {
                (Vec::new(), Vec::new(), Vec::new())
            };

        let parent_name = if token_is_identifier_named(self.peek(), "extends") {
            self.advance();
            Some(
                self.parse_resolved_class_name("expected parent class name")?
                    .0,
            )
        } else {
            None
        };

        let mut interfaces = Vec::new();
        if token_is_identifier_named(self.peek(), "implements") {
            self.advance();
            interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            }
        }

        let class_name = self.next_anonymous_class_name(parent_name.as_deref(), &interfaces);
        self.expect_left_brace()?;
        let mut properties = Vec::new();
        let mut static_properties = Vec::new();
        let mut constants = Vec::new();
        let mut methods = Vec::new();
        let mut trait_uses = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            match self.parse_class_member(false, false, &class_name)? {
                ParsedClassMember::Method(method) => methods.push(method),
                ParsedClassMember::Properties(parsed_properties) => {
                    properties.extend(parsed_properties);
                }
                ParsedClassMember::StaticProperties(parsed_properties) => {
                    static_properties.extend(parsed_properties);
                }
                ParsedClassMember::Constants(parsed_constants) => {
                    constants.extend(parsed_constants);
                }
                ParsedClassMember::TraitUses(parsed_trait_uses) => {
                    trait_uses.extend(parsed_trait_uses);
                }
            }
        }
        let right_span = self.expect_right_brace()?;
        let span = combine_spans(start_span, right_span);
        let source = self
            .source
            .get(start_span.byte_start..right_span.byte_end)
            .unwrap_or_default()
            .to_string();
        self.anonymous_classes.push(ClassDecl {
            name: class_name.clone(),
            parent_name,
            interfaces,
            trait_uses,
            attributes: AttributeMetadata::default(),
            is_abstract: false,
            is_final: false,
            is_interface: false,
            is_readonly: false,
            properties,
            static_properties,
            constants,
            methods,
            span: class_span,
        });

        Ok(Expr::NewObject {
            class_name,
            arguments,
            argument_names,
            argument_unpacks,
            anonymous_class_source: Some(source),
            span,
        })
    }

    fn next_anonymous_class_name(
        &mut self,
        parent_name: Option<&str>,
        interfaces: &[String],
    ) -> String {
        let base = parent_name
            .or_else(|| interfaces.first().map(String::as_str))
            .map(|name| format!("{name}@anonymous"))
            .unwrap_or_else(|| "class@anonymous".to_string());
        let count = self
            .anonymous_class_name_counts
            .entry(base.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        if *count == 1 {
            base
        } else {
            format!("{base}#{}", *count)
        }
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

    fn parse_long_list_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        self.expect_left_paren()?;
        let (elements, right_span) = self.parse_list_expr_elements()?;
        Ok(Expr::List(ListExpr {
            elements,
            span: combine_spans(start_span, right_span),
        }))
    }

    fn parse_list_expr_elements(&mut self) -> Result<(Vec<ListExprElement>, SourceSpan)> {
        let mut elements = Vec::new();
        loop {
            if matches!(self.peek().kind, TokenKind::RightParen) {
                break;
            }
            if matches!(self.peek().kind, TokenKind::Comma) {
                let span = self.advance().span;
                elements.push(ListExprElement {
                    key: None,
                    target: None,
                    span,
                });
                continue;
            }

            elements.push(self.parse_list_expr_element()?);
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        let right_span = self.expect_right_paren()?;
        Ok((elements, right_span))
    }

    fn parse_list_expr_element(&mut self) -> Result<ListExprElement> {
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            let span = self.advance().span;
            let target = ListExprElementTarget::Reference(self.parse_reference_target()?);
            if matches!(self.peek().kind, TokenKind::DoubleArrow) {
                return Err(Diagnostic::new(
                    "Key element cannot be a reference",
                    Some(self.peek().span),
                ));
            }
            return Ok(ListExprElement {
                key: None,
                target: Some(target),
                span,
            });
        }

        let first = self.parse_expr()?;
        let first_span = first.span();
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let target = self.parse_list_expr_element_value()?;
            let span = combine_spans(first_span, list_expr_element_target_span(&target));
            Ok(ListExprElement {
                key: Some(first),
                target: Some(target),
                span,
            })
        } else {
            Ok(ListExprElement {
                key: None,
                target: Some(ListExprElementTarget::Value(first)),
                span: first_span,
            })
        }
    }

    fn parse_list_expr_element_value(&mut self) -> Result<ListExprElementTarget> {
        if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            return Ok(ListExprElementTarget::Reference(
                self.parse_reference_target()?,
            ));
        }
        Ok(ListExprElementTarget::Value(self.parse_expr()?))
    }

    fn parse_array_elements(
        &mut self,
        terminator: TokenKind,
    ) -> Result<(Vec<ArrayElement>, SourceSpan)> {
        let mut elements = Vec::new();
        while !self.at_array_terminator(&terminator) {
            if matches!(self.peek().kind, TokenKind::Comma) {
                let span = self.advance().span;
                elements.push(ArrayElement {
                    key: None,
                    value: ArrayElementValue::Hole(span),
                    line: span.line,
                });
                continue;
            }
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
        let line = self.peek().span.line;
        if matches!(self.peek().kind, TokenKind::Ellipsis) {
            self.advance();
            let value = self.parse_expr()?;
            return Ok(ArrayElement {
                key: None,
                value: ArrayElementValue::Unpack(value),
                line,
            });
        }

        if matches!(self.peek().kind, TokenKind::Ampersand) {
            self.advance();
            let value = ArrayElementValue::Reference(self.parse_reference_target()?);
            if matches!(self.peek().kind, TokenKind::DoubleArrow) {
                return Err(Diagnostic::new(
                    "Key element cannot be a reference",
                    Some(self.peek().span),
                ));
            }
            return Ok(ArrayElement {
                key: None,
                value,
                line,
            });
        }

        let first = self.parse_expr()?;
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let value = self.parse_array_element_value()?;
            Ok(ArrayElement {
                key: Some(first),
                value,
                line,
            })
        } else {
            Ok(ArrayElement {
                key: None,
                value: ArrayElementValue::Value(first),
                line,
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
        let (targets, argument_names, argument_unpacks, right_span) =
            self.parse_call_arguments()?;
        reject_named_language_construct_arguments(&argument_names, start_span)?;
        reject_unpacked_language_construct_arguments(&argument_unpacks, start_span)?;
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
        let (mut arguments, argument_names, argument_unpacks, right_span) =
            self.parse_call_arguments()?;
        reject_named_language_construct_arguments(&argument_names, start_span)?;
        reject_unpacked_language_construct_arguments(&argument_unpacks, start_span)?;
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

    fn parse_call_arguments(
        &mut self,
    ) -> Result<(Vec<Expr>, Vec<Option<String>>, Vec<bool>, SourceSpan)> {
        self.expect_left_paren()?;
        let mut arguments = Vec::new();
        let mut argument_names = Vec::new();
        let mut argument_unpacks = Vec::new();
        let mut named_arguments = HashSet::new();
        let mut seen_named_argument = false;
        let mut seen_unpacked_argument = false;
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            let (name, unpack, argument, span) = self.parse_call_argument()?;
            if let Some(name) = &name {
                seen_named_argument = true;
                if !named_arguments.insert(name.clone()) {
                    return Err(Diagnostic::new(
                        format!("Named parameter ${name} overwrites previous argument"),
                        Some(span),
                    ));
                }
            }
            if unpack {
                seen_unpacked_argument = true;
            }
            arguments.push(argument);
            argument_names.push(name);
            argument_unpacks.push(unpack);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                if matches!(self.peek().kind, TokenKind::RightParen) {
                    break;
                }
                let (name, unpack, argument, span) = self.parse_call_argument()?;
                if let Some(name) = &name {
                    seen_named_argument = true;
                    if !named_arguments.insert(name.clone()) {
                        return Err(Diagnostic::new(
                            format!("Named parameter ${name} overwrites previous argument"),
                            Some(span),
                        ));
                    }
                } else if seen_named_argument {
                    return Err(Diagnostic::new(
                        "Cannot use positional argument after named argument",
                        Some(span),
                    ));
                } else if seen_unpacked_argument && !unpack {
                    return Err(Diagnostic::new(
                        "Cannot use positional argument after argument unpacking",
                        Some(span),
                    ));
                }
                if unpack {
                    if seen_named_argument {
                        return Err(Diagnostic::new(
                            "Cannot use argument unpacking after named arguments",
                            Some(span),
                        ));
                    }
                    seen_unpacked_argument = true;
                }
                arguments.push(argument);
                argument_names.push(name);
                argument_unpacks.push(unpack);
            }
        }
        let right_span = self.expect_right_paren()?;
        Ok((arguments, argument_names, argument_unpacks, right_span))
    }

    fn peek_is_first_class_callable_arguments(&self) -> bool {
        matches!(self.peek().kind, TokenKind::LeftParen)
            && matches!(self.peek_next().kind, TokenKind::Ellipsis)
            && matches!(self.peek_n(2).kind, TokenKind::RightParen)
    }

    fn parse_first_class_callable_arguments(&mut self) -> Result<SourceSpan> {
        self.expect_left_paren()?;
        let token = self.advance().clone();
        if !matches!(token.kind, TokenKind::Ellipsis) {
            return Err(Diagnostic::new("expected ...", Some(token.span)));
        }
        self.expect_right_paren()
    }

    fn parse_call_argument(&mut self) -> Result<(Option<String>, bool, Expr, SourceSpan)> {
        if matches!(self.peek().kind, TokenKind::Ellipsis) {
            let spread_span = self.advance().span;
            let value = self.parse_call_argument_expr()?;
            return Ok((None, true, value, spread_span));
        }
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::Colon)
            ) {
                let name = name.clone();
                let name_span = self.advance().span;
                self.expect_colon()?;
                let value = self.parse_call_argument_expr()?;
                return Ok((Some(name), false, value, name_span));
            }
        }

        let value = self.parse_call_argument_expr()?;
        let span = value.span();
        Ok((None, false, value, span))
    }

    fn parse_call_argument_expr(&mut self) -> Result<Expr> {
        let previous = self.allow_append_array_read;
        self.allow_append_array_read = true;
        let value = self.parse_expr();
        self.allow_append_array_read = previous;
        value
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

    fn peek_next_is_void_cast_name(&self) -> bool {
        matches!(
            &self.peek_next().kind,
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
            TokenKind::Identifier(ref name) if name.eq_ignore_ascii_case("array") => {
                Some(CastKind::Array)
            }
            TokenKind::Identifier(ref name) if name.eq_ignore_ascii_case("object") => {
                Some(CastKind::Object)
            }
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

    fn peek_keyword_boolean_op(&self) -> Option<(BinaryOp, u8)> {
        match self.peek().kind {
            TokenKind::KeywordOr => Some((BinaryOp::Or, KEYWORD_OR_PRECEDENCE)),
            TokenKind::KeywordXor => Some((BinaryOp::Xor, KEYWORD_XOR_PRECEDENCE)),
            TokenKind::KeywordAnd => Some((BinaryOp::And, KEYWORD_AND_PRECEDENCE)),
            _ => None,
        }
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
            let message = nested_ternary_message(first_is_short, self.peek_next_is_colon());
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
                | TokenKind::BacktickString(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Variable(_)
                | TokenKind::Dollar
                | TokenKind::Function
                | TokenKind::New
                | TokenKind::Clone
                | TokenKind::Yield
                | TokenKind::Identifier(_)
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Tilde
                | TokenKind::At
                | TokenKind::Print
                | TokenKind::Include
                | TokenKind::IncludeOnce
                | TokenKind::Require
                | TokenKind::RequireOnce
                | TokenKind::Throw
                | TokenKind::Match
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

    fn peek_starts_set_visibility_modifier(&self) -> bool {
        matches!(self.peek_next().kind, TokenKind::LeftParen)
            && matches!(
                &self.peek_n(2).kind,
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("set")
            )
            && matches!(self.peek_n(3).kind, TokenKind::RightParen)
    }

    fn peek_starts_class_decl(&self) -> bool {
        token_is_identifier_named(self.peek(), "class")
            || token_is_identifier_named(self.peek(), "interface")
            || (token_is_identifier_named(self.peek(), "readonly")
                && token_is_identifier_named(self.peek_next(), "class"))
            || (token_is_identifier_named(self.peek(), "abstract")
                && (token_is_identifier_named(self.peek_next(), "class")
                    || token_is_identifier_named(self.peek_next(), "final")
                    || token_is_identifier_named(self.peek_next(), "readonly")))
            || (token_is_identifier_named(self.peek(), "final")
                && (token_is_identifier_named(self.peek_next(), "class")
                    || token_is_identifier_named(self.peek_next(), "abstract")
                    || token_is_identifier_named(self.peek_next(), "final")))
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
        self.peek_is_assignment_op()
    }

    fn expect_equal(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::Equal) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected equal", Some(token.span)))
        }
    }

    fn expect_double_arrow(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(token.kind, TokenKind::DoubleArrow) {
            Ok(token.span)
        } else {
            Err(syntax_error_unexpected(token, Some("=>")))
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

    fn peek_n(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.index + offset)
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

fn validate_closure_use_name(name: &str, span: SourceSpan) -> Result<()> {
    if name == "this" {
        return Err(Diagnostic::new(
            "Cannot use $this as lexical variable",
            Some(span),
        ));
    }
    if is_auto_global_name(name) {
        return Err(Diagnostic::new(
            "Cannot use auto-global as lexical variable",
            Some(span),
        ));
    }
    Ok(())
}

fn validate_closure_use_parameter_names(
    parameters: &[FunctionParameter],
    captures: &[ClosureUseCapture],
) -> Result<()> {
    for capture in captures {
        if parameters
            .iter()
            .any(|parameter| parameter.name == capture.name)
        {
            return Err(Diagnostic::new(
                format!(
                    "Cannot use lexical variable ${} as a parameter name",
                    capture.name
                ),
                Some(capture.span),
            ));
        }
    }
    Ok(())
}

fn is_auto_global_name(name: &str) -> bool {
    matches!(
        name,
        "GLOBALS"
            | "_SERVER"
            | "_GET"
            | "_POST"
            | "_FILES"
            | "_COOKIE"
            | "_SESSION"
            | "_REQUEST"
            | "_ENV"
    )
}

fn is_unsupported_builtin_type_hint_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "callable" | "false" | "never" | "static" | "true" | "void"
    )
}

fn is_unqualified_only_builtin_type_hint_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array"
            | "callable"
            | "bool"
            | "boolean"
            | "float"
            | "double"
            | "int"
            | "integer"
            | "iterable"
            | "mixed"
            | "null"
            | "object"
            | "string"
            | "binary"
            | "void"
            | "never"
    )
}

fn property_type_token_text(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::IntType => "int",
        TokenKind::IntegerType => "integer",
        TokenKind::FloatType => "float",
        TokenKind::DoubleType => "double",
        TokenKind::StringType => "string",
        TokenKind::BinaryType => "binary",
        TokenKind::BoolType => "bool",
        TokenKind::BooleanType => "boolean",
        _ => "mixed",
    }
}

fn property_type_hint_from_type_hint(type_hint: &TypeHint) -> PropertyTypeHint {
    match type_hint {
        TypeHint::Null => PropertyTypeHint {
            text: "null".to_string(),
            kind: PropertyTypeKind::Null,
            allows_null: false,
        },
        TypeHint::Array => PropertyTypeHint {
            text: "array".to_string(),
            kind: PropertyTypeKind::Array,
            allows_null: false,
        },
        TypeHint::Int => PropertyTypeHint {
            text: "int".to_string(),
            kind: PropertyTypeKind::Int,
            allows_null: false,
        },
        TypeHint::Float => PropertyTypeHint {
            text: "float".to_string(),
            kind: PropertyTypeKind::Float,
            allows_null: false,
        },
        TypeHint::String => PropertyTypeHint {
            text: "string".to_string(),
            kind: PropertyTypeKind::String,
            allows_null: false,
        },
        TypeHint::Bool => PropertyTypeHint {
            text: "bool".to_string(),
            kind: PropertyTypeKind::Bool,
            allows_null: false,
        },
        TypeHint::True | TypeHint::False | TypeHint::Static => PropertyTypeHint {
            text: type_hint_display(type_hint),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
        },
        TypeHint::Mixed => PropertyTypeHint {
            text: "mixed".to_string(),
            kind: PropertyTypeKind::Mixed,
            allows_null: false,
        },
        TypeHint::Object => PropertyTypeHint {
            text: "object".to_string(),
            kind: PropertyTypeKind::Object,
            allows_null: false,
        },
        TypeHint::Callable
        | TypeHint::Iterable
        | TypeHint::Union(_)
        | TypeHint::Intersection(_) => PropertyTypeHint {
            text: type_hint_key(type_hint),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
        },
        TypeHint::Void | TypeHint::Never => PropertyTypeHint {
            text: "mixed".to_string(),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
        },
        TypeHint::Nullable(inner) => {
            let mut nested = property_type_hint_from_type_hint(inner);
            nested.text = format!("?{}", nested.text);
            nested.allows_null = true;
            nested
        }
        TypeHint::Class(name) => PropertyTypeHint {
            text: name.clone(),
            kind: PropertyTypeKind::Class(name.clone()),
            allows_null: false,
        },
    }
}

fn nullable_type_hint(type_hint: TypeHint, span: SourceSpan) -> Result<TypeHint> {
    match type_hint {
        TypeHint::Null => Err(Diagnostic::new(
            "null cannot be marked as nullable",
            Some(span),
        )),
        TypeHint::Mixed => Err(Diagnostic::new(
            "Type mixed cannot be marked as nullable since mixed already includes null",
            Some(span),
        )),
        TypeHint::Void => Err(Diagnostic::new(
            "void cannot be marked as nullable",
            Some(span),
        )),
        TypeHint::Never => Err(Diagnostic::new(
            "never cannot be marked as nullable",
            Some(span),
        )),
        TypeHint::Nullable(_) | TypeHint::Union(_) | TypeHint::Intersection(_) => {
            Err(Diagnostic::new("invalid nullable type hint", Some(span)))
        }
        other => Ok(TypeHint::Nullable(Box::new(other))),
    }
}

fn validate_union_type_hint(types: &[TypeHint], span: SourceSpan) -> Result<()> {
    for type_hint in types {
        match type_hint {
            TypeHint::Mixed => {
                return Err(Diagnostic::new(
                    "Type mixed can only be used as a standalone type",
                    Some(span),
                ));
            }
            TypeHint::Void => {
                return Err(Diagnostic::new(
                    "Void can only be used as a standalone type",
                    Some(span),
                ));
            }
            TypeHint::Never => {
                return Err(Diagnostic::new(
                    "never can only be used as a standalone type",
                    Some(span),
                ));
            }
            TypeHint::Nullable(_) | TypeHint::Union(_) => {
                return Err(Diagnostic::new("invalid union type hint", Some(span)));
            }
            _ => {}
        }
    }

    let iterable_count = types
        .iter()
        .filter(|type_hint| matches!(type_hint, TypeHint::Iterable))
        .count();
    if iterable_count > 1 || (iterable_count == 1 && types.iter().any(type_hint_is_array)) {
        return Err(Diagnostic::new(
            "Duplicate type array is redundant",
            Some(span),
        ));
    }
    if iterable_count == 1
        && types
            .iter()
            .any(|type_hint| type_hint_is_class_named(type_hint, "Traversable"))
    {
        return Err(Diagnostic::new(
            "Duplicate type Traversable is redundant",
            Some(span),
        ));
    }
    if types
        .iter()
        .any(|type_hint| matches!(type_hint, TypeHint::True))
        && types
            .iter()
            .any(|type_hint| matches!(type_hint, TypeHint::False))
    {
        return Err(Diagnostic::new(
            "Type contains both true and false, bool must be used instead",
            Some(span),
        ));
    }
    if types
        .iter()
        .any(|type_hint| matches!(type_hint, TypeHint::Bool))
    {
        if types
            .iter()
            .any(|type_hint| matches!(type_hint, TypeHint::False))
        {
            return Err(Diagnostic::new(
                "Duplicate type false is redundant",
                Some(span),
            ));
        }
        if types
            .iter()
            .any(|type_hint| matches!(type_hint, TypeHint::True))
        {
            return Err(Diagnostic::new(
                "Duplicate type true is redundant",
                Some(span),
            ));
        }
    }
    if types
        .iter()
        .any(|type_hint| matches!(type_hint, TypeHint::Object))
        && types.iter().any(type_hint_is_explicit_class_like)
    {
        return Err(Diagnostic::new(
            format!(
                "Type {} contains both object and a class type, which is redundant",
                union_type_redundancy_display(types)
            ),
            Some(span),
        ));
    }
    let mut seen = Vec::new();
    for type_hint in types {
        let key = type_hint_key(type_hint);
        if seen.iter().any(|seen_key| seen_key == &key) {
            if matches!(type_hint, TypeHint::Intersection(_)) {
                let display = type_hint_display(type_hint);
                return Err(Diagnostic::new(
                    format!("Type {display} is redundant with type {display}"),
                    Some(span),
                ));
            } else {
                return Err(Diagnostic::new(
                    format!(
                        "Duplicate type {} is redundant",
                        type_hint_duplicate_display(type_hint)
                    ),
                    Some(span),
                ));
            }
        }
        seen.push(key);
    }
    Ok(())
}

fn validate_intersection_type_hint(types: &[TypeHint], span: SourceSpan) -> Result<()> {
    let mut seen = Vec::new();
    for type_hint in types {
        match type_hint {
            TypeHint::Class(_) => {}
            TypeHint::Iterable => {
                return Err(Diagnostic::new(
                    "Type Traversable|array cannot be part of an intersection type",
                    Some(span),
                ));
            }
            other => {
                return Err(Diagnostic::new(
                    format!(
                        "Type {} cannot be part of an intersection type",
                        type_hint_display(other)
                    ),
                    Some(span),
                ));
            }
        }
        let key = type_hint_key(type_hint);
        if seen.iter().any(|seen_key| seen_key == &key) {
            return Err(Diagnostic::new(
                format!(
                    "Duplicate type {} is redundant",
                    type_hint_duplicate_display(type_hint)
                ),
                Some(span),
            ));
        }
        seen.push(key);
    }
    Ok(())
}

fn type_hint_is_array(type_hint: &TypeHint) -> bool {
    matches!(type_hint, TypeHint::Array)
}

fn type_hint_is_class_named(type_hint: &TypeHint, name: &str) -> bool {
    matches!(type_hint, TypeHint::Class(class_name) if class_name.eq_ignore_ascii_case(name))
}

fn type_hint_is_explicit_class_like(type_hint: &TypeHint) -> bool {
    matches!(
        type_hint,
        TypeHint::Class(_) | TypeHint::Static | TypeHint::Intersection(_)
    )
}

fn type_hint_duplicate_display(type_hint: &TypeHint) -> String {
    match type_hint {
        TypeHint::Class(name) => name.clone(),
        TypeHint::Static => "static".to_string(),
        TypeHint::Intersection(_) => type_hint_display(type_hint),
        other => type_hint_key(other),
    }
}

fn union_type_redundancy_display(types: &[TypeHint]) -> String {
    let mut class_like = Vec::new();
    let mut object = Vec::new();
    let mut array_like = Vec::new();
    let mut other = Vec::new();

    for type_hint in types {
        match type_hint {
            TypeHint::Class(name) => class_like.push(name.clone()),
            TypeHint::Static => class_like.push("static".to_string()),
            TypeHint::Intersection(_) => {
                class_like.push(format!("({})", type_hint_display(type_hint)));
            }
            TypeHint::Iterable => {
                class_like.push("Traversable".to_string());
                array_like.push("array".to_string());
            }
            TypeHint::Object => object.push("object".to_string()),
            TypeHint::Array => array_like.push("array".to_string()),
            TypeHint::Null => other.push("null".to_string()),
            other_type => other.push(type_hint_duplicate_display(other_type)),
        }
    }

    class_like
        .into_iter()
        .chain(object)
        .chain(array_like)
        .chain(other)
        .collect::<Vec<_>>()
        .join("|")
}

fn type_hint_key(type_hint: &TypeHint) -> String {
    match type_hint {
        TypeHint::Null => "null".to_string(),
        TypeHint::Array => "array".to_string(),
        TypeHint::Int => "int".to_string(),
        TypeHint::Float => "float".to_string(),
        TypeHint::String => "string".to_string(),
        TypeHint::Bool => "bool".to_string(),
        TypeHint::True => "true".to_string(),
        TypeHint::False => "false".to_string(),
        TypeHint::Callable => "callable".to_string(),
        TypeHint::Object => "object".to_string(),
        TypeHint::Iterable => "iterable".to_string(),
        TypeHint::Mixed => "mixed".to_string(),
        TypeHint::Void => "void".to_string(),
        TypeHint::Never => "never".to_string(),
        TypeHint::Static => "static".to_string(),
        TypeHint::Nullable(inner) => format!("?{}", type_hint_key(inner)),
        TypeHint::Union(types) => types
            .iter()
            .map(type_hint_key)
            .collect::<Vec<_>>()
            .join("|"),
        TypeHint::Intersection(types) => types
            .iter()
            .map(type_hint_key)
            .collect::<Vec<_>>()
            .join("&"),
        TypeHint::Class(name) => name.to_ascii_lowercase(),
    }
}

fn nested_ternary_message(first_is_short: bool, second_is_short: bool) -> &'static str {
    match (first_is_short, second_is_short) {
        (true, _) => "Unparenthesized `a ?: b ? c : d` is not supported. Use either `(a ?: b) ? c : d` or `a ?: (b ? c : d)`",
        (false, true) => "Unparenthesized `a ? b : c ?: d` is not supported. Use either `(a ? b : c) ?: d` or `a ? b : (c ?: d)`",
        (false, false) => "Unparenthesized `a ? b : c ? d : e` is not supported. Use either `(a ? b : c) ? d : e` or `a ? b : (c ? d : e)`",
    }
}

fn describe_unexpected_token(token: &Token) -> String {
    match &token.kind {
        TokenKind::Identifier(name) => format!("identifier \"{name}\""),
        TokenKind::Variable(name) => format!("variable \"${name}\""),
        TokenKind::String(value) => format!("string \"{}\"", escape_token_text(value)),
        TokenKind::BacktickString(value) => {
            format!("execution string \"{}\"", escape_token_text(value))
        }
        TokenKind::InterpolatedString(_) => "encapsed string".to_string(),
        TokenKind::Int(value) => format!("integer \"{value}\""),
        TokenKind::Float(value) => format!("floating-point number \"{value}\""),
        TokenKind::Eof => "end of file".to_string(),
        _ => format!("token \"{}\"", token_text(&token.kind)),
    }
}

fn object_member_name_from_token(kind: &TokenKind) -> Option<String> {
    let name = match kind {
        TokenKind::Echo
        | TokenKind::Print
        | TokenKind::If
        | TokenKind::Elseif
        | TokenKind::Else
        | TokenKind::Do
        | TokenKind::While
        | TokenKind::For
        | TokenKind::Foreach
        | TokenKind::As
        | TokenKind::Switch
        | TokenKind::Match
        | TokenKind::Case
        | TokenKind::Default
        | TokenKind::Break
        | TokenKind::Continue
        | TokenKind::Return
        | TokenKind::Include
        | TokenKind::IncludeOnce
        | TokenKind::Require
        | TokenKind::RequireOnce
        | TokenKind::Try
        | TokenKind::Catch
        | TokenKind::Throw
        | TokenKind::Yield
        | TokenKind::Goto
        | TokenKind::Const
        | TokenKind::Function
        | TokenKind::Global
        | TokenKind::New
        | TokenKind::Clone
        | TokenKind::True
        | TokenKind::False
        | TokenKind::Null
        | TokenKind::KeywordAnd
        | TokenKind::KeywordOr
        | TokenKind::KeywordXor
        | TokenKind::IntType
        | TokenKind::IntegerType
        | TokenKind::FloatType
        | TokenKind::DoubleType
        | TokenKind::StringType
        | TokenKind::BinaryType
        | TokenKind::BoolType
        | TokenKind::BooleanType => token_text(kind),
        _ => return None,
    };
    Some(name.to_string())
}

fn method_name_from_token(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Identifier(name) => Some(name.clone()),
        _ => object_member_name_from_token(kind),
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
        TokenKind::Match => "match",
        TokenKind::Case => "case",
        TokenKind::Default => "default",
        TokenKind::Break => "break",
        TokenKind::Continue => "continue",
        TokenKind::Return => "return",
        TokenKind::Include => "include",
        TokenKind::IncludeOnce => "include_once",
        TokenKind::Require => "require",
        TokenKind::RequireOnce => "require_once",
        TokenKind::Try => "try",
        TokenKind::Catch => "catch",
        TokenKind::Throw => "throw",
        TokenKind::Goto => "goto",
        TokenKind::Const => "const",
        TokenKind::Function => "function",
        TokenKind::Yield => "yield",
        TokenKind::Global => "global",
        TokenKind::New => "new",
        TokenKind::Clone => "clone",
        TokenKind::Identifier(_) => "identifier",
        TokenKind::String(_) => "string",
        TokenKind::BacktickString(_) => "execution string",
        TokenKind::InterpolatedString(_) => "encapsed string",
        TokenKind::Int(_) => "integer",
        TokenKind::Float(_) => "float",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::Null => "null",
        TokenKind::Variable(_) => "variable",
        TokenKind::Dollar => "$",
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
        TokenKind::NullsafeObjectOperator => "?->",
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
        TokenKind::PipeGreater => "|>",
        TokenKind::Caret => "^",
        TokenKind::Tilde => "~",
        TokenKind::Bang => "!",
        TokenKind::At => "@",
        TokenKind::AttributeStart => "#[",
        TokenKind::Backslash => "\\",
        TokenKind::Ellipsis => "...",
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

fn arrow_function_captures(
    parameters: &[FunctionParameter],
    expression: &Expr,
    is_static: bool,
) -> Vec<ClosureUseCapture> {
    let mut exclusions = HashSet::new();
    for parameter in parameters {
        exclusions.insert(parameter.name.clone());
    }
    if is_static {
        exclusions.insert("this".to_string());
    }

    let mut seen = HashSet::new();
    let mut captures = Vec::new();
    collect_arrow_captures_from_expr(expression, &exclusions, &mut seen, &mut captures);
    captures
}

fn collect_arrow_captures_from_expr(
    expr: &Expr,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    match expr {
        Expr::Variable(name, span) => {
            add_arrow_capture(name, *span, exclusions, seen, captures);
        }
        Expr::DynamicVariable { name, .. } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
        Expr::AnonymousFunction(function) => {
            for capture in &function.captures {
                add_arrow_capture(&capture.name, capture.span, exclusions, seen, captures);
            }
        }
        Expr::IncDec { target, .. } => {
            collect_arrow_captures_from_inc_dec_target(target, exclusions, seen, captures);
        }
        Expr::Assign {
            target, op, value, ..
        } => {
            collect_arrow_captures_from_assignment_target(
                target,
                !matches!(op, AssignmentOp::Assign),
                exclusions,
                seen,
                captures,
            );
            collect_arrow_captures_from_expr(value, exclusions, seen, captures);
        }
        Expr::AssignRef { target, source, .. } => {
            collect_arrow_captures_from_assignment_target(
                target, false, exclusions, seen, captures,
            );
            collect_arrow_captures_from_expr(source, exclusions, seen, captures);
        }
        Expr::Call { arguments, .. } | Expr::NewObject { arguments, .. } => {
            for argument in arguments {
                collect_arrow_captures_from_expr(argument, exclusions, seen, captures);
            }
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            collect_arrow_captures_from_expr(class_name, exclusions, seen, captures);
            for argument in arguments {
                collect_arrow_captures_from_expr(argument, exclusions, seen, captures);
            }
        }
        Expr::FirstClassCallable { callable, .. } => {
            collect_arrow_captures_from_expr(callable, exclusions, seen, captures);
        }
        Expr::DynamicCall {
            callee, arguments, ..
        } => {
            collect_arrow_captures_from_expr(callee, exclusions, seen, captures);
            for argument in arguments {
                collect_arrow_captures_from_expr(argument, exclusions, seen, captures);
            }
        }
        Expr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            for argument in arguments {
                collect_arrow_captures_from_expr(argument, exclusions, seen, captures);
            }
        }
        Expr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
            for argument in arguments {
                collect_arrow_captures_from_expr(argument, exclusions, seen, captures);
            }
        }
        Expr::PropertyFetch { receiver, .. } | Expr::NullsafePropertyFetch { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
        Expr::Clone { expr, .. } => {
            collect_arrow_captures_from_expr(expr, exclusions, seen, captures);
        }
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    collect_arrow_captures_from_expr(key, exclusions, seen, captures);
                }
                match &element.value {
                    ArrayElementValue::Hole(_) => {}
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        collect_arrow_captures_from_expr(value, exclusions, seen, captures);
                    }
                    ArrayElementValue::Reference(target) => {
                        collect_arrow_captures_from_reference_target(
                            target, exclusions, seen, captures,
                        );
                    }
                }
            }
        }
        Expr::List(list) => {
            for element in &list.elements {
                if let Some(key) = &element.key {
                    collect_arrow_captures_from_expr(key, exclusions, seen, captures);
                }
                match &element.target {
                    Some(ListExprElementTarget::Value(value)) => {
                        collect_arrow_captures_from_expr(value, exclusions, seen, captures);
                    }
                    Some(ListExprElementTarget::Reference(target)) => {
                        collect_arrow_captures_from_reference_target(
                            target, exclusions, seen, captures,
                        );
                    }
                    None => {}
                }
            }
        }
        Expr::ArrayAccess { array, index, .. } => {
            collect_arrow_captures_from_expr(array, exclusions, seen, captures);
            if let Some(index) = index {
                collect_arrow_captures_from_expr(index, exclusions, seen, captures);
            }
        }
        Expr::Isset { targets, .. } => {
            for target in targets {
                collect_arrow_captures_from_expr(target, exclusions, seen, captures);
            }
        }
        Expr::Empty { target, .. }
        | Expr::Print {
            expression: target, ..
        }
        | Expr::Include { path: target, .. }
        | Expr::Throw { value: target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::Cast { expr: target, .. }
        | Expr::Grouped { expr: target, .. }
        | Expr::PipeValue { expr: target, .. } => {
            collect_arrow_captures_from_expr(target, exclusions, seen, captures);
        }
        Expr::Yield { key, value, .. } => {
            if let Some(key) = key {
                collect_arrow_captures_from_expr(key, exclusions, seen, captures);
            }
            if let Some(value) = value {
                collect_arrow_captures_from_expr(value, exclusions, seen, captures);
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_arrow_captures_from_expr(left, exclusions, seen, captures);
            collect_arrow_captures_from_expr(right, exclusions, seen, captures);
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            collect_arrow_captures_from_expr(condition, exclusions, seen, captures);
            if let Some(if_true) = if_true {
                collect_arrow_captures_from_expr(if_true, exclusions, seen, captures);
            }
            collect_arrow_captures_from_expr(if_false, exclusions, seen, captures);
        }
        Expr::Match { subject, arms, .. } => {
            collect_arrow_captures_from_expr(subject, exclusions, seen, captures);
            for arm in arms {
                for condition in &arm.conditions {
                    collect_arrow_captures_from_expr(condition, exclusions, seen, captures);
                }
                collect_arrow_captures_from_expr(&arm.value, exclusions, seen, captures);
            }
        }
        Expr::InterpolatedString(parts, _) => {
            for part in parts {
                collect_arrow_captures_from_string_part(part, exclusions, seen, captures);
            }
        }
        Expr::DynamicClassNameFetch { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
        Expr::InstanceOf { expr, target, .. } => {
            collect_arrow_captures_from_expr(expr, exclusions, seen, captures);
            if let InstanceOfTarget::Expr(target) = target {
                collect_arrow_captures_from_expr(target, exclusions, seen, captures);
            }
        }
        Expr::String(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. } => {}
    }
}

fn collect_arrow_captures_from_assignment_target(
    target: &AssignmentTarget,
    reads_target: bool,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    match target {
        AssignmentTarget::Variable { name, span } => {
            if reads_target {
                add_arrow_capture(name, *span, exclusions, seen, captures);
            }
        }
        AssignmentTarget::DynamicVariable { name, .. } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
                }
            }
        }
        AssignmentTarget::ArrayDim(target) => {
            if reads_target {
                add_arrow_capture(&target.array, target.span, exclusions, seen, captures);
            }
            collect_arrow_captures_from_array_dim_target(target, exclusions, seen, captures);
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
                }
            }
        }
        AssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
                }
            }
        }
        AssignmentTarget::Property { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
        AssignmentTarget::DynamicProperty { receiver, name, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
        AssignmentTarget::StaticProperty { .. } => {}
        AssignmentTarget::List(target) => {
            for element in &target.elements {
                if let Some(key) = &element.key {
                    collect_arrow_captures_from_expr(key, exclusions, seen, captures);
                }
                match &element.target {
                    ListAssignmentElementTarget::Value(target) => {
                        collect_arrow_captures_from_assignment_target(
                            target, false, exclusions, seen, captures,
                        );
                    }
                    ListAssignmentElementTarget::Reference(target) => {
                        collect_arrow_captures_from_reference_target(
                            target, exclusions, seen, captures,
                        );
                    }
                }
            }
        }
    }
}

fn collect_arrow_captures_from_reference_target(
    target: &ReferenceTarget,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    match target {
        ReferenceTarget::Variable { name, span } => {
            add_arrow_capture(name, *span, exclusions, seen, captures);
        }
        ReferenceTarget::ArrayDim(target) => {
            add_arrow_capture(&target.array, target.span, exclusions, seen, captures);
            collect_arrow_captures_from_array_dim_target(target, exclusions, seen, captures);
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            for dimension in dimensions.iter().flatten() {
                collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
            }
        }
        ReferenceTarget::Property { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
    }
}

fn collect_arrow_captures_from_inc_dec_target(
    target: &IncDecTarget,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    match target {
        IncDecTarget::Variable { name, span } => {
            add_arrow_capture(name, *span, exclusions, seen, captures);
        }
        IncDecTarget::DynamicVariable { name, .. } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
        }
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
                }
            }
        }
        IncDecTarget::ArrayDim(target) => {
            add_arrow_capture(&target.array, target.span, exclusions, seen, captures);
            collect_arrow_captures_from_array_dim_target(target, exclusions, seen, captures);
        }
        IncDecTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
                }
            }
        }
        IncDecTarget::Property { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
        IncDecTarget::StaticProperty { .. } => {}
    }
}

fn collect_arrow_captures_from_array_dim_target(
    target: &ArrayDimTarget,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    for dimension in &target.dimensions {
        if let Some(dimension) = dimension {
            collect_arrow_captures_from_expr(dimension, exclusions, seen, captures);
        }
    }
}

fn collect_arrow_captures_from_string_part(
    part: &StringPart,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    match part {
        StringPart::Literal(_) => {}
        StringPart::Variable(name) | StringPart::LegacyDollarBraceVariable(name) => {
            add_arrow_capture(
                name,
                SourceSpan::new(0, 0, 0, 0),
                exclusions,
                seen,
                captures,
            );
        }
        StringPart::PropertyFetch { variable, .. } => {
            add_arrow_capture(
                variable,
                SourceSpan::new(0, 0, 0, 0),
                exclusions,
                seen,
                captures,
            );
        }
        StringPart::ArrayAccess { array, indices } => {
            add_arrow_capture(
                array,
                SourceSpan::new(0, 0, 0, 0),
                exclusions,
                seen,
                captures,
            );
            for index in indices {
                if let StringInterpolationIndex::Variable(name) = index {
                    add_arrow_capture(
                        name,
                        SourceSpan::new(0, 0, 0, 0),
                        exclusions,
                        seen,
                        captures,
                    );
                }
            }
        }
    }
}

fn add_arrow_capture(
    name: &str,
    span: SourceSpan,
    exclusions: &HashSet<String>,
    seen: &mut HashSet<String>,
    captures: &mut Vec<ClosureUseCapture>,
) {
    if exclusions.contains(name) || is_php_auto_global(name) || !seen.insert(name.to_string()) {
        return;
    }
    captures.push(ClosureUseCapture {
        name: name.to_string(),
        by_ref: false,
        warn_if_missing: false,
        span,
    });
}

fn is_php_auto_global(name: &str) -> bool {
    matches!(
        name,
        "GLOBALS"
            | "_SERVER"
            | "_GET"
            | "_POST"
            | "_FILES"
            | "_COOKIE"
            | "_SESSION"
            | "_REQUEST"
            | "_ENV"
    )
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

#[derive(Default)]
struct ParsedAttributeArguments {
    positional: Vec<String>,
    message: Option<String>,
    since: Option<String>,
}

impl ParsedAttributeArguments {
    fn record(&mut self, name: Option<String>, value: String) {
        match name.as_deref() {
            Some(name) if name.eq_ignore_ascii_case("message") => {
                self.message = Some(value);
            }
            Some(name) if name.eq_ignore_ascii_case("since") => {
                self.since = Some(value);
            }
            Some(_) => {}
            None => self.positional.push(value),
        }
    }

    fn message(&self) -> Option<String> {
        self.message
            .clone()
            .or_else(|| self.positional.first().cloned())
    }
}

fn apply_parsed_attribute(
    attributes: &mut AttributeMetadata,
    name_segments: &[String],
    arguments: &ParsedAttributeArguments,
) {
    if name_segments.len() != 1 {
        return;
    }
    let name = &name_segments[0];
    if name.eq_ignore_ascii_case("Override") {
        attributes.has_override = true;
    } else if name.eq_ignore_ascii_case("Attribute") {
        attributes.has_attribute = true;
    } else if name.eq_ignore_ascii_case("AllowDynamicProperties") {
        attributes.has_allow_dynamic_properties = true;
    } else if name.eq_ignore_ascii_case("Deprecated") {
        attributes.deprecated_message = Some(arguments.message().unwrap_or_default());
        attributes.deprecated_since = arguments.since.clone();
    } else if name.eq_ignore_ascii_case("NoDiscard") {
        attributes.no_discard_message = Some(arguments.message().unwrap_or_default());
    }
}

fn merge_parsed_attributes(attributes: &mut AttributeMetadata, group: AttributeMetadata) {
    attributes.has_override |= group.has_override;
    attributes.has_attribute |= group.has_attribute;
    attributes.has_allow_dynamic_properties |= group.has_allow_dynamic_properties;
    if group.deprecated_message.is_some() {
        attributes.deprecated_message = group.deprecated_message;
        attributes.deprecated_since = group.deprecated_since;
    }
    if group.no_discard_message.is_some() {
        attributes.no_discard_message = group.no_discard_message;
    }
}

fn is_unsupported_class_like_declaration(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "enum")
}

fn is_modeled_builtin_exception_class_name(name: &str) -> bool {
    matches!(
        name.trim_start_matches('\\').to_ascii_lowercase().as_str(),
        "exception"
            | "errorexception"
            | "reflectionexception"
            | "runtimeexception"
            | "invalidargumentexception"
            | "unexpectedvalueexception"
            | "outofboundsexception"
            | "outofrangeexception"
            | "error"
            | "typeerror"
            | "argumentcounterror"
            | "valueerror"
            | "arithmeticerror"
            | "divisionbyzeroerror"
            | "assertionerror"
            | "parseerror"
            | "unhandledmatcherror"
    )
}

fn is_modeled_builtin_interface_name(name: &str) -> bool {
    matches!(
        name.trim_start_matches('\\').to_ascii_lowercase().as_str(),
        "arrayaccess"
            | "iterator"
            | "iteratoraggregate"
            | "outeriterator"
            | "recursiveiterator"
            | "seekableiterator"
            | "splobserver"
            | "splsubject"
            | "traversable"
            | "stringable"
            | "throwable"
            | "datetimeinterface"
            | "countable"
            | "serializable"
    )
}

fn is_modeled_spl_iterator_class_name(name: &str) -> bool {
    matches!(
        name.trim_start_matches('\\').to_ascii_lowercase().as_str(),
        "arrayiterator"
            | "arrayobject"
            | "callbackfilteriterator"
            | "filteriterator"
            | "infiniteiterator"
            | "iteratoriterator"
            | "limititerator"
            | "recursivearrayiterator"
    )
}

fn modeled_builtin_interface_extends(interface_name: &str, target_name: &str) -> bool {
    let interface_name = interface_name.trim_start_matches('\\').to_ascii_lowercase();
    let target_name = target_name.trim_start_matches('\\').to_ascii_lowercase();
    if interface_name == target_name {
        return true;
    }
    matches!(
        (interface_name.as_str(), target_name.as_str()),
        ("iterator", "traversable")
            | ("iteratoraggregate", "traversable")
            | ("outeriterator", "iterator")
            | ("outeriterator", "traversable")
            | ("recursiveiterator", "iterator")
            | ("recursiveiterator", "traversable")
            | ("seekableiterator", "iterator")
            | ("seekableiterator", "traversable")
    )
}

fn visibility_allows_set_visibility(
    read_visibility: PropertyVisibility,
    set_visibility: PropertyVisibility,
) -> bool {
    visibility_rank(set_visibility) >= visibility_rank(read_visibility)
}

fn validate_asymmetric_property_visibility(
    class_name: &str,
    property_name: &str,
    read_visibility: PropertyVisibility,
    set_visibility: PropertyVisibility,
    span: SourceSpan,
) -> Result<()> {
    if visibility_allows_set_visibility(read_visibility, set_visibility) {
        return Ok(());
    }
    Err(Diagnostic::new(
        format!(
            "Visibility of property {class_name}::${property_name} must not be weaker than set visibility"
        ),
        Some(span),
    ))
}

fn promoted_properties_from_constructor(
    method: &MethodDecl,
    class_is_readonly: bool,
) -> Vec<PropertyDecl> {
    method
        .parameters
        .iter()
        .filter_map(|parameter| {
            let promoted = parameter.promoted_property.as_ref()?;
            Some(PropertyDecl {
                name: parameter.name.clone(),
                visibility: promoted.visibility,
                set_visibility: promoted.set_visibility,
                is_readonly: class_is_readonly || promoted.is_readonly,
                type_hint: parameter
                    .type_hint
                    .as_ref()
                    .map(property_type_hint_from_type_hint),
                has_override_attribute: promoted.has_override_attribute,
                value: None,
                span: promoted.span,
            })
        })
        .collect()
}

fn constructor_promoted_property_assignments(parameters: &[FunctionParameter]) -> Vec<Statement> {
    parameters
        .iter()
        .filter(|parameter| parameter.promoted_property.is_some())
        .map(|parameter| {
            let span = parameter.span;
            Statement::Expression {
                expression: Expr::Assign {
                    target: AssignmentTarget::Property {
                        receiver: Box::new(Expr::Variable("this".to_string(), span)),
                        name: parameter.name.clone(),
                        span,
                    },
                    op: AssignmentOp::Assign,
                    value: Box::new(Expr::Variable(parameter.name.clone(), span)),
                    span,
                },
                span,
            }
        })
        .collect()
}

fn visibility_rank(visibility: PropertyVisibility) -> u8 {
    match visibility {
        PropertyVisibility::Public => 0,
        PropertyVisibility::Protected => 1,
        PropertyVisibility::Private => 2,
    }
}

fn property_visibility_name(visibility: PropertyVisibility) -> &'static str {
    match visibility {
        PropertyVisibility::Public => "public",
        PropertyVisibility::Protected => "protected",
        PropertyVisibility::Private => "private",
    }
}

fn default_set_visibility(
    read_visibility: PropertyVisibility,
    is_readonly: bool,
) -> PropertyVisibility {
    if !is_readonly {
        return read_visibility;
    }
    match read_visibility {
        PropertyVisibility::Public | PropertyVisibility::Protected => PropertyVisibility::Protected,
        PropertyVisibility::Private => PropertyVisibility::Private,
    }
}

fn token_is_identifier_named(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case(expected))
}

fn token_starts_type_hint_atom(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::LeftParen
            | TokenKind::Backslash
            | TokenKind::Null
            | TokenKind::IntType
            | TokenKind::IntegerType
            | TokenKind::FloatType
            | TokenKind::DoubleType
            | TokenKind::StringType
            | TokenKind::BinaryType
            | TokenKind::BoolType
            | TokenKind::BooleanType
            | TokenKind::Identifier(_)
    )
}

fn normalize_runtime_class_alias_key(name: &str) -> String {
    normalize_runtime_class_alias_target(name).to_ascii_lowercase()
}

fn normalize_runtime_class_alias_target(name: &str) -> String {
    name.trim_start_matches('\\').to_string()
}

fn literal_member_name_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value, _) if !value.contains('\0') => Some(value.clone()),
        Expr::Int(value, _) => Some(value.to_string()),
        _ => None,
    }
}

fn reject_named_language_construct_arguments(
    argument_names: &[Option<String>],
    span: SourceSpan,
) -> Result<()> {
    if argument_names.iter().any(Option::is_some) {
        return Err(Diagnostic::new(
            "named arguments are unsupported for this language construct",
            Some(span),
        ));
    }
    Ok(())
}

fn reject_unpacked_language_construct_arguments(
    argument_unpacks: &[bool],
    span: SourceSpan,
) -> Result<()> {
    if argument_unpacks.iter().any(|unpack| *unpack) {
        return Err(Diagnostic::new(
            "argument unpacking is unsupported for this language construct",
            Some(span),
        ));
    }
    Ok(())
}

fn compose_traits(traits: &mut [TraitDecl]) -> Result<()> {
    let originals = traits.to_vec();
    let mut cache = HashMap::new();
    let mut visiting = HashSet::new();
    for trait_decl in traits {
        *trait_decl = compose_trait_decl(&trait_decl.name, &originals, &mut visiting, &mut cache)?;
    }
    Ok(())
}

fn compose_trait_decl(
    name: &str,
    traits: &[TraitDecl],
    visiting: &mut HashSet<String>,
    cache: &mut HashMap<String, TraitDecl>,
) -> Result<TraitDecl> {
    let lookup_name = name.to_ascii_lowercase();
    if let Some(cached) = cache.get(&lookup_name) {
        return Ok(cached.clone());
    }
    if !visiting.insert(lookup_name.clone()) {
        return Err(Diagnostic::new(
            format!("Trait \"{name}\" circular reference detected"),
            None,
        ));
    }
    let Some(trait_decl) = traits
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(name))
    else {
        visiting.remove(&lookup_name);
        return Err(Diagnostic::new(format!("Trait \"{name}\" not found"), None));
    };
    let mut composed = trait_decl.clone();
    for trait_use in &trait_decl.trait_uses {
        let used_trait = compose_trait_decl(&trait_use.name, traits, visiting, cache)?;
        import_trait_members_into_trait(&mut composed, &used_trait);
    }
    visiting.remove(&lookup_name);
    cache.insert(lookup_name, composed.clone());
    Ok(composed)
}

fn import_trait_members_into_trait(target: &mut TraitDecl, source: &TraitDecl) {
    for property in &source.properties {
        if !target
            .properties
            .iter()
            .any(|candidate| candidate.name == property.name)
        {
            target.properties.push(property.clone());
        }
    }
    for property in &source.static_properties {
        if !target
            .static_properties
            .iter()
            .any(|candidate| candidate.name == property.name)
        {
            target.static_properties.push(property.clone());
        }
    }
    for constant in &source.constants {
        if !target
            .constants
            .iter()
            .any(|candidate| candidate.name.eq_ignore_ascii_case(&constant.name))
        {
            target.constants.push(constant.clone());
        }
    }
    for method in &source.methods {
        if !target
            .methods
            .iter()
            .any(|candidate| candidate.name.eq_ignore_ascii_case(&method.name))
        {
            target
                .methods
                .push(method_with_trait_origin(method, source));
        }
    }
}

fn compose_class_traits(classes: &mut [ClassDecl], traits: &[TraitDecl]) -> Result<()> {
    for class in classes {
        if class.trait_uses.is_empty() {
            continue;
        }
        let own_method_names = class
            .methods
            .iter()
            .map(|method| method.name.to_ascii_lowercase())
            .collect::<HashSet<_>>();
        let mut imported_method_names = HashSet::new();
        let trait_uses = class.trait_uses.clone();
        for trait_use in trait_uses {
            let Some(trait_decl) = traits
                .iter()
                .find(|candidate| candidate.name.eq_ignore_ascii_case(&trait_use.name))
            else {
                return Err(Diagnostic::new(
                    format!("Trait \"{}\" not found", trait_use.name),
                    Some(trait_use.span),
                ));
            };
            import_trait_members_into_class(
                class,
                trait_decl,
                &own_method_names,
                &mut imported_method_names,
            )?;
        }
    }
    Ok(())
}

fn import_trait_members_into_class(
    class: &mut ClassDecl,
    trait_decl: &TraitDecl,
    own_method_names: &HashSet<String>,
    imported_method_names: &mut HashSet<String>,
) -> Result<()> {
    for property in &trait_decl.properties {
        if !class
            .properties
            .iter()
            .any(|candidate| candidate.name == property.name)
        {
            class.properties.push(property.clone());
        }
    }
    for property in &trait_decl.static_properties {
        if !class
            .static_properties
            .iter()
            .any(|candidate| candidate.name == property.name)
        {
            class.static_properties.push(property.clone());
        }
    }
    for constant in &trait_decl.constants {
        if !class
            .constants
            .iter()
            .any(|candidate| candidate.name.eq_ignore_ascii_case(&constant.name))
        {
            class.constants.push(constant.clone());
        }
    }
    for method in &trait_decl.methods {
        let method_key = method.name.to_ascii_lowercase();
        if own_method_names.contains(&method_key) {
            continue;
        }
        if !imported_method_names.insert(method_key) {
            return Err(Diagnostic::new(
                format!(
                    "Trait method {}::{} has not been applied because of a collision",
                    trait_decl.name, method.name
                ),
                Some(class.span),
            ));
        }
        class
            .methods
            .push(method_with_trait_origin(method, trait_decl));
    }
    Ok(())
}

fn method_with_trait_origin(method: &MethodDecl, trait_decl: &TraitDecl) -> MethodDecl {
    let mut imported = method.clone();
    if imported.trait_name.is_none() {
        imported.trait_name = Some(trait_decl.name.clone());
    }
    imported
}

fn validate_class_names(classes: &[ClassDecl], traits: &[TraitDecl]) -> Result<()> {
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
    for trait_decl in traits {
        let lookup_name = trait_decl.name.to_ascii_lowercase();
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot declare trait {lookup_name}, because the name is already in use"),
                Some(trait_decl.span),
            ));
        }
    }
    Ok(())
}

fn validate_trait_names(traits: &[TraitDecl]) -> Result<()> {
    let mut names = HashSet::new();
    for trait_decl in traits {
        let lookup_name = trait_decl.name.to_ascii_lowercase();
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot declare trait {lookup_name}, because the name is already in use"),
                Some(trait_decl.span),
            ));
        }
    }
    Ok(())
}

fn validate_parent_class_names(classes: &[ClassDecl]) -> Result<()> {
    let names = classes
        .iter()
        .filter(|class| !class.is_interface)
        .map(|class| class.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for class in classes {
        if class.is_interface {
            continue;
        }
        let Some(parent_name) = &class.parent_name else {
            continue;
        };
        if parent_name.eq_ignore_ascii_case("stdClass")
            || is_modeled_spl_iterator_class_name(parent_name)
            || is_modeled_builtin_exception_class_name(parent_name)
            || parent_name.eq_ignore_ascii_case("Generator")
        {
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

fn validate_interface_references(classes: &[ClassDecl]) -> Result<()> {
    let interface_names = classes
        .iter()
        .filter(|class| class.is_interface)
        .map(|class| class.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for class in classes {
        let mut seen_interfaces = HashSet::new();
        for interface_name in &class.interfaces {
            if !seen_interfaces.insert(interface_name.to_ascii_lowercase()) {
                let declaration_kind = if class.is_interface {
                    "Interface"
                } else {
                    "Class"
                };
                return Err(Diagnostic::new(
                    format!(
                        "{declaration_kind} {} cannot implement previously implemented interface {interface_name}",
                        class.name
                    ),
                    Some(class.span),
                ));
            }
            if class.is_interface && class.name.eq_ignore_ascii_case(interface_name) {
                return Err(Diagnostic::new(
                    format!("Interface \"{interface_name}\" not found"),
                    Some(class.span),
                ));
            }
            if interface_names.contains(&interface_name.to_ascii_lowercase())
                || is_modeled_builtin_interface_name(interface_name)
            {
                continue;
            }
            return Err(Diagnostic::new(
                format!("Interface \"{interface_name}\" not found"),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn class_hierarchy_implements_interface(
    classes: &[ClassDecl],
    class: &ClassDecl,
    interface_name: &str,
) -> bool {
    if class
        .interfaces
        .iter()
        .any(|interface| modeled_builtin_interface_extends(interface, interface_name))
    {
        return true;
    }
    class
        .parent_name
        .as_deref()
        .and_then(|parent_name| {
            classes
                .iter()
                .find(|candidate| candidate.name.eq_ignore_ascii_case(parent_name))
        })
        .is_some_and(|parent| class_hierarchy_implements_interface(classes, parent, interface_name))
}

fn validate_traversable_implementations(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        if class.is_interface {
            continue;
        }
        if class_hierarchy_implements_interface(classes, class, "Iterator")
            && class_hierarchy_implements_interface(classes, class, "IteratorAggregate")
        {
            return Err(Diagnostic::new(
                format!(
                    "Class {} cannot implement both Iterator and IteratorAggregate at the same time",
                    class.name
                ),
                Some(class.span),
            ));
        }
        if class.is_abstract {
            continue;
        }
        if class_hierarchy_implements_interface(classes, class, "Traversable")
            && !class_hierarchy_implements_interface(classes, class, "Iterator")
            && !class_hierarchy_implements_interface(classes, class, "IteratorAggregate")
        {
            return Err(Diagnostic::new(
                format!(
                    "Class {} must implement interface Traversable as part of either Iterator or IteratorAggregate",
                    class.name
                ),
                Some(
                    if class
                        .interfaces
                        .iter()
                        .any(|interface| interface.eq_ignore_ascii_case("Traversable"))
                    {
                        class.span
                    } else {
                        SourceSpan::new(0, 0, 0, 0)
                    },
                ),
            ));
        }
    }
    Ok(())
}

fn validate_method_signature_compatibility(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        for method in &class.methods {
            if method.visibility == PropertyVisibility::Private {
                continue;
            }
            if let Some((parent_class, parent_method)) =
                find_visible_parent_method(class, &method.name, classes)
            {
                validate_method_signature_pair(
                    class,
                    method,
                    parent_class,
                    parent_method,
                    classes,
                )?;
            }

            let mut seen_interfaces = HashSet::new();
            let mut interface_methods = Vec::new();
            for interface_name in &class.interfaces {
                collect_interface_methods(
                    interface_name,
                    &method.name,
                    classes,
                    &mut seen_interfaces,
                    &mut interface_methods,
                );
            }
            for (interface, interface_method) in interface_methods {
                validate_method_signature_pair(
                    class,
                    method,
                    interface,
                    interface_method,
                    classes,
                )?;
            }
        }
    }
    Ok(())
}

fn find_visible_parent_method<'a>(
    class: &ClassDecl,
    method_name: &str,
    classes: &'a [ClassDecl],
) -> Option<(&'a ClassDecl, &'a MethodDecl)> {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let parent = find_class(classes, name)?;
        if let Some(method) = parent.methods.iter().find(|candidate| {
            candidate.visibility != PropertyVisibility::Private
                && candidate.name.eq_ignore_ascii_case(method_name)
        }) {
            return Some((parent, method));
        }
        parent_name = parent.parent_name.as_deref();
    }
    None
}

fn collect_interface_methods<'a>(
    interface_name: &str,
    method_name: &str,
    classes: &'a [ClassDecl],
    seen: &mut HashSet<String>,
    methods: &mut Vec<(&'a ClassDecl, &'a MethodDecl)>,
) {
    let lookup_name = interface_name.trim_start_matches('\\').to_ascii_lowercase();
    if !seen.insert(lookup_name) {
        return;
    }
    let Some(interface) = find_class(classes, interface_name) else {
        return;
    };
    if !interface.is_interface {
        return;
    }
    for method in &interface.methods {
        if method.name.eq_ignore_ascii_case(method_name) {
            methods.push((interface, method));
        }
    }
    for parent_name in &interface.interfaces {
        collect_interface_methods(parent_name, method_name, classes, seen, methods);
    }
}

fn validate_method_signature_pair(
    class: &ClassDecl,
    method: &MethodDecl,
    parent_class: &ClassDecl,
    parent_method: &MethodDecl,
    classes: &[ClassDecl],
) -> Result<()> {
    if parent_method.return_by_ref && !method.return_by_ref {
        return Err(method_signature_compatibility_error(
            class,
            method,
            parent_class,
            parent_method,
        ));
    }

    for (index, parent_parameter) in parent_method.parameters.iter().enumerate() {
        let Some(parameter) = method.parameters.get(index) else {
            return Err(method_signature_compatibility_error(
                class,
                method,
                parent_class,
                parent_method,
            ));
        };
        if parameter.by_ref != parent_parameter.by_ref
            || parameter.is_variadic != parent_parameter.is_variadic
        {
            return Err(method_signature_compatibility_error(
                class,
                method,
                parent_class,
                parent_method,
            ));
        }
        if !parameter_type_is_contravariant(parameter, parent_parameter, classes) {
            return Err(method_signature_compatibility_error(
                class,
                method,
                parent_class,
                parent_method,
            ));
        }
    }

    if let Some(parent_return_type) = &parent_method.return_type {
        let Some(return_type) = &method.return_type else {
            return Err(method_signature_compatibility_error(
                class,
                method,
                parent_class,
                parent_method,
            ));
        };
        if !type_hint_is_subtype(return_type, parent_return_type, classes) {
            if let Some(unavailable_name) =
                unresolved_compatibility_class(return_type, parent_return_type, classes)
            {
                return Err(method_signature_unresolved_compatibility_error(
                    class,
                    method,
                    parent_class,
                    parent_method,
                    &unavailable_name,
                ));
            }
            return Err(method_signature_compatibility_error(
                class,
                method,
                parent_class,
                parent_method,
            ));
        }
    }

    Ok(())
}

fn unresolved_compatibility_class(
    candidate: &TypeHint,
    target: &TypeHint,
    classes: &[ClassDecl],
) -> Option<String> {
    if !type_hint_mentions_iterable_or_traversable(target) {
        return None;
    }
    first_unavailable_intersection_class(candidate, classes)
}

fn type_hint_mentions_iterable_or_traversable(type_hint: &TypeHint) -> bool {
    match type_hint {
        TypeHint::Iterable => true,
        TypeHint::Class(name) => name.eq_ignore_ascii_case("Traversable"),
        TypeHint::Nullable(inner) => type_hint_mentions_iterable_or_traversable(inner),
        TypeHint::Union(types) | TypeHint::Intersection(types) => {
            types.iter().any(type_hint_mentions_iterable_or_traversable)
        }
        TypeHint::Null
        | TypeHint::Array
        | TypeHint::Int
        | TypeHint::Float
        | TypeHint::String
        | TypeHint::Bool
        | TypeHint::True
        | TypeHint::False
        | TypeHint::Callable
        | TypeHint::Object
        | TypeHint::Mixed
        | TypeHint::Void
        | TypeHint::Never
        | TypeHint::Static => false,
    }
}

fn first_unavailable_intersection_class(
    type_hint: &TypeHint,
    classes: &[ClassDecl],
) -> Option<String> {
    match type_hint {
        TypeHint::Intersection(types) => types.iter().find_map(|member| {
            if let TypeHint::Class(name) = member {
                if !class_type_name_is_available(name, classes) {
                    return Some(name.clone());
                }
            }
            None
        }),
        TypeHint::Nullable(inner) => first_unavailable_intersection_class(inner, classes),
        TypeHint::Union(types) => types
            .iter()
            .find_map(|member| first_unavailable_intersection_class(member, classes)),
        TypeHint::Null
        | TypeHint::Array
        | TypeHint::Int
        | TypeHint::Float
        | TypeHint::String
        | TypeHint::Bool
        | TypeHint::True
        | TypeHint::False
        | TypeHint::Callable
        | TypeHint::Object
        | TypeHint::Iterable
        | TypeHint::Mixed
        | TypeHint::Void
        | TypeHint::Never
        | TypeHint::Static
        | TypeHint::Class(_) => None,
    }
}

fn class_type_name_is_available(name: &str, classes: &[ClassDecl]) -> bool {
    is_modeled_builtin_interface_name(name)
        || name.eq_ignore_ascii_case("stdClass")
        || name.eq_ignore_ascii_case("ArrayIterator")
        || name.eq_ignore_ascii_case("IteratorIterator")
        || name.eq_ignore_ascii_case("ArrayObject")
        || name.eq_ignore_ascii_case("Generator")
        || is_modeled_builtin_exception_class_name(name)
        || find_class(classes, name).is_some()
}

fn parameter_type_is_contravariant(
    parameter: &FunctionParameter,
    parent_parameter: &FunctionParameter,
    classes: &[ClassDecl],
) -> bool {
    match (&parameter.type_hint, &parent_parameter.type_hint) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(type_hint), Some(parent_type_hint)) => {
            type_hint_is_subtype(parent_type_hint, type_hint, classes)
        }
    }
}

fn method_signature_compatibility_error(
    class: &ClassDecl,
    method: &MethodDecl,
    parent_class: &ClassDecl,
    parent_method: &MethodDecl,
) -> Diagnostic {
    Diagnostic::new(
        format!(
            "Declaration of {}::{} must be compatible with {}::{}",
            class.name,
            method_signature_display(method),
            parent_class.name,
            method_signature_display(parent_method)
        ),
        Some(method.span),
    )
}

fn method_signature_unresolved_compatibility_error(
    class: &ClassDecl,
    method: &MethodDecl,
    parent_class: &ClassDecl,
    parent_method: &MethodDecl,
    unavailable_name: &str,
) -> Diagnostic {
    Diagnostic::new(
        format!(
            "Could not check compatibility between {}::{} and {}::{}, because class {} is not available",
            class.name,
            method_signature_display_canonical(method),
            parent_class.name,
            method_signature_display_canonical(parent_method),
            unavailable_name
        ),
        Some(method.span),
    )
}

fn method_signature_display(method: &MethodDecl) -> String {
    method_signature_display_with(method, type_hint_display)
}

fn method_signature_display_canonical(method: &MethodDecl) -> String {
    method_signature_display_with(method, type_hint_display_canonical)
}

fn method_signature_display_with(
    method: &MethodDecl,
    display_type: fn(&TypeHint) -> String,
) -> String {
    let mut signature = String::new();
    if method.return_by_ref {
        signature.push('&');
    }
    signature.push_str(&method.name);
    signature.push('(');
    for (index, parameter) in method.parameters.iter().enumerate() {
        if index > 0 {
            signature.push_str(", ");
        }
        signature.push_str(&parameter_signature_display_with(parameter, display_type));
    }
    signature.push(')');
    if let Some(return_type) = &method.return_type {
        signature.push_str(": ");
        signature.push_str(&display_type(return_type));
    }
    signature
}

fn parameter_signature_display_with(
    parameter: &FunctionParameter,
    display_type: fn(&TypeHint) -> String,
) -> String {
    let mut display = String::new();
    if let Some(type_hint) = &parameter.type_hint {
        display.push_str(&display_type(type_hint));
        display.push(' ');
    }
    if parameter.by_ref {
        display.push('&');
    }
    if parameter.is_variadic {
        display.push_str("...");
    }
    display.push('$');
    display.push_str(&parameter.name);
    if let Some(default_value) = &parameter.default_value {
        display.push_str(" = ");
        display.push_str(&parameter_default_display(default_value));
    }
    display
}

fn parameter_default_display(default_value: &Expr) -> String {
    match default_value {
        Expr::Null(_) => "null".to_string(),
        Expr::Bool(value, _) => value.to_string(),
        Expr::Int(value, _) => value.to_string(),
        Expr::Float(value, _) => value.to_string(),
        Expr::String(value, _) => format!("'{}'", value.replace('\'', "\\'")),
        Expr::Array { elements, .. } if elements.is_empty() => "[]".to_string(),
        _ => "<expression>".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeAtom {
    Null,
    Array,
    Int,
    Float,
    String,
    Bool,
    True,
    False,
    Callable,
    Object,
    Mixed,
    Never,
    Static,
    Class(String),
}

#[derive(Debug, Clone)]
struct TypeAlternative {
    atoms: Vec<TypeAtom>,
}

fn type_hint_is_subtype(candidate: &TypeHint, target: &TypeHint, classes: &[ClassDecl]) -> bool {
    let candidate_alternatives = type_hint_alternatives(candidate);
    let target_alternatives = type_hint_alternatives(target);
    candidate_alternatives.iter().all(|candidate| {
        target_alternatives
            .iter()
            .any(|target| type_alternative_is_subtype(candidate, target, classes))
    })
}

fn type_hint_alternatives(type_hint: &TypeHint) -> Vec<TypeAlternative> {
    match type_hint {
        TypeHint::Null => vec![TypeAlternative {
            atoms: vec![TypeAtom::Null],
        }],
        TypeHint::Array => vec![TypeAlternative {
            atoms: vec![TypeAtom::Array],
        }],
        TypeHint::Int => vec![TypeAlternative {
            atoms: vec![TypeAtom::Int],
        }],
        TypeHint::Float => vec![TypeAlternative {
            atoms: vec![TypeAtom::Float],
        }],
        TypeHint::String => vec![TypeAlternative {
            atoms: vec![TypeAtom::String],
        }],
        TypeHint::Bool => vec![TypeAlternative {
            atoms: vec![TypeAtom::Bool],
        }],
        TypeHint::True => vec![TypeAlternative {
            atoms: vec![TypeAtom::True],
        }],
        TypeHint::False => vec![TypeAlternative {
            atoms: vec![TypeAtom::False],
        }],
        TypeHint::Callable => vec![TypeAlternative {
            atoms: vec![TypeAtom::Callable],
        }],
        TypeHint::Object => vec![TypeAlternative {
            atoms: vec![TypeAtom::Object],
        }],
        TypeHint::Iterable => vec![
            TypeAlternative {
                atoms: vec![TypeAtom::Array],
            },
            TypeAlternative {
                atoms: vec![TypeAtom::Class("Traversable".to_string())],
            },
        ],
        TypeHint::Mixed | TypeHint::Void => vec![TypeAlternative {
            atoms: vec![TypeAtom::Mixed],
        }],
        TypeHint::Never => vec![TypeAlternative {
            atoms: vec![TypeAtom::Never],
        }],
        TypeHint::Static => vec![TypeAlternative {
            atoms: vec![TypeAtom::Static],
        }],
        TypeHint::Nullable(inner) => {
            let mut alternatives = type_hint_alternatives(inner);
            alternatives.push(TypeAlternative {
                atoms: vec![TypeAtom::Null],
            });
            alternatives
        }
        TypeHint::Union(types) => types.iter().flat_map(type_hint_alternatives).collect(),
        TypeHint::Intersection(types) => {
            let mut alternatives = vec![TypeAlternative { atoms: Vec::new() }];
            for type_hint in types {
                let member_alternatives = type_hint_alternatives(type_hint);
                let mut combined = Vec::new();
                for current in &alternatives {
                    for member in &member_alternatives {
                        let mut atoms = current.atoms.clone();
                        atoms.extend(member.atoms.clone());
                        combined.push(TypeAlternative { atoms });
                    }
                }
                alternatives = combined;
            }
            alternatives
        }
        TypeHint::Class(class_name) => vec![TypeAlternative {
            atoms: vec![TypeAtom::Class(class_name.clone())],
        }],
    }
}

fn type_alternative_is_subtype(
    candidate: &TypeAlternative,
    target: &TypeAlternative,
    classes: &[ClassDecl],
) -> bool {
    target.atoms.iter().all(|target_atom| {
        candidate
            .atoms
            .iter()
            .any(|candidate_atom| type_atom_is_subtype(candidate_atom, target_atom, classes))
    })
}

fn type_atom_is_subtype(candidate: &TypeAtom, target: &TypeAtom, classes: &[ClassDecl]) -> bool {
    if matches!(candidate, TypeAtom::Never) || matches!(target, TypeAtom::Mixed) {
        return true;
    }
    if candidate == target {
        return true;
    }
    match (candidate, target) {
        (TypeAtom::True | TypeAtom::False, TypeAtom::Bool) => true,
        (TypeAtom::Static, TypeAtom::Object) => true,
        (TypeAtom::Class(_), TypeAtom::Object) => true,
        (TypeAtom::Class(candidate), TypeAtom::Class(target)) => {
            class_type_name_is_subtype(candidate, target, classes)
        }
        _ => false,
    }
}

fn class_type_name_is_subtype(
    candidate_name: &str,
    target_name: &str,
    classes: &[ClassDecl],
) -> bool {
    if candidate_name.eq_ignore_ascii_case(target_name) {
        return true;
    }
    if builtin_class_type_is_subtype(candidate_name, target_name) {
        return true;
    }
    let Some(candidate) = find_class(classes, candidate_name) else {
        return false;
    };
    if candidate.is_interface {
        return interface_type_extends(candidate, target_name, classes);
    }
    class_type_extends_or_implements(candidate, target_name, classes)
}

fn builtin_class_type_is_subtype(candidate_name: &str, target_name: &str) -> bool {
    if target_name.eq_ignore_ascii_case("Traversable")
        && (candidate_name.eq_ignore_ascii_case("Iterator")
            || candidate_name.eq_ignore_ascii_case("IteratorAggregate"))
    {
        return true;
    }
    let implemented = if candidate_name.eq_ignore_ascii_case("ArrayIterator") {
        &["ArrayAccess", "Countable", "Iterator", "Traversable"][..]
    } else if candidate_name.eq_ignore_ascii_case("IteratorIterator") {
        &["Iterator", "Traversable"][..]
    } else if candidate_name.eq_ignore_ascii_case("ArrayObject") {
        &[
            "ArrayAccess",
            "Countable",
            "IteratorAggregate",
            "Traversable",
        ][..]
    } else if candidate_name.eq_ignore_ascii_case("Generator") {
        &["Iterator", "Traversable"][..]
    } else {
        &[][..]
    };
    implemented
        .iter()
        .any(|interface| interface.eq_ignore_ascii_case(target_name))
}

fn class_type_extends_or_implements(
    class: &ClassDecl,
    target_name: &str,
    classes: &[ClassDecl],
) -> bool {
    if class
        .interfaces
        .iter()
        .any(|interface| interface_name_is_subtype(interface, target_name, classes))
    {
        return true;
    }
    let Some(parent_name) = class.parent_name.as_deref() else {
        return false;
    };
    if parent_name.eq_ignore_ascii_case(target_name)
        || builtin_class_type_is_subtype(parent_name, target_name)
    {
        return true;
    }
    find_class(classes, parent_name)
        .is_some_and(|parent| class_type_extends_or_implements(parent, target_name, classes))
}

fn interface_name_is_subtype(
    interface_name: &str,
    target_name: &str,
    classes: &[ClassDecl],
) -> bool {
    if interface_name.eq_ignore_ascii_case(target_name)
        || builtin_class_type_is_subtype(interface_name, target_name)
    {
        return true;
    }
    find_class(classes, interface_name)
        .filter(|interface| interface.is_interface)
        .is_some_and(|interface| interface_type_extends(interface, target_name, classes))
}

fn interface_type_extends(interface: &ClassDecl, target_name: &str, classes: &[ClassDecl]) -> bool {
    interface
        .interfaces
        .iter()
        .any(|parent_name| interface_name_is_subtype(parent_name, target_name, classes))
}

fn type_hint_display(type_hint: &TypeHint) -> String {
    match type_hint {
        TypeHint::Null => "null".to_string(),
        TypeHint::Array => "array".to_string(),
        TypeHint::Int => "int".to_string(),
        TypeHint::Float => "float".to_string(),
        TypeHint::String => "string".to_string(),
        TypeHint::Bool => "bool".to_string(),
        TypeHint::True => "true".to_string(),
        TypeHint::False => "false".to_string(),
        TypeHint::Callable => "callable".to_string(),
        TypeHint::Object => "object".to_string(),
        TypeHint::Iterable => "Traversable|array".to_string(),
        TypeHint::Mixed => "mixed".to_string(),
        TypeHint::Void => "void".to_string(),
        TypeHint::Never => "never".to_string(),
        TypeHint::Static => "static".to_string(),
        TypeHint::Nullable(inner) => match inner.as_ref() {
            TypeHint::Iterable => "Traversable|array|null".to_string(),
            TypeHint::Union(_) | TypeHint::Intersection(_) => {
                format!("{}|null", type_hint_display(inner))
            }
            _ => format!("?{}", type_hint_display(inner)),
        },
        TypeHint::Union(types) => types
            .iter()
            .map(|member| type_hint_union_member_display(member))
            .collect::<Vec<_>>()
            .join("|"),
        TypeHint::Intersection(types) => types
            .iter()
            .map(type_hint_display)
            .collect::<Vec<_>>()
            .join("&"),
        TypeHint::Class(class_name) => class_name.clone(),
    }
}

fn type_hint_display_canonical(type_hint: &TypeHint) -> String {
    match type_hint {
        TypeHint::Union(types) => {
            let mut members = types.iter().collect::<Vec<_>>();
            members.sort_by_key(|member| !matches!(member, TypeHint::Intersection(_)));
            members
                .into_iter()
                .map(|member| match member {
                    TypeHint::Intersection(_) => {
                        format!("({})", type_hint_display_canonical(member))
                    }
                    _ => type_hint_display_canonical(member),
                })
                .collect::<Vec<_>>()
                .join("|")
        }
        TypeHint::Intersection(types) => types
            .iter()
            .map(type_hint_display_canonical)
            .collect::<Vec<_>>()
            .join("&"),
        TypeHint::Nullable(inner) => match inner.as_ref() {
            TypeHint::Iterable => "Traversable|array|null".to_string(),
            TypeHint::Union(_) | TypeHint::Intersection(_) => {
                format!("{}|null", type_hint_display_canonical(inner))
            }
            _ => format!("?{}", type_hint_display_canonical(inner)),
        },
        _ => type_hint_display(type_hint),
    }
}

fn type_hint_union_member_display(type_hint: &TypeHint) -> String {
    match type_hint {
        TypeHint::Intersection(_) => format!("({})", type_hint_display(type_hint)),
        _ => type_hint_display(type_hint),
    }
}

fn validate_override_attributes(classes: &[ClassDecl], traits: &[TraitDecl]) -> Result<()> {
    for class in classes {
        for method in &class.methods {
            if !method.has_override_attribute {
                continue;
            }
            if method_override_target_exists(class, method, classes, traits) {
                continue;
            }
            return Err(Diagnostic::new(
                format!(
                    "{}::{}() has #[\\Override] attribute, but no matching parent method exists",
                    class.name, method.name
                ),
                Some(method.span),
            ));
        }
        for property in &class.properties {
            if !property.has_override_attribute {
                continue;
            }
            if property_override_target_exists(class, &property.name, false, classes) {
                continue;
            }
            return Err(Diagnostic::new(
                format!(
                    "{}::${} has #[\\Override] attribute, but no matching parent property exists",
                    class.name, property.name
                ),
                Some(property.span),
            ));
        }
        for property in &class.static_properties {
            if !property.has_override_attribute {
                continue;
            }
            if property_override_target_exists(class, &property.name, true, classes) {
                continue;
            }
            return Err(Diagnostic::new(
                format!(
                    "{}::${} has #[\\Override] attribute, but no matching parent property exists",
                    class.name, property.name
                ),
                Some(property.span),
            ));
        }
    }
    Ok(())
}

fn method_override_target_exists(
    class: &ClassDecl,
    method: &MethodDecl,
    classes: &[ClassDecl],
    traits: &[TraitDecl],
) -> bool {
    parent_method_override_target_exists(class, method, classes)
        || class_interface_method_exists(class, &method.name, classes)
        || trait_abstract_method_exists(class, &method.name, traits)
}

fn parent_method_override_target_exists(
    class: &ClassDecl,
    method: &MethodDecl,
    classes: &[ClassDecl],
) -> bool {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let Some(parent) = find_class(classes, name) else {
            break;
        };
        if parent.methods.iter().any(|candidate| {
            method_can_satisfy_override(candidate, &method.name)
                && (!method.name.eq_ignore_ascii_case("__construct") || candidate.is_abstract)
        }) {
            return true;
        }
        parent_name = parent.parent_name.as_deref();
    }
    false
}

fn method_can_satisfy_override(method: &MethodDecl, name: &str) -> bool {
    method.visibility != PropertyVisibility::Private && method.name.eq_ignore_ascii_case(name)
}

fn class_interface_method_exists(
    class: &ClassDecl,
    method_name: &str,
    classes: &[ClassDecl],
) -> bool {
    let mut current = Some(class);
    let mut seen_classes = HashSet::new();
    while let Some(candidate) = current {
        if !seen_classes.insert(candidate.name.to_ascii_lowercase()) {
            break;
        }
        let mut seen_interfaces = HashSet::new();
        if candidate.interfaces.iter().any(|interface_name| {
            interface_method_exists(interface_name, method_name, classes, &mut seen_interfaces)
        }) {
            return true;
        }
        current = candidate
            .parent_name
            .as_deref()
            .and_then(|name| find_class(classes, name));
    }
    false
}

fn interface_method_exists(
    interface_name: &str,
    method_name: &str,
    classes: &[ClassDecl],
    seen: &mut HashSet<String>,
) -> bool {
    let lookup_name = interface_name.trim_start_matches('\\').to_ascii_lowercase();
    if !seen.insert(lookup_name) {
        return false;
    }
    if modeled_builtin_interface_method_exists(interface_name, method_name) {
        return true;
    }
    let Some(interface) = find_class(classes, interface_name) else {
        return false;
    };
    if !interface.is_interface {
        return false;
    }
    interface
        .methods
        .iter()
        .any(|method| method.name.eq_ignore_ascii_case(method_name))
        || interface
            .interfaces
            .iter()
            .any(|parent_name| interface_method_exists(parent_name, method_name, classes, seen))
}

fn modeled_builtin_interface_method_exists(interface_name: &str, method_name: &str) -> bool {
    let interface_name = interface_name.trim_start_matches('\\').to_ascii_lowercase();
    let method_name = method_name.to_ascii_lowercase();
    matches!(
        (interface_name.as_str(), method_name.as_str()),
        ("arrayaccess", "offsetexists")
            | ("arrayaccess", "offsetget")
            | ("arrayaccess", "offsetset")
            | ("arrayaccess", "offsetunset")
            | ("iterator", "current")
            | ("iterator", "key")
            | ("iterator", "next")
            | ("iterator", "rewind")
            | ("iterator", "valid")
            | ("iteratoraggregate", "getiterator")
            | ("outeriterator", "getinneriterator")
            | ("recursiveiterator", "getchildren")
            | ("recursiveiterator", "haschildren")
            | ("serializable", "serialize")
            | ("seekableiterator", "seek")
            | ("serializable", "unserialize")
            | ("stringable", "__tostring")
    )
}

fn trait_abstract_method_exists(
    class: &ClassDecl,
    method_name: &str,
    traits: &[TraitDecl],
) -> bool {
    class.trait_uses.iter().any(|trait_use| {
        find_trait(traits, &trait_use.name).is_some_and(|trait_decl| {
            trait_decl
                .methods
                .iter()
                .any(|method| method.is_abstract && method.name.eq_ignore_ascii_case(method_name))
        })
    })
}

fn property_override_target_exists(
    class: &ClassDecl,
    property_name: &str,
    is_static: bool,
    classes: &[ClassDecl],
) -> bool {
    parent_property_override_target_exists(class, property_name, is_static, classes)
        || class_interface_property_exists(class, property_name, is_static, classes)
}

fn parent_property_override_target_exists(
    class: &ClassDecl,
    property_name: &str,
    is_static: bool,
    classes: &[ClassDecl],
) -> bool {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let Some(parent) = find_class(classes, name) else {
            break;
        };
        let has_property = if is_static {
            parent.static_properties.iter().any(|candidate| {
                candidate.visibility != PropertyVisibility::Private
                    && candidate.name == property_name
            })
        } else {
            parent.properties.iter().any(|candidate| {
                candidate.visibility != PropertyVisibility::Private
                    && candidate.name == property_name
            })
        };
        if has_property {
            return true;
        }
        parent_name = parent.parent_name.as_deref();
    }
    false
}

fn class_interface_property_exists(
    class: &ClassDecl,
    property_name: &str,
    is_static: bool,
    classes: &[ClassDecl],
) -> bool {
    let mut current = Some(class);
    let mut seen_classes = HashSet::new();
    while let Some(candidate) = current {
        if !seen_classes.insert(candidate.name.to_ascii_lowercase()) {
            break;
        }
        let mut seen_interfaces = HashSet::new();
        if candidate.interfaces.iter().any(|interface_name| {
            interface_property_exists(
                interface_name,
                property_name,
                is_static,
                classes,
                &mut seen_interfaces,
            )
        }) {
            return true;
        }
        current = candidate
            .parent_name
            .as_deref()
            .and_then(|name| find_class(classes, name));
    }
    false
}

fn interface_property_exists(
    interface_name: &str,
    property_name: &str,
    is_static: bool,
    classes: &[ClassDecl],
    seen: &mut HashSet<String>,
) -> bool {
    let lookup_name = interface_name.trim_start_matches('\\').to_ascii_lowercase();
    if !seen.insert(lookup_name) {
        return false;
    }
    let Some(interface) = find_class(classes, interface_name) else {
        return false;
    };
    if !interface.is_interface {
        return false;
    }
    let has_property = if is_static {
        interface
            .static_properties
            .iter()
            .any(|property| property.name == property_name)
    } else {
        interface
            .properties
            .iter()
            .any(|property| property.name == property_name)
    };
    has_property
        || interface.interfaces.iter().any(|parent_name| {
            interface_property_exists(parent_name, property_name, is_static, classes, seen)
        })
}

fn find_class<'a>(classes: &'a [ClassDecl], name: &str) -> Option<&'a ClassDecl> {
    classes
        .iter()
        .find(|class| class.name.eq_ignore_ascii_case(name))
}

fn find_trait<'a>(traits: &'a [TraitDecl], name: &str) -> Option<&'a TraitDecl> {
    traits
        .iter()
        .find(|trait_decl| trait_decl.name.eq_ignore_ascii_case(name))
}

fn validate_builtin_attribute_targets(
    classes: &[ClassDecl],
    traits: &[TraitDecl],
    functions: &[FunctionDecl],
) -> Result<()> {
    for function in functions {
        if function.attributes.has_attribute {
            return Err(Diagnostic::new(
                "Attribute \"Attribute\" cannot target function (allowed targets: class)",
                Some(function.span),
            ));
        }
        validate_no_discard_function_target(
            function.attributes.no_discard_message.is_some(),
            function.return_type.as_ref(),
            &format!("Function {}", function.name),
            function.span,
        )?;
    }
    for class in classes {
        if class.attributes.has_attribute {
            if class.is_interface {
                return Err(Diagnostic::new(
                    format!("Cannot apply #[\\Attribute] to interface {}", class.name),
                    Some(class.span),
                ));
            }
            if class.is_abstract {
                return Err(Diagnostic::new(
                    format!(
                        "Cannot apply #[\\Attribute] to abstract class {}",
                        class.name
                    ),
                    Some(class.span),
                ));
            }
        }
        if class.attributes.has_allow_dynamic_properties && class.is_interface {
            return Err(Diagnostic::new(
                format!(
                    "Cannot apply #[\\AllowDynamicProperties] to interface {}",
                    class.name
                ),
                Some(class.span),
            ));
        }
        if class.attributes.deprecated_message.is_some() && class.is_interface {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Deprecated] to interface {}", class.name),
                Some(class.span),
            ));
        }
        if class.attributes.deprecated_message.is_some() && !class.is_interface {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Deprecated] to class {}", class.name),
                Some(class.span),
            ));
        }
        for method in &class.methods {
            if method.attributes.has_attribute {
                return Err(Diagnostic::new(
                    "Attribute \"Attribute\" cannot target method (allowed targets: class)",
                    Some(method.span),
                ));
            }
            if method.attributes.no_discard_message.is_some()
                && (method.name.eq_ignore_ascii_case("__construct")
                    || method.name.eq_ignore_ascii_case("__clone"))
            {
                return Err(Diagnostic::new(
                    format!(
                        "Method {}::{} cannot be #[\\NoDiscard]",
                        class.name, method.name
                    ),
                    Some(method.span),
                ));
            }
            validate_no_discard_function_target(
                method.attributes.no_discard_message.is_some(),
                method.return_type.as_ref(),
                &format!("Method {}::{}", class.name, method.name),
                method.span,
            )?;
        }
    }
    for trait_decl in traits {
        if trait_decl.attributes.has_attribute {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Attribute] to trait {}", trait_decl.name),
                Some(trait_decl.span),
            ));
        }
        if trait_decl.attributes.has_allow_dynamic_properties {
            return Err(Diagnostic::new(
                format!(
                    "Cannot apply #[\\AllowDynamicProperties] to trait {}",
                    trait_decl.name
                ),
                Some(trait_decl.span),
            ));
        }
    }
    Ok(())
}

fn validate_no_discard_function_target(
    has_no_discard: bool,
    return_type: Option<&TypeHint>,
    _display_name: &str,
    span: SourceSpan,
) -> Result<()> {
    if !has_no_discard {
        return Ok(());
    }
    match return_type {
        Some(TypeHint::Void) => Err(Diagnostic::new(
            "A void function does not return a value, but #[\\NoDiscard] requires a return value",
            Some(span),
        )),
        Some(TypeHint::Never) => Err(Diagnostic::new(
            "A never returning function does not return a value, but #[\\NoDiscard] requires a return value",
            Some(span),
        )),
        _ => Ok(()),
    }
}

fn validate_abstract_methods(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        if class.is_abstract {
            continue;
        }
        if let Some(method) = class
            .methods
            .iter()
            .find(|method| method.is_abstract && method.trait_name.is_none())
        {
            if parent_concrete_method_exists(class, &method.name, classes) {
                continue;
            }
            return Err(Diagnostic::new(
                format!(
                    "Class {} declares abstract method {}() and must therefore be declared abstract",
                    class.name, method.name
                ),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn parent_concrete_method_exists(
    class: &ClassDecl,
    method_name: &str,
    classes: &[ClassDecl],
) -> bool {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let Some(parent) = find_class(classes, name) else {
            break;
        };
        if parent
            .methods
            .iter()
            .any(|method| method_can_satisfy_override(method, method_name) && !method.is_abstract)
        {
            return true;
        }
        parent_name = parent.parent_name.as_deref();
    }
    false
}

fn validate_final_class_inheritance(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        if class.is_interface {
            continue;
        }
        let Some(parent_name) = &class.parent_name else {
            continue;
        };
        if parent_name.eq_ignore_ascii_case("Generator") {
            return Err(Diagnostic::new(
                format!("Class {} cannot extend final class Generator", class.name),
                Some(class.span),
            ));
        }
        let Some(parent) = classes
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(parent_name))
        else {
            continue;
        };
        if parent.is_final {
            return Err(Diagnostic::new(
                format!(
                    "Class {} cannot extend final class {parent_name}",
                    class.name
                ),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn validate_readonly_class_inheritance(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        if class.is_interface {
            continue;
        }
        let Some(parent_name) = &class.parent_name else {
            continue;
        };
        let parent = classes
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(parent_name));
        if !class.is_readonly {
            if let Some(parent) = parent {
                if parent.is_readonly {
                    return Err(Diagnostic::new(
                        format!(
                            "Non-readonly class {} cannot extend readonly class {parent_name}",
                            class.name
                        ),
                        Some(class.span),
                    ));
                }
            }
            continue;
        }
        if parent_name.eq_ignore_ascii_case("stdClass")
            || is_modeled_builtin_exception_class_name(parent_name)
        {
            return Err(Diagnostic::new(
                format!(
                    "Readonly class {} cannot extend non-readonly class {parent_name}",
                    class.name
                ),
                Some(class.span),
            ));
        }
        let Some(parent) = parent else {
            continue;
        };
        if !parent.is_readonly {
            return Err(Diagnostic::new(
                format!(
                    "Readonly class {} cannot extend non-readonly class {parent_name}",
                    class.name
                ),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

fn validate_property_override_set_visibility(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        let Some(parent_name) = &class.parent_name else {
            continue;
        };
        let Some(parent) = classes
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(parent_name))
        else {
            continue;
        };
        for property in &class.properties {
            let Some(parent_property) = parent
                .properties
                .iter()
                .find(|candidate| candidate.name == property.name)
            else {
                continue;
            };
            if visibility_rank(property.set_visibility)
                <= visibility_rank(parent_property.set_visibility)
            {
                continue;
            }
            return Err(Diagnostic::new(
                format!(
                    "Set access level of {}::${} must be {}(set) (as in class {}) or weaker",
                    class.name,
                    property.name,
                    property_visibility_name(parent_property.set_visibility),
                    parent.name
                ),
                Some(property.span),
            ));
        }
    }
    Ok(())
}

fn validate_method_names(class: &ClassDecl) -> Result<()> {
    let mut names = HashSet::new();
    for method in &class.methods {
        let lookup_name = method.name.to_ascii_lowercase();
        if method.is_static && lookup_name == "__invoke" {
            return Err(Diagnostic::new(
                format!("Method {}::__invoke() cannot be static", class.name),
                Some(method.span),
            ));
        }
        if !names.insert(lookup_name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare {}::{}()", class.name, lookup_name),
                Some(method.span),
            ));
        }
    }
    Ok(())
}

fn validate_property_names(class: &ClassDecl) -> Result<()> {
    let mut names = HashSet::new();
    for property in &class.properties {
        if !names.insert(property.name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare {}::${}", class.name, property.name),
                Some(property.span),
            ));
        }
    }
    for property in &class.static_properties {
        if !names.insert(property.name.clone()) {
            return Err(Diagnostic::new(
                format!("Cannot redeclare {}::${}", class.name, property.name),
                Some(property.span),
            ));
        }
    }
    Ok(())
}

fn validate_class_constant_names(class: &ClassDecl) -> Result<()> {
    let mut names = HashSet::new();
    for constant in &class.constants {
        if !names.insert(constant.name.clone()) {
            return Err(Diagnostic::new(
                format!(
                    "Cannot redefine class constant {}::{}",
                    class.name, constant.name
                ),
                Some(constant.span),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct LabelInfo {
    control_path: Vec<usize>,
    finally_path: Vec<usize>,
}

fn validate_goto_labels(statements: &[Statement]) -> Result<()> {
    let mut labels = HashMap::new();
    let mut control_path = Vec::new();
    let mut finally_path = Vec::new();
    collect_labels(
        statements,
        &mut labels,
        &mut control_path,
        &mut finally_path,
    )?;
    validate_gotos(statements, &labels, &mut control_path, &mut finally_path)
}

#[derive(Debug, Clone, Copy)]
enum ControlTransfer {
    Break,
    Continue,
}

impl ControlTransfer {
    fn keyword(self) -> &'static str {
        match self {
            Self::Break => "break",
            Self::Continue => "continue",
        }
    }
}

fn validate_control_transfers_in_statements(
    statements: &[Statement],
    control_depth: usize,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Assign { value, .. }
            | Statement::AssignRef { source: value, .. }
            | Statement::Print {
                expression: value, ..
            }
            | Statement::Expression {
                expression: value, ..
            }
            | Statement::Return {
                value: Some(value), ..
            }
            | Statement::Throw { value, .. } => {
                validate_control_transfers_in_expr(value)?;
            }
            Statement::ArrayAssign { target, value, .. } => {
                validate_control_transfers_in_array_dim_target(target)?;
                validate_control_transfers_in_expr(value)?;
            }
            Statement::ArrayAssignRef { target, source, .. } => {
                validate_control_transfers_in_array_dim_target(target)?;
                validate_control_transfers_in_expr(source)?;
            }
            Statement::Call { arguments, .. }
            | Statement::Echo {
                expressions: arguments,
                ..
            } => {
                validate_control_transfers_in_exprs(arguments)?;
            }
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    validate_control_transfers_in_expr(&declaration.value)?;
                }
            }
            Statement::Static { declarations, .. } => {
                for declaration in declarations {
                    if let Some(value) = &declaration.value {
                        validate_control_transfers_in_expr(value)?;
                    }
                }
            }
            Statement::Block { statements, .. } => {
                validate_control_transfers_in_statements(statements, control_depth)?;
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                validate_control_transfers_in_expr(condition)?;
                validate_control_transfers_in_statements(then_body, control_depth)?;
                validate_control_transfers_in_statements(else_body, control_depth)?;
            }
            Statement::While {
                condition, body, ..
            } => {
                validate_control_transfers_in_expr(condition)?;
                validate_control_transfers_in_statements(body, control_depth + 1)?;
            }
            Statement::DoWhile {
                body, condition, ..
            } => {
                validate_control_transfers_in_statements(body, control_depth + 1)?;
                validate_control_transfers_in_expr(condition)?;
            }
            Statement::For {
                initializers,
                condition,
                updates,
                body,
                ..
            } => {
                validate_control_transfers_in_statements(initializers, control_depth)?;
                if let Some(condition) = condition {
                    validate_control_transfers_in_expr(condition)?;
                }
                validate_control_transfers_in_statements(updates, control_depth)?;
                validate_control_transfers_in_statements(body, control_depth + 1)?;
            }
            Statement::Foreach {
                iterable,
                key,
                value,
                body,
                ..
            } => {
                validate_control_transfers_in_expr(iterable)?;
                if let Some(key) = key {
                    validate_control_transfers_in_assignment_target(key)?;
                }
                validate_control_transfers_in_assignment_target(value)?;
                validate_control_transfers_in_statements(body, control_depth + 1)?;
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                validate_control_transfers_in_expr(expression)?;
                for case in cases {
                    if let Some(condition) = &case.condition {
                        validate_control_transfers_in_expr(condition)?;
                    }
                    validate_control_transfers_in_statements(&case.body, control_depth + 1)?;
                }
            }
            Statement::Break { level, span } => validate_control_transfer_target(
                ControlTransfer::Break,
                *level,
                *span,
                control_depth,
            )?,
            Statement::Continue { level, span } => validate_control_transfer_target(
                ControlTransfer::Continue,
                *level,
                *span,
                control_depth,
            )?,
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_control_transfers_in_statements(body, control_depth)?;
                for catch in catches {
                    validate_control_transfers_in_statements(&catch.body, control_depth)?;
                }
                validate_control_transfers_in_statements(finally_body, control_depth)?;
            }
            Statement::Increment { target, .. } => {
                validate_control_transfers_in_inc_dec_target(target)?;
            }
            Statement::Unset { targets, .. } => {
                for target in targets {
                    validate_control_transfers_in_unset_target(target)?;
                }
            }
            Statement::Return { value: None, .. }
            | Statement::Empty { .. }
            | Statement::ClassDeclaration { .. }
            | Statement::FunctionDeclaration { .. }
            | Statement::Global { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => {}
        }
    }
    Ok(())
}

fn validate_control_transfer_target(
    transfer: ControlTransfer,
    level: usize,
    span: SourceSpan,
    control_depth: usize,
) -> Result<()> {
    let keyword = transfer.keyword();
    if level == 0 {
        return Err(Diagnostic::new(
            format!("'{keyword}' operator accepts only positive integers"),
            Some(span),
        ));
    }
    if control_depth == 0 {
        return Err(Diagnostic::new(
            format!("'{keyword}' not in the 'loop' or 'switch' context"),
            Some(span),
        ));
    }
    if level > control_depth {
        let suffix = if level == 1 { "level" } else { "levels" };
        return Err(Diagnostic::new(
            format!("Cannot '{keyword}' {level} {suffix}"),
            Some(span),
        ));
    }
    Ok(())
}

fn validate_control_transfers_in_exprs(expressions: &[Expr]) -> Result<()> {
    for expression in expressions {
        validate_control_transfers_in_expr(expression)?;
    }
    Ok(())
}

fn validate_control_transfers_in_expr(expr: &Expr) -> Result<()> {
    match expr {
        Expr::AnonymousFunction(function) => {
            validate_control_transfers_in_statements(&function.body, 0)?;
        }
        Expr::DynamicVariable { name, .. } => validate_control_transfers_in_expr(name)?,
        Expr::IncDec { target, .. } => validate_control_transfers_in_inc_dec_target(target)?,
        Expr::Assign { target, value, .. } => {
            validate_control_transfers_in_assignment_target(target)?;
            validate_control_transfers_in_expr(value)?;
        }
        Expr::AssignRef { target, source, .. } => {
            validate_control_transfers_in_assignment_target(target)?;
            validate_control_transfers_in_expr(source)?;
        }
        Expr::Call { arguments, .. } | Expr::NewObject { arguments, .. } => {
            validate_control_transfers_in_exprs(arguments)?;
        }
        Expr::FirstClassCallable { callable, .. } => {
            validate_control_transfers_in_expr(callable)?;
        }
        Expr::DynamicCall {
            callee, arguments, ..
        } => {
            validate_control_transfers_in_expr(callee)?;
            validate_control_transfers_in_exprs(arguments)?;
        }
        Expr::MethodCall {
            receiver,
            arguments,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_exprs(arguments)?;
        }
        Expr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_expr(name)?;
            validate_control_transfers_in_exprs(arguments)?;
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            validate_control_transfers_in_expr(class_name)?;
            validate_control_transfers_in_exprs(arguments)?;
        }
        Expr::InstanceOf { expr, target, .. } => {
            validate_control_transfers_in_expr(expr)?;
            if let InstanceOfTarget::Expr(target) = target {
                validate_control_transfers_in_expr(target)?;
            }
        }
        Expr::Clone { expr, .. }
        | Expr::PropertyFetch { receiver: expr, .. }
        | Expr::NullsafePropertyFetch { receiver: expr, .. }
        | Expr::DynamicClassNameFetch { receiver: expr, .. }
        | Expr::Empty { target: expr, .. }
        | Expr::Print {
            expression: expr, ..
        }
        | Expr::Include { path: expr, .. }
        | Expr::Throw { value: expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Grouped { expr, .. }
        | Expr::PipeValue { expr, .. } => validate_control_transfers_in_expr(expr)?,
        Expr::Yield { key, value, .. } => {
            if let Some(key) = key {
                validate_control_transfers_in_expr(key)?;
            }
            if let Some(value) = value {
                validate_control_transfers_in_expr(value)?;
            }
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_expr(name)?;
        }
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    validate_control_transfers_in_expr(key)?;
                }
                match &element.value {
                    ArrayElementValue::Hole(_) => {}
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        validate_control_transfers_in_expr(value)?;
                    }
                    ArrayElementValue::Reference(target) => {
                        validate_control_transfers_in_reference_target(target)?;
                    }
                }
            }
        }
        Expr::List(list) => validate_control_transfers_in_list_expr(list)?,
        Expr::ArrayAccess { array, index, .. } => {
            validate_control_transfers_in_expr(array)?;
            if let Some(index) = index {
                validate_control_transfers_in_expr(index)?;
            }
        }
        Expr::Isset { targets, .. } => validate_control_transfers_in_exprs(targets)?,
        Expr::Binary { left, right, .. } => {
            validate_control_transfers_in_expr(left)?;
            validate_control_transfers_in_expr(right)?;
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            validate_control_transfers_in_expr(condition)?;
            if let Some(if_true) = if_true {
                validate_control_transfers_in_expr(if_true)?;
            }
            validate_control_transfers_in_expr(if_false)?;
        }
        Expr::Match { subject, arms, .. } => {
            validate_control_transfers_in_expr(subject)?;
            for arm in arms {
                validate_control_transfers_in_exprs(&arm.conditions)?;
                validate_control_transfers_in_expr(&arm.value)?;
            }
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. } => {}
    }
    Ok(())
}

fn validate_control_transfers_in_assignment_target(target: &AssignmentTarget) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. } | AssignmentTarget::StaticProperty { .. } => {}
        AssignmentTarget::DynamicVariable { name, .. } => {
            validate_control_transfers_in_expr(name)?;
        }
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            validate_control_transfers_in_expr(name)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::ArrayDim(target) => {
            validate_control_transfers_in_array_dim_target(target)?;
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::Property { receiver, .. } => {
            validate_control_transfers_in_expr(receiver)?;
        }
        AssignmentTarget::DynamicProperty { receiver, name, .. } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_expr(name)?;
        }
        AssignmentTarget::List(list) => {
            validate_control_transfers_in_list_assignment_target(list)?;
        }
    }
    Ok(())
}

fn validate_control_transfers_in_list_assignment_target(
    target: &ListAssignmentTarget,
) -> Result<()> {
    for element in &target.elements {
        if let Some(key) = &element.key {
            validate_control_transfers_in_expr(key)?;
        }
        match &element.target {
            ListAssignmentElementTarget::Value(target) => {
                validate_control_transfers_in_assignment_target(target)?;
            }
            ListAssignmentElementTarget::Reference(target) => {
                validate_control_transfers_in_reference_target(target)?;
            }
        }
    }
    Ok(())
}

fn validate_control_transfers_in_list_expr(list: &ListExpr) -> Result<()> {
    for element in &list.elements {
        if let Some(key) = &element.key {
            validate_control_transfers_in_expr(key)?;
        }
        match &element.target {
            Some(ListExprElementTarget::Value(value)) => {
                validate_control_transfers_in_expr(value)?;
            }
            Some(ListExprElementTarget::Reference(target)) => {
                validate_control_transfers_in_reference_target(target)?;
            }
            None => {}
        }
    }
    Ok(())
}

fn validate_control_transfers_in_reference_target(target: &ReferenceTarget) -> Result<()> {
    match target {
        ReferenceTarget::Variable { .. } => {}
        ReferenceTarget::ArrayDim(target) => {
            validate_control_transfers_in_array_dim_target(target)?;
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        ReferenceTarget::Property { receiver, .. } => {
            validate_control_transfers_in_expr(receiver)?;
        }
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_expr(name)?;
        }
    }
    Ok(())
}

fn validate_control_transfers_in_array_dim_target(target: &ArrayDimTarget) -> Result<()> {
    validate_control_transfers_in_optional_exprs(&target.dimensions)
}

fn validate_control_transfers_in_inc_dec_target(target: &IncDecTarget) -> Result<()> {
    match target {
        IncDecTarget::Variable { .. } | IncDecTarget::StaticProperty { .. } => {}
        IncDecTarget::DynamicVariable { name, .. } => {
            validate_control_transfers_in_expr(name)?;
        }
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            validate_control_transfers_in_expr(name)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        IncDecTarget::ArrayDim(target) => {
            validate_control_transfers_in_array_dim_target(target)?;
        }
        IncDecTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        IncDecTarget::Property { receiver, .. } => {
            validate_control_transfers_in_expr(receiver)?;
        }
    }
    Ok(())
}

fn validate_control_transfers_in_unset_target(target: &UnsetTarget) -> Result<()> {
    match target {
        UnsetTarget::Variable { .. } => {}
        UnsetTarget::DynamicVariable { name, .. } => {
            validate_control_transfers_in_expr(name)?;
        }
        UnsetTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            validate_control_transfers_in_expr(name)?;
            validate_control_transfers_in_exprs(dimensions)?;
        }
        UnsetTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_exprs(dimensions)?;
        }
        UnsetTarget::Property { receiver, .. } => {
            validate_control_transfers_in_expr(receiver)?;
        }
        UnsetTarget::ArrayDim(target) => {
            validate_control_transfers_in_array_dim_target(target)?;
        }
    }
    Ok(())
}

fn validate_control_transfers_in_optional_exprs(expressions: &[Option<Expr>]) -> Result<()> {
    for expression in expressions.iter().flatten() {
        validate_control_transfers_in_expr(expression)?;
    }
    Ok(())
}

fn reference_source_is_variable(source: &Expr, variable: &str) -> bool {
    match source {
        Expr::Variable(name, _) => name == variable,
        Expr::Grouped { expr, .. } => reference_source_is_variable(expr, variable),
        _ => false,
    }
}

fn validate_recursive_reference_assignment_value(
    target: &AssignmentTarget,
    value: &Expr,
) -> Result<()> {
    let variable = match target {
        AssignmentTarget::Variable { name, .. } => name,
        AssignmentTarget::DynamicVariable { .. } => return Ok(()),
        AssignmentTarget::DynamicArrayDim { .. } => return Ok(()),
        AssignmentTarget::ArrayDim(target) => &target.array,
        AssignmentTarget::PropertyArrayDim { .. } => return Ok(()),
        AssignmentTarget::StaticPropertyArrayDim { .. } => return Ok(()),
        AssignmentTarget::Property { .. } => return Ok(()),
        AssignmentTarget::DynamicProperty { .. } => return Ok(()),
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
        Expr::List(list) => list.elements.iter().find_map(|element| {
            element
                .key
                .as_ref()
                .and_then(|key| expr_array_literal_reference_to_variable(key, variable))
                .or_else(|| match &element.target {
                    Some(ListExprElementTarget::Value(value)) => {
                        expr_array_literal_reference_to_variable(value, variable)
                    }
                    Some(ListExprElementTarget::Reference(target)) => {
                        reference_target_reference_to_variable(target, variable)
                    }
                    None => None,
                })
        }),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => expr_array_literal_reference_to_variable(condition, variable)
            .or_else(|| {
                if_true
                    .as_deref()
                    .and_then(|if_true| expr_array_literal_reference_to_variable(if_true, variable))
            })
            .or_else(|| expr_array_literal_reference_to_variable(if_false, variable)),
        Expr::Match { subject, arms, .. } => {
            expr_array_literal_reference_to_variable(subject, variable).or_else(|| {
                arms.iter().find_map(|arm| {
                    arm.conditions
                        .iter()
                        .find_map(|condition| {
                            expr_array_literal_reference_to_variable(condition, variable)
                        })
                        .or_else(|| expr_array_literal_reference_to_variable(&arm.value, variable))
                })
            })
        }
        Expr::Assign { value, .. }
        | Expr::AssignRef { source: value, .. }
        | Expr::Print {
            expression: value, ..
        }
        | Expr::Include { path: value, .. }
        | Expr::Throw { value, .. }
        | Expr::Clone { expr: value, .. }
        | Expr::FirstClassCallable {
            callable: value, ..
        }
        | Expr::Grouped { expr: value, .. }
        | Expr::PipeValue { expr: value, .. } => {
            expr_array_literal_reference_to_variable(value, variable)
        }
        Expr::Yield { key, value, .. } => key
            .as_deref()
            .and_then(|key| expr_array_literal_reference_to_variable(key, variable))
            .or_else(|| {
                value
                    .as_deref()
                    .and_then(|value| expr_array_literal_reference_to_variable(value, variable))
            }),
        Expr::DynamicVariable { name, .. } => {
            expr_array_literal_reference_to_variable(name, variable)
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::AnonymousFunction(_)
        | Expr::IncDec { .. }
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::DynamicNewObject { .. }
        | Expr::PropertyFetch { .. }
        | Expr::NullsafePropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. }
        | Expr::DynamicClassNameFetch { .. }
        | Expr::InstanceOf { .. }
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
            ArrayElementValue::Hole(_) => None,
            ArrayElementValue::Reference(target) => {
                reference_target_reference_to_variable(target, variable)
            }
            ArrayElementValue::Value(value) => {
                expr_array_literal_reference_to_variable(value, variable)
            }
            ArrayElementValue::Unpack(value) => {
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
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_array_literal_reference_to_variable(receiver, variable).or_else(|| {
            dimensions
                .iter()
                .flatten()
                .find_map(|dimension| expr_array_literal_reference_to_variable(dimension, variable))
        }),
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            expr_array_literal_reference_to_variable(receiver, variable)
                .or_else(|| expr_array_literal_reference_to_variable(name, variable))
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
        Expr::Variable(_, _)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::PropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. } => Ok(()),
        Expr::Grouped { expr, .. } => validate_reference_source_expr(expr),
        Expr::ArrayAccess { .. } => validate_array_reference_lvalue_expr(
            source,
            "temporary array offset references are unsupported",
        ),
        _ => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(source.span()),
        )),
    }
}

fn validate_array_reference_lvalue_expr(expr: &Expr, temporary_message: &str) -> Result<()> {
    match expr {
        Expr::Variable(_, _) | Expr::PropertyFetch { .. } => Ok(()),
        Expr::Grouped { expr, .. } => validate_array_reference_lvalue_expr(expr, temporary_message),
        Expr::ArrayAccess { array, index, span } => {
            if let Some(index) = index {
                reject_append_array_read(index)?;
            }
            match array.as_ref() {
                Expr::Variable(_, _) | Expr::PropertyFetch { .. } => Ok(()),
                Expr::Grouped { expr, .. } => match expr.as_ref() {
                    Expr::Variable(_, _) | Expr::PropertyFetch { .. } => Ok(()),
                    Expr::ArrayAccess { .. } => {
                        validate_array_reference_lvalue_expr(expr.as_ref(), temporary_message)
                    }
                    _ => Err(Diagnostic::new(temporary_message, Some(*span))),
                },
                Expr::ArrayAccess { .. } => {
                    validate_array_reference_lvalue_expr(array.as_ref(), temporary_message)
                }
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

fn validate_void_returns(functions: &[FunctionDecl]) -> Result<()> {
    for function in functions {
        if matches!(&function.return_type, Some(TypeHint::Void)) {
            validate_void_returns_in_statements(&function.body)?;
        }
    }
    Ok(())
}

fn validate_void_returns_in_statements(statements: &[Statement]) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Return {
                value: Some(_),
                span,
            } => {
                return Err(Diagnostic::new(
                    "A void function must not return a value",
                    Some(*span),
                ));
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_void_returns_in_statements(then_body)?;
                validate_void_returns_in_statements(else_body)?;
            }
            Statement::Block { statements, .. } => {
                validate_void_returns_in_statements(statements)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_void_returns_in_statements(body)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_void_returns_in_statements(initializers)?;
                validate_void_returns_in_statements(updates)?;
                validate_void_returns_in_statements(body)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_void_returns_in_statements(body)?;
                for catch in catches {
                    validate_void_returns_in_statements(&catch.body)?;
                }
                validate_void_returns_in_statements(finally_body)?;
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_void_returns_in_statements(&case.body)?;
                }
            }
            _ => {}
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
            }
            | Statement::Throw { value, .. } => {
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
            Statement::Static { declarations, .. } => {
                for declaration in declarations {
                    if let Some(value) = &declaration.value {
                        validate_anonymous_functions_in_expr(value, functions)?;
                    }
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
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_anonymous_functions_in_statements(body, functions)?;
                for catch in catches {
                    validate_anonymous_functions_in_statements(&catch.body, functions)?;
                }
                validate_anonymous_functions_in_statements(finally_body, functions)?;
            }
            Statement::Return { value: None, .. }
            | Statement::Empty { .. }
            | Statement::ClassDeclaration { .. }
            | Statement::FunctionDeclaration { .. }
            | Statement::Unset { .. }
            | Statement::Global { .. }
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Label { .. }
            | Statement::Goto { .. }
            | Statement::InlineHtml { .. } => {}
            Statement::Increment { target, .. } => {
                validate_anonymous_functions_in_inc_dec_target(target, functions)?;
            }
        }
    }
    Ok(())
}

fn validate_anonymous_functions_in_inc_dec_target(
    target: &IncDecTarget,
    functions: &[FunctionDecl],
) -> Result<()> {
    match target {
        IncDecTarget::Variable { .. } => Ok(()),
        IncDecTarget::DynamicVariable { name, .. } => {
            validate_anonymous_functions_in_expr(name, functions)
        }
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            validate_anonymous_functions_in_expr(name, functions)?;
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    validate_anonymous_functions_in_expr(dimension, functions)?;
                }
            }
            Ok(())
        }
        IncDecTarget::ArrayDim(target) => {
            for dimension in &target.dimensions {
                if let Some(dimension) = dimension {
                    validate_anonymous_functions_in_expr(dimension, functions)?;
                }
            }
            Ok(())
        }
        IncDecTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    validate_anonymous_functions_in_expr(dimension, functions)?;
                }
            }
            Ok(())
        }
        IncDecTarget::Property { receiver, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)
        }
        IncDecTarget::StaticProperty { .. } => Ok(()),
    }
}

fn validate_anonymous_functions_in_expr(expr: &Expr, functions: &[FunctionDecl]) -> Result<()> {
    match expr {
        Expr::AnonymousFunction(function) => {
            if function.return_by_ref {
                validate_by_reference_returns_in_statements(&function.body, "{closure}")?;
            }
            if matches!(&function.return_type, Some(TypeHint::Void)) {
                validate_void_returns_in_statements(&function.body)?;
            }
            validate_anonymous_functions_in_statements(&function.body, functions)?;
            validate_reference_assignment_sources(&function.body, functions)?;
            validate_goto_labels(&function.body)?;
        }
        Expr::IncDec { target, .. } => {
            validate_anonymous_functions_in_inc_dec_target(target, functions)?;
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
        | Expr::DynamicMethodCall { arguments, .. }
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
            if let Expr::DynamicMethodCall { receiver, name, .. } = expr {
                validate_anonymous_functions_in_expr(receiver, functions)?;
                validate_anonymous_functions_in_expr(name, functions)?;
            }
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => {
            validate_anonymous_functions_in_expr(class_name, functions)?;
            for argument in arguments {
                validate_anonymous_functions_in_expr(argument, functions)?;
            }
        }
        Expr::FirstClassCallable { callable, .. } => {
            validate_anonymous_functions_in_expr(callable, functions)?;
        }
        Expr::PropertyFetch { receiver, .. } | Expr::NullsafePropertyFetch { receiver, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
            validate_anonymous_functions_in_expr(name, functions)?;
        }
        Expr::DynamicClassNameFetch { receiver, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
        }
        Expr::InstanceOf { expr, target, .. } => {
            validate_anonymous_functions_in_expr(expr, functions)?;
            if let InstanceOfTarget::Expr(target) = target {
                validate_anonymous_functions_in_expr(target, functions)?;
            }
        }
        Expr::Clone { expr, .. } => {
            validate_anonymous_functions_in_expr(expr, functions)?;
        }
        Expr::StaticPropertyFetch { .. } | Expr::ClassConstantFetch { .. } => {}
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    validate_anonymous_functions_in_expr(key, functions)?;
                }
                match &element.value {
                    ArrayElementValue::Hole(_) => {}
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        validate_anonymous_functions_in_expr(value, functions)?;
                    }
                    ArrayElementValue::Reference(_) => {}
                }
            }
        }
        Expr::List(list) => {
            for element in &list.elements {
                if let Some(key) = &element.key {
                    validate_anonymous_functions_in_expr(key, functions)?;
                }
                if let Some(ListExprElementTarget::Value(value)) = &element.target {
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
        Expr::Print {
            expression: expr, ..
        }
        | Expr::DynamicVariable { name: expr, .. }
        | Expr::Include { path: expr, .. }
        | Expr::Throw { value: expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Grouped { expr, .. }
        | Expr::PipeValue { expr, .. } => validate_anonymous_functions_in_expr(expr, functions)?,
        Expr::Yield { key, value, .. } => {
            if let Some(key) = key {
                validate_anonymous_functions_in_expr(key, functions)?;
            }
            if let Some(value) = value {
                validate_anonymous_functions_in_expr(value, functions)?;
            }
        }
        Expr::Binary { left, right, .. } => {
            validate_anonymous_functions_in_expr(left, functions)?;
            validate_anonymous_functions_in_expr(right, functions)?;
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            validate_anonymous_functions_in_expr(condition, functions)?;
            if let Some(if_true) = if_true {
                validate_anonymous_functions_in_expr(if_true, functions)?;
            }
            validate_anonymous_functions_in_expr(if_false, functions)?;
        }
        Expr::Match { subject, arms, .. } => {
            validate_anonymous_functions_in_expr(subject, functions)?;
            for arm in arms {
                for condition in &arm.conditions {
                    validate_anonymous_functions_in_expr(condition, functions)?;
                }
                validate_anonymous_functions_in_expr(&arm.value, functions)?;
            }
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
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
            Statement::Return { value, .. } => {
                if let Some(value) = value {
                    validate_by_reference_return_value(value, function_name)?;
                }
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
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_by_reference_returns_in_statements(body, function_name)?;
                for catch in catches {
                    validate_by_reference_returns_in_statements(&catch.body, function_name)?;
                }
                validate_by_reference_returns_in_statements(finally_body, function_name)?;
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
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_reference_assignment_sources(body, functions)?;
                for catch in catches {
                    validate_reference_assignment_sources(&catch.body, functions)?;
                }
                validate_reference_assignment_sources(finally_body, functions)?;
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
        Expr::Print { expression, .. } => {
            validate_reference_assignment_source_expr(expression, functions)
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            validate_reference_assignment_source_expr(condition, functions)?;
            if let Some(if_true) = if_true {
                validate_reference_assignment_source_expr(if_true, functions)?;
            }
            validate_reference_assignment_source_expr(if_false, functions)
        }
        Expr::Call { .. } => Ok(()),
        _ => Ok(()),
    }
}

fn validate_by_reference_return_value(value: &Expr, function_name: &str) -> Result<()> {
    match value {
        Expr::Grouped { expr, .. } => validate_by_reference_return_value(expr, function_name),
        Expr::Call { name, span, .. } if name.eq_ignore_ascii_case(function_name) => {
            Err(Diagnostic::new(
                "recursive by-reference returns are unsupported",
                Some(*span),
            ))
        }
        _ => Ok(()),
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
            | "strcasecmp"
            | "strncasecmp"
            | "strncmp"
            | "strnatcasecmp"
            | "strnatcmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "strpos"
            | "stripos"
            | "strrpos"
            | "strripos"
            | "strstr"
            | "stristr"
            | "substr_compare"
            | "substr_count"
            | "strspn"
            | "strcspn"
            | "strcoll"
            | "strpbrk"
            | "str_pad"
            | "str_repeat"
            | "str_split"
            | "strrev"
            | "stripcslashes"
            | "stripslashes"
            | "strtolower"
            | "strtoupper"
            | "str_increment"
            | "ucfirst"
            | "lcfirst"
            | "chop"
            | "touch"
            | "trait_exists"
            | "trim"
            | "ltrim"
            | "rtrim"
            | "quotemeta"
            | "quoted_printable_encode"
            | "chunk_split"
            | "convert_uudecode"
            | "convert_uuencode"
            | "base64_decode"
            | "base64_encode"
            | "closedir"
            | "compact"
            | "strip_tags"
            | "pack"
            | "unpack"
            | "count_chars"
            | "crc32"
            | "md5"
            | "md5_file"
            | "sha1"
            | "sha1_file"
            | "similar_text"
            | "substr"
            | "dirname"
            | "bin2hex"
            | "hex2bin"
            | "parse_ini_string"
            | "quoted_printable_decode"
            | "soundex"
            | "metaphone"
            | "utf8_decode"
            | "utf8_encode"
            | "sprintf"
            | "printf"
            | "fprintf"
            | "sscanf"
            | "vsprintf"
            | "vprintf"
            | "vfprintf"
            | "json_encode"
            | "gc_collect_cycles"
            | "ceil"
            | "floor"
            | "abs"
            | "sqrt"
            | "pow"
            | "fdiv"
            | "fclose"
            | "feof"
            | "fflush"
            | "fgetc"
            | "fgetcsv"
            | "fgets"
            | "file"
            | "file_exists"
            | "file_get_contents"
            | "file_put_contents"
            | "fileatime"
            | "filectime"
            | "filegroup"
            | "fileinode"
            | "filemtime"
            | "fileowner"
            | "fileperms"
            | "filesize"
            | "filetype"
            | "fopen"
            | "opendir"
            | "fpassthru"
            | "fputcsv"
            | "fputs"
            | "fread"
            | "fseek"
            | "fstat"
            | "ftell"
            | "ftruncate"
            | "fwrite"
            | "pathinfo"
            | "parse_str"
            | "readdir"
            | "readfile"
            | "stream_get_meta_data"
            | "rewinddir"
            | "stream_context_create"
            | "stream_copy_to_stream"
            | "stream_filter_append"
            | "stream_filter_prepend"
            | "stream_get_contents"
            | "stream_get_line"
            | "tmpfile"
            | "get_cfg_var"
            | "get_loaded_extensions"
            | "get_html_translation_table"
            | "highlight_file"
            | "highlight_string"
            | "ini_get"
            | "ini_parse_quantity"
            | "intdiv"
            | "assert"
            | "basename"
            | "chdir"
            | "chmod"
            | "clearstatcache"
            | "pi"
            | "getrandmax"
            | "getmypid"
            | "php_ini_scanned_files"
            | "php_sapi_name"
            | "php_uname"
            | "phpversion"
            | "preg_match"
            | "print_r"
            | "realpath"
            | "rewind"
            | "set_time_limit"
            | "zend_version"
            | "var_export"
            | "bindec"
            | "hexdec"
            | "implode"
            | "in_array"
            | "html_entity_decode"
            | "htmlentities"
            | "ob_get_contents"
            | "octdec"
            | "boolval"
            | "doubleval"
            | "floatval"
            | "intval"
            | "chr"
            | "ord"
            | "rand"
            | "range"
            | "arsort"
            | "asort"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "error_reporting"
            | "explode"
            | "func_get_arg"
            | "func_get_args"
            | "func_num_args"
            | "gettype"
            | "is_array"
            | "is_null"
            | "is_bool"
            | "is_object"
            | "is_countable"
            | "is_resource"
            | "is_dir"
            | "is_executable"
            | "is_file"
            | "is_link"
            | "is_readable"
            | "is_writable"
            | "is_writeable"
            | "is_int"
            | "is_integer"
            | "is_iterable"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_scalar"
            | "is_finite"
            | "is_infinite"
            | "is_nan"
            | "join"
            | "levenshtein"
            | "localeconv"
            | "nl_langinfo"
            | "lstat"
            | "define"
            | "constant"
            | "defined"
            | "extension_loaded"
            | "extract"
            | "function_exists"
            | "getcwd"
            | "get_called_class"
            | "get_class"
            | "get_parent_class"
            | "interface_exists"
            | "stat"
            | "isset"
            | "empty"
            | "count"
            | "sizeof"
            | "array_all"
            | "array_any"
            | "array_change_key_case"
            | "array_chunk"
            | "array_column"
            | "array_combine"
            | "array_count_values"
            | "array_diff"
            | "array_diff_assoc"
            | "array_diff_key"
            | "array_diff_uassoc"
            | "array_diff_ukey"
            | "array_fill"
            | "array_fill_keys"
            | "array_filter"
            | "array_find"
            | "array_find_key"
            | "array_first"
            | "array_flip"
            | "array_intersect"
            | "array_intersect_assoc"
            | "array_intersect_key"
            | "array_intersect_uassoc"
            | "array_intersect_ukey"
            | "array_is_list"
            | "array_key_exists"
            | "array_key_first"
            | "array_key_last"
            | "array_keys"
            | "array_last"
            | "array_map"
            | "array_merge"
            | "array_merge_recursive"
            | "array_multisort"
            | "array_pad"
            | "array_pop"
            | "array_product"
            | "array_push"
            | "array_reduce"
            | "array_replace"
            | "array_replace_recursive"
            | "array_reverse"
            | "array_search"
            | "array_shift"
            | "array_slice"
            | "array_splice"
            | "array_sum"
            | "array_udiff"
            | "array_udiff_assoc"
            | "array_udiff_uassoc"
            | "array_uintersect"
            | "array_uintersect_assoc"
            | "array_uintersect_uassoc"
            | "array_unshift"
            | "array_unique"
            | "array_values"
            | "array_walk"
            | "array_walk_recursive"
            | "shuffle"
            | "sort"
            | "rsort"
            | "uasort"
            | "uksort"
            | "usort"
            | "current"
            | "end"
            | "key"
            | "next"
            | "nl2br"
            | "prev"
            | "reset"
            | "mkdir"
            | "rmdir"
            | "scandir"
            | "setlocale"
            | "str_shuffle"
            | "str_ireplace"
            | "str_replace"
            | "strrchr"
            | "strtok"
            | "str_getcsv"
            | "strtr"
            | "str_word_count"
            | "unlink"
            | "call_user_func"
            | "call_user_func_array"
            | "class_alias"
            | "class_exists"
            | "debug_zval_dump"
            | "is_callable"
            | "is_subclass_of"
            | "method_exists"
            | "property_exists"
            | "spl_object_hash"
            | "spl_object_id"
    )
}

fn is_modeled_global_constant_name(name: &str) -> bool {
    matches!(
        name,
        "E_ERROR"
            | "E_WARNING"
            | "E_PARSE"
            | "E_NOTICE"
            | "E_CORE_ERROR"
            | "E_CORE_WARNING"
            | "E_COMPILE_ERROR"
            | "E_COMPILE_WARNING"
            | "E_USER_ERROR"
            | "E_USER_WARNING"
            | "E_USER_NOTICE"
            | "E_STRICT"
            | "E_RECOVERABLE_ERROR"
            | "E_DEPRECATED"
            | "E_USER_DEPRECATED"
            | "E_ALL"
            | "INI_USER"
            | "INI_PERDIR"
            | "INI_SYSTEM"
            | "INI_ALL"
            | "EXTR_OVERWRITE"
            | "EXTR_SKIP"
            | "EXTR_PREFIX_SAME"
            | "EXTR_PREFIX_ALL"
            | "EXTR_PREFIX_INVALID"
            | "EXTR_PREFIX_IF_EXISTS"
            | "EXTR_IF_EXISTS"
            | "EXTR_REFS"
            | "CASE_LOWER"
            | "CASE_UPPER"
            | "SORT_REGULAR"
            | "SORT_NUMERIC"
            | "SORT_STRING"
            | "SORT_DESC"
            | "SORT_ASC"
            | "SORT_LOCALE_STRING"
            | "SORT_NATURAL"
            | "SORT_FLAG_CASE"
            | "ARRAY_FILTER_USE_BOTH"
            | "ARRAY_FILTER_USE_KEY"
            | "PATHINFO_DIRNAME"
            | "PATHINFO_BASENAME"
            | "PATHINFO_EXTENSION"
            | "PATHINFO_FILENAME"
            | "PATHINFO_ALL"
            | "HTML_SPECIALCHARS"
            | "HTML_ENTITIES"
            | "ENT_NOQUOTES"
            | "ENT_COMPAT"
            | "ENT_QUOTES"
            | "ENT_IGNORE"
            | "ENT_SUBSTITUTE"
            | "ENT_DISALLOWED"
            | "ENT_HTML401"
            | "ENT_XML1"
            | "ENT_XHTML"
            | "ENT_HTML5"
            | "CRYPT_SALT_LENGTH"
            | "CRYPT_STD_DES"
            | "CRYPT_EXT_DES"
            | "CRYPT_MD5"
            | "CRYPT_BLOWFISH"
            | "CRYPT_SHA256"
            | "CRYPT_SHA512"
            | "LC_ALL"
            | "LC_COLLATE"
            | "LC_CTYPE"
            | "LC_MESSAGES"
            | "LC_MONETARY"
            | "LC_NUMERIC"
            | "LC_TIME"
            | "ABDAY_1"
            | "ABDAY_2"
            | "ABDAY_3"
            | "ABDAY_4"
            | "ABDAY_5"
            | "ABDAY_6"
            | "ABDAY_7"
            | "DAY_1"
            | "DAY_2"
            | "DAY_3"
            | "DAY_4"
            | "DAY_5"
            | "DAY_6"
            | "DAY_7"
            | "ABMON_1"
            | "ABMON_2"
            | "ABMON_3"
            | "ABMON_4"
            | "ABMON_5"
            | "ABMON_6"
            | "ABMON_7"
            | "ABMON_8"
            | "ABMON_9"
            | "ABMON_10"
            | "ABMON_11"
            | "ABMON_12"
            | "MON_1"
            | "MON_2"
            | "MON_3"
            | "MON_4"
            | "MON_5"
            | "MON_6"
            | "MON_7"
            | "MON_8"
            | "MON_9"
            | "MON_10"
            | "MON_11"
            | "MON_12"
            | "RADIXCHAR"
            | "THOUSEP"
            | "YESEXPR"
            | "NOEXPR"
            | "CODESET"
            | "DATE_ATOM"
            | "DATE_COOKIE"
            | "DATE_ISO8601"
            | "DATE_ISO8601_EXPANDED"
            | "DATE_RFC822"
            | "DATE_RFC850"
            | "DATE_RFC1036"
            | "DATE_RFC1123"
            | "DATE_RFC7231"
            | "DATE_RFC2822"
            | "DATE_RFC3339"
            | "DATE_RFC3339_EXTENDED"
            | "DATE_RSS"
            | "DATE_W3C"
            | "M_E"
            | "M_LOG2E"
            | "M_LOG10E"
            | "M_LN2"
            | "M_LN10"
            | "PHP_INT_MIN"
            | "PHP_INT_MAX"
            | "PHP_INT_SIZE"
            | "PHP_MAXPATHLEN"
            | "PHP_VERSION"
            | "PHP_MAJOR_VERSION"
            | "PHP_MINOR_VERSION"
            | "PHP_RELEASE_VERSION"
            | "PHP_EXTRA_VERSION"
            | "PHP_VERSION_ID"
            | "PHP_ZTS"
            | "PHP_DEBUG"
            | "PHP_SAPI"
            | "PHP_OS"
            | "PHP_OS_FAMILY"
            | "PHP_SHLIB_SUFFIX"
            | "PHP_EOL"
            | "DIRECTORY_SEPARATOR"
            | "PATH_SEPARATOR"
            | "INF"
            | "NAN"
            | "M_PI"
            | "M_PI_2"
            | "M_PI_4"
            | "M_1_PI"
            | "M_2_PI"
            | "M_SQRTPI"
            | "M_2_SQRTPI"
            | "M_LNPI"
            | "M_EULER"
            | "M_SQRT2"
            | "M_SQRT1_2"
            | "M_SQRT3"
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
        "array_pop"
            | "array_push"
            | "array_shift"
            | "array_unshift"
            | "arsort"
            | "asort"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "rsort"
            | "shuffle"
            | "sort"
            | "uasort"
            | "uksort"
            | "usort"
    )
}

fn is_array_path_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_pop"
            | "array_push"
            | "array_shift"
            | "array_unshift"
            | "arsort"
            | "asort"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "rsort"
            | "shuffle"
            | "sort"
            | "uasort"
            | "uksort"
            | "usort"
    )
}

fn is_array_multisort_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_, _) => true,
        Expr::Array { .. } => true,
        Expr::Grouped { expr, .. } => is_array_multisort_argument(expr),
        Expr::Int(_, _) => true,
        Expr::Constant(name, _) => matches!(
            name.as_str(),
            "SORT_REGULAR"
                | "SORT_NUMERIC"
                | "SORT_STRING"
                | "SORT_DESC"
                | "SORT_ASC"
                | "SORT_LOCALE_STRING"
                | "SORT_NATURAL"
                | "SORT_FLAG_CASE"
        ),
        Expr::Binary {
            op: crate::ast::BinaryOp::BitwiseOr,
            left,
            right,
            ..
        } => is_array_multisort_argument(left) && is_array_multisort_argument(right),
        _ => false,
    }
}

fn is_direct_variable_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_, _) => true,
        Expr::Grouped { expr, .. } => is_direct_variable_argument(expr),
        _ => false,
    }
}

fn is_variable_array_access_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_, _) => true,
        Expr::ArrayAccess { array, .. } => match array.as_ref() {
            Expr::Variable(_, _) | Expr::ArrayAccess { .. } | Expr::Grouped { .. } => {
                is_variable_array_access_argument(array)
            }
            _ => false,
        },
        Expr::Grouped { expr, .. } => is_variable_array_access_argument(expr),
        _ => false,
    }
}

fn is_by_ref_temporary_array_mutation_argument(expr: &Expr) -> bool {
    match expr {
        Expr::Call { .. } | Expr::DynamicCall { .. } | Expr::MethodCall { .. } => true,
        Expr::Grouped { expr, .. } => is_by_ref_temporary_array_mutation_argument(expr),
        _ => false,
    }
}

fn validate_mutating_array_internal_call(
    name: &str,
    arguments: &[Expr],
    call_span: SourceSpan,
) -> Result<()> {
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "arsort" | "asort" | "krsort" | "ksort" | "rsort" | "sort"
    ) && arguments.len() > 2
    {
        let normalized = name.to_ascii_lowercase();
        return Err(Diagnostic::new(
            format!(
                "{normalized}() currently supports one direct variable array argument and an optional sort flag"
            ),
            Some(arguments.get(2).map_or(call_span, Expr::span)),
        ));
    }
    if (name.eq_ignore_ascii_case("natsort") || name.eq_ignore_ascii_case("natcasesort"))
        && arguments.len() > 1
    {
        let normalized = name.to_ascii_lowercase();
        return Err(Diagnostic::new(
            format!(
                "{normalized}() currently supports exactly one direct variable array argument; extra arguments are unsupported"
            ),
            Some(arguments.get(1).map_or(call_span, Expr::span)),
        ));
    }
    if name.eq_ignore_ascii_case("array_multisort") {
        if arguments.iter().all(is_array_multisort_argument) {
            return Ok(());
        }
        return Err(Diagnostic::new(
            "array_multisort() requires variable array arguments and scalar sort flags; non-variable array mutation targets are unsupported",
            Some(arguments.first().map_or(call_span, Expr::span)),
        ));
    }
    if arguments.is_empty() {
        return Ok(());
    }
    if is_array_cursor_mutation_name(name) && arguments.len() == 1 {
        return Ok(());
    }

    if !is_array_by_ref_mutation_name(name) {
        return Ok(());
    }

    if is_direct_variable_argument(&arguments[0]) {
        return Ok(());
    }
    if is_array_path_mutation_name(name) && is_variable_array_access_argument(&arguments[0]) {
        return Ok(());
    }
    if (name.eq_ignore_ascii_case("array_pop") || name.eq_ignore_ascii_case("array_shift"))
        && arguments.len() == 1
        && is_by_ref_temporary_array_mutation_argument(&arguments[0])
    {
        return Ok(());
    }
    if name.eq_ignore_ascii_case("array_splice")
        && (2..=4).contains(&arguments.len())
        && is_by_ref_temporary_array_mutation_argument(&arguments[0])
    {
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
    finally_path: &mut Vec<usize>,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_labels(then_body, labels, control_path, finally_path)?;
                collect_labels(else_body, labels, control_path, finally_path)?;
            }
            Statement::Block { statements, .. } => {
                collect_labels(statements, labels, control_path, finally_path)?;
            }
            Statement::While { body, span, .. } | Statement::DoWhile { body, span, .. } => {
                collect_control_labels(*span, body, labels, control_path, finally_path)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                span,
                ..
            } => {
                collect_labels(initializers, labels, control_path, finally_path)?;
                collect_labels(updates, labels, control_path, finally_path)?;
                collect_control_labels(*span, body, labels, control_path, finally_path)?;
            }
            Statement::Foreach { body, span, .. } => {
                collect_control_labels(*span, body, labels, control_path, finally_path)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                span,
            } => {
                collect_labels(body, labels, control_path, finally_path)?;
                for catch in catches {
                    collect_labels(&catch.body, labels, control_path, finally_path)?;
                }
                control_path.push(span.byte_start);
                finally_path.push(span.byte_start);
                collect_labels(finally_body, labels, control_path, finally_path)?;
                finally_path.pop();
                control_path.pop();
            }
            Statement::Switch { cases, span, .. } => {
                control_path.push(span.byte_start);
                for case in cases {
                    collect_labels(&case.body, labels, control_path, finally_path)?;
                }
                control_path.pop();
            }
            Statement::Label { name, span } => {
                let previous = labels.insert(
                    name.clone(),
                    LabelInfo {
                        control_path: control_path.clone(),
                        finally_path: finally_path.clone(),
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
    finally_path: &mut Vec<usize>,
) -> Result<()> {
    control_path.push(span.byte_start);
    let result = collect_labels(body, labels, control_path, finally_path);
    control_path.pop();
    result
}

fn validate_gotos(
    statements: &[Statement],
    labels: &HashMap<String, LabelInfo>,
    control_path: &mut Vec<usize>,
    finally_path: &mut Vec<usize>,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_gotos(then_body, labels, control_path, finally_path)?;
                validate_gotos(else_body, labels, control_path, finally_path)?;
            }
            Statement::Block { statements, .. } => {
                validate_gotos(statements, labels, control_path, finally_path)?;
            }
            Statement::While { body, span, .. } | Statement::DoWhile { body, span, .. } => {
                validate_control_gotos(*span, body, labels, control_path, finally_path)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                span,
                ..
            } => {
                validate_gotos(initializers, labels, control_path, finally_path)?;
                validate_gotos(updates, labels, control_path, finally_path)?;
                validate_control_gotos(*span, body, labels, control_path, finally_path)?;
            }
            Statement::Foreach { body, span, .. } => {
                validate_control_gotos(*span, body, labels, control_path, finally_path)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                span,
            } => {
                validate_gotos(body, labels, control_path, finally_path)?;
                for catch in catches {
                    validate_gotos(&catch.body, labels, control_path, finally_path)?;
                }
                control_path.push(span.byte_start);
                finally_path.push(span.byte_start);
                validate_gotos(finally_body, labels, control_path, finally_path)?;
                finally_path.pop();
                control_path.pop();
            }
            Statement::Switch { cases, span, .. } => {
                control_path.push(span.byte_start);
                for case in cases {
                    validate_gotos(&case.body, labels, control_path, finally_path)?;
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
                    if !target_control_path_is_reachable(&target.finally_path, finally_path) {
                        return Err(Diagnostic::new(
                            "jump into a finally block is disallowed",
                            Some(*span),
                        ));
                    }
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
    finally_path: &mut Vec<usize>,
) -> Result<()> {
    control_path.push(span.byte_start);
    let result = validate_gotos(body, labels, control_path, finally_path);
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

fn unset_array_dim_target_from_expr(expr: Expr) -> Result<UnsetTarget> {
    let span = expr.span();
    let mut dimensions = Vec::new();
    let mut current = expr;
    loop {
        match current {
            Expr::ArrayAccess { array, index, .. } => {
                let Some(index) = index else {
                    return Err(Diagnostic::new(
                        "append array access is unsupported in unset targets",
                        Some(span),
                    ));
                };
                dimensions.push(*index);
                current = *array;
            }
            Expr::Variable(array, variable_span) => {
                dimensions.reverse();
                return Ok(UnsetTarget::ArrayDim(ArrayDimTarget {
                    array,
                    dimensions: dimensions.into_iter().map(Some).collect(),
                    span: combine_spans(variable_span, span),
                }));
            }
            Expr::DynamicVariable {
                name,
                span: variable_span,
            } => {
                dimensions.reverse();
                return Ok(UnsetTarget::DynamicArrayDim {
                    name,
                    dimensions,
                    span: combine_spans(variable_span, span),
                });
            }
            Expr::PropertyFetch {
                receiver,
                name,
                span: property_span,
            } => {
                dimensions.reverse();
                return Ok(UnsetTarget::PropertyArrayDim {
                    receiver,
                    name,
                    dimensions,
                    span: combine_spans(property_span, span),
                });
            }
            Expr::Grouped { expr, .. } => {
                current = *expr;
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

fn inc_dec_target_from_expr(expr: Expr, op_span: SourceSpan) -> Result<IncDecTarget> {
    let target = assignment_target_from_expr(expr).map_err(|_| {
        Diagnostic::new(
            "increment/decrement expression target must be a variable, array offset, or property",
            Some(op_span),
        )
    })?;
    match target {
        AssignmentTarget::Variable { name, span } => Ok(IncDecTarget::Variable { name, span }),
        AssignmentTarget::DynamicVariable { name, span } => {
            Ok(IncDecTarget::DynamicVariable { name, span })
        }
        AssignmentTarget::ArrayDim(target) => Ok(IncDecTarget::ArrayDim(target)),
        AssignmentTarget::DynamicArrayDim {
            name,
            dimensions,
            span,
        } => Ok(IncDecTarget::DynamicArrayDim {
            name,
            dimensions,
            span,
        }),
        AssignmentTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            span,
        } => Ok(IncDecTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            span,
        }),
        AssignmentTarget::StaticPropertyArrayDim { span, .. } => Err(Diagnostic::new(
            "increment/decrement expression target must be a variable, array offset, or property",
            Some(span),
        )),
        AssignmentTarget::Property {
            receiver,
            name,
            span,
        } => Ok(IncDecTarget::Property {
            receiver,
            name,
            span,
        }),
        AssignmentTarget::StaticProperty {
            class_name,
            name,
            span,
        } => Ok(IncDecTarget::StaticProperty {
            class_name,
            name,
            span,
        }),
        _ => Err(Diagnostic::new(
            "increment/decrement expression target must be a variable, array offset, or property",
            Some(assignment_target_span(&target)),
        )),
    }
}

fn inc_dec_target_span(target: &IncDecTarget) -> SourceSpan {
    match target {
        IncDecTarget::Variable { span, .. } => *span,
        IncDecTarget::DynamicVariable { span, .. } => *span,
        IncDecTarget::DynamicArrayDim { span, .. } => *span,
        IncDecTarget::ArrayDim(target) => target.span,
        IncDecTarget::PropertyArrayDim { span, .. } => *span,
        IncDecTarget::Property { span, .. } => *span,
        IncDecTarget::StaticProperty { span, .. } => *span,
    }
}

fn reference_target_from_expr(expr: Expr) -> Result<ReferenceTarget> {
    let span = expr.span();
    match expr {
        Expr::Variable(name, span) => Ok(ReferenceTarget::Variable { name, span }),
        Expr::Grouped { expr, .. } => reference_target_from_expr(*expr),
        array_expr @ Expr::ArrayAccess { .. } => {
            reference_array_dim_target_from_expr(array_expr, span)
        }
        Expr::PropertyFetch {
            receiver,
            name,
            span,
        } => Ok(ReferenceTarget::Property {
            receiver,
            name,
            span,
        }),
        Expr::DynamicPropertyFetch {
            receiver,
            name,
            span,
        } => Ok(ReferenceTarget::DynamicProperty {
            receiver,
            name,
            span,
        }),
        other => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(other.span()),
        )),
    }
}

fn reference_array_dim_target_from_expr(
    expr: Expr,
    target_span: SourceSpan,
) -> Result<ReferenceTarget> {
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
                return Ok(ReferenceTarget::ArrayDim(ArrayDimTarget {
                    array,
                    dimensions,
                    span: target_span,
                }));
            }
            Expr::PropertyFetch {
                receiver,
                name,
                span: property_span,
            } => {
                dimensions.reverse();
                return Ok(ReferenceTarget::PropertyArrayDim {
                    receiver,
                    name,
                    dimensions,
                    span: combine_spans(property_span, target_span),
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
        Expr::DynamicVariable { name, span } => {
            Ok(AssignmentTarget::DynamicVariable { name, span })
        }
        Expr::ArrayAccess { .. } => assignment_array_dim_target_from_expr(expr),
        Expr::PropertyFetch {
            receiver,
            name,
            span,
        } => Ok(AssignmentTarget::Property {
            receiver,
            name,
            span,
        }),
        Expr::DynamicPropertyFetch {
            receiver,
            name,
            span,
        } => Ok(AssignmentTarget::DynamicProperty {
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
        Expr::ClassConstantFetch { span, .. } => Err(Diagnostic::new(
            "class constant fetch is not a writable target",
            Some(span),
        )),
        Expr::DynamicClassNameFetch { span, .. } => Err(Diagnostic::new(
            "class name fetch is not a writable target",
            Some(span),
        )),
        Expr::Array { elements, span } => Ok(AssignmentTarget::List(
            list_assignment_target_from_array_elements(elements, span)?,
        )),
        Expr::List(list) => Ok(AssignmentTarget::List(
            list_assignment_target_from_list_expr(list)?,
        )),
        Expr::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            span,
        } if name.eq_ignore_ascii_case("list") => {
            reject_named_language_construct_arguments(&argument_names, span)?;
            reject_unpacked_language_construct_arguments(&argument_unpacks, span)?;
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

fn assignment_array_dim_target_from_expr(expr: Expr) -> Result<AssignmentTarget> {
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
                return Ok(AssignmentTarget::ArrayDim(ArrayDimTarget {
                    array,
                    dimensions,
                    span: combine_spans(variable_span, span),
                }));
            }
            Expr::DynamicVariable {
                name,
                span: variable_span,
            } => {
                dimensions.reverse();
                return Ok(AssignmentTarget::DynamicArrayDim {
                    name,
                    dimensions,
                    span: combine_spans(variable_span, span),
                });
            }
            Expr::PropertyFetch {
                receiver,
                name,
                span: property_span,
            } => {
                dimensions.reverse();
                return Ok(AssignmentTarget::PropertyArrayDim {
                    receiver,
                    name,
                    dimensions,
                    span: combine_spans(property_span, span),
                });
            }
            Expr::StaticPropertyFetch {
                class_name,
                name,
                span: property_span,
            } => {
                dimensions.reverse();
                return Ok(AssignmentTarget::StaticPropertyArrayDim {
                    class_name,
                    name,
                    dimensions,
                    span: combine_spans(property_span, span),
                });
            }
            Expr::Grouped { expr, .. } => {
                current = *expr;
            }
            _ => {
                return Err(Diagnostic::new(
                    "unsupported assignment target",
                    Some(current.span()),
                ));
            }
        }
    }
}

fn assignment_target_span(target: &AssignmentTarget) -> SourceSpan {
    match target {
        AssignmentTarget::Variable { span, .. }
        | AssignmentTarget::DynamicVariable { span, .. }
        | AssignmentTarget::DynamicArrayDim { span, .. }
        | AssignmentTarget::PropertyArrayDim { span, .. }
        | AssignmentTarget::StaticPropertyArrayDim { span, .. }
        | AssignmentTarget::Property { span, .. }
        | AssignmentTarget::DynamicProperty { span, .. }
        | AssignmentTarget::StaticProperty { span, .. } => *span,
        AssignmentTarget::ArrayDim(target) => target.span,
        AssignmentTarget::List(target) => target.span,
    }
}

fn reference_target_span(target: &ReferenceTarget) -> SourceSpan {
    match target {
        ReferenceTarget::Variable { span, .. } => *span,
        ReferenceTarget::ArrayDim(target) => target.span,
        ReferenceTarget::PropertyArrayDim { span, .. } => *span,
        ReferenceTarget::Property { span, .. } => *span,
        ReferenceTarget::DynamicProperty { span, .. } => *span,
    }
}

fn list_expr_element_target_span(target: &ListExprElementTarget) -> SourceSpan {
    match target {
        ListExprElementTarget::Value(value) => value.span(),
        ListExprElementTarget::Reference(target) => reference_target_span(target),
    }
}

fn validate_foreach_by_reference_target(target: &AssignmentTarget, span: SourceSpan) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. }
        | AssignmentTarget::ArrayDim(_)
        | AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::StaticPropertyArrayDim { .. } => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(span),
        )),
        AssignmentTarget::List(_) => Err(Diagnostic::new(
            "foreach destructuring is unsupported",
            Some(assignment_target_span(target)),
        )),
        _ => Err(Diagnostic::new(
            "unsupported by-reference assignment target",
            Some(span),
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
        AssignmentTarget::DynamicVariable { .. } => Ok(()),
        AssignmentTarget::DynamicArrayDim { dimensions, .. } => {
            if dimensions.iter().any(Option::is_none) {
                return Err(Diagnostic::new(
                    "null coalescing assignment cannot use append array access",
                    Some(span),
                ));
            }
            Ok(())
        }
        AssignmentTarget::ArrayDim(target) => {
            if target.dimensions.iter().any(Option::is_none) {
                return Err(Diagnostic::new(
                    "null coalescing assignment cannot use append array access",
                    Some(span),
                ));
            }
            Ok(())
        }
        AssignmentTarget::PropertyArrayDim { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::StaticPropertyArrayDim { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::DynamicProperty { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::StaticProperty { .. } => Ok(()),
        AssignmentTarget::List(_) => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables and array/string offsets",
            Some(span),
        )),
    }
}

fn validate_expression_assignment_target(
    op: AssignmentOp,
    target: &AssignmentTarget,
    span: SourceSpan,
) -> Result<()> {
    validate_coalesce_assignment_target(op, target, span)?;

    if matches!(op, AssignmentOp::Assign | AssignmentOp::CoalesceAssign) {
        return Ok(());
    }

    match target {
        AssignmentTarget::Variable { .. } => Ok(()),
        AssignmentTarget::ArrayDim(_)
        | AssignmentTarget::DynamicArrayDim { .. }
        | AssignmentTarget::PropertyArrayDim { .. }
        | AssignmentTarget::StaticPropertyArrayDim { .. } => Ok(()),
        AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::DynamicProperty { .. } => Ok(()),
        AssignmentTarget::StaticProperty { .. } => Ok(()),
        AssignmentTarget::DynamicVariable { .. } | AssignmentTarget::List(_) => Err(
            Diagnostic::new(
                "compound assignment expressions currently support variables, array/string offsets, and properties",
                Some(span),
            ),
        ),
    }
}

fn validate_reference_assignment_target_source(
    target: &AssignmentTarget,
    source: &Expr,
    span: SourceSpan,
) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. } => {}
        AssignmentTarget::DynamicVariable { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::DynamicArrayDim { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::ArrayDim(target) => {
            if reference_source_is_variable(source, &target.array) {
                return Err(Diagnostic::new(
                    "recursive array references are unsupported",
                    Some(target.span),
                ));
            }
        }
        AssignmentTarget::PropertyArrayDim { .. } => {}
        AssignmentTarget::StaticPropertyArrayDim { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::Property { .. } => {}
        AssignmentTarget::DynamicProperty { .. } => {
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
    for (index, element) in elements.into_iter().enumerate() {
        if matches!(element.value, ArrayElementValue::Hole(_)) {
            continue;
        }
        let key = element.key.or_else(|| {
            Some(Expr::Int(
                i64::try_from(index).expect("list destructuring index fits in i64"),
                span,
            ))
        });
        let target = match element.value {
            ArrayElementValue::Value(value) => {
                ListAssignmentElementTarget::Value(Box::new(assignment_target_from_expr(value)?))
            }
            ArrayElementValue::Reference(target) => ListAssignmentElementTarget::Reference(target),
            ArrayElementValue::Hole(_) => unreachable!("list assignment holes are skipped"),
            ArrayElementValue::Unpack(value) => {
                return Err(Diagnostic::new(
                    "Spread operator is not supported in assignments",
                    Some(value.span()),
                ));
            }
        };
        lowered.push(ListAssignmentElement { key, target });
    }
    if lowered.is_empty() {
        return Err(Diagnostic::new("Cannot use empty list", Some(span)));
    }
    Ok(ListAssignmentTarget {
        elements: lowered,
        span,
    })
}

fn list_assignment_target_from_list_expr(list: ListExpr) -> Result<ListAssignmentTarget> {
    let mut lowered = Vec::with_capacity(list.elements.len());
    for (index, element) in list.elements.into_iter().enumerate() {
        let Some(target) = element.target else {
            continue;
        };
        let key = element.key.or_else(|| {
            Some(Expr::Int(
                i64::try_from(index).expect("list destructuring index fits in i64"),
                element.span,
            ))
        });
        let target = match target {
            ListExprElementTarget::Value(value) => {
                ListAssignmentElementTarget::Value(Box::new(assignment_target_from_expr(value)?))
            }
            ListExprElementTarget::Reference(target) => {
                ListAssignmentElementTarget::Reference(target)
            }
        };
        lowered.push(ListAssignmentElement { key, target });
    }
    if lowered.is_empty() {
        return Err(Diagnostic::new("Cannot use empty list", Some(list.span)));
    }
    Ok(ListAssignmentTarget {
        elements: lowered,
        span: list.span,
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
        Expr::Call { .. } => {}
        Expr::FirstClassCallable { callable, .. } => reject_append_array_read(callable)?,
        Expr::DynamicCall { callee, .. } => {
            reject_append_array_read(callee)?;
        }
        Expr::MethodCall { receiver, .. } => reject_append_array_read(receiver)?,
        Expr::DynamicMethodCall { receiver, name, .. } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read(name)?;
        }
        Expr::NewObject { .. } => {}
        Expr::DynamicNewObject { class_name, .. } => reject_append_array_read(class_name)?,
        Expr::PropertyFetch { receiver, .. } | Expr::NullsafePropertyFetch { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read(name)?;
        }
        Expr::DynamicClassNameFetch { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        Expr::InstanceOf { expr, target, .. } => {
            reject_append_array_read(expr)?;
            if let InstanceOfTarget::Expr(target) = target {
                reject_append_array_read(target)?;
            }
        }
        Expr::Clone { expr, .. } => reject_append_array_read(expr)?,
        Expr::StaticPropertyFetch { .. } | Expr::ClassConstantFetch { .. } => {}
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    reject_append_array_read(key)?;
                }
                match &element.value {
                    ArrayElementValue::Hole(span) => {
                        return Err(Diagnostic::new("expected expression", Some(*span)));
                    }
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        reject_append_array_read(value)?;
                    }
                    ArrayElementValue::Reference(_) => {}
                }
            }
        }
        Expr::List(list) => {
            for element in &list.elements {
                if let Some(key) = &element.key {
                    reject_append_array_read(key)?;
                }
                if let Some(ListExprElementTarget::Value(value)) = &element.target {
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
        | Expr::Print {
            expression: target, ..
        }
        | Expr::DynamicVariable { name: target, .. }
        | Expr::Include { path: target, .. }
        | Expr::Throw { value: target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::Cast { expr: target, .. }
        | Expr::Grouped { expr: target, .. }
        | Expr::PipeValue { expr: target, .. } => reject_append_array_read(target)?,
        Expr::Yield { key, value, .. } => {
            if let Some(key) = key {
                reject_append_array_read(key)?;
            }
            if let Some(value) = value {
                reject_append_array_read(value)?;
            }
        }
        Expr::Binary { left, right, .. } => {
            reject_append_array_read(left)?;
            reject_append_array_read(right)?;
        }
        Expr::IncDec { target, .. } => {
            reject_append_array_read_in_inc_dec_target(target)?;
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            reject_append_array_read(condition)?;
            if let Some(if_true) = if_true {
                reject_append_array_read(if_true)?;
            }
            reject_append_array_read(if_false)?;
        }
        Expr::Match { subject, arms, .. } => {
            reject_append_array_read(subject)?;
            for arm in arms {
                for condition in &arm.conditions {
                    reject_append_array_read(condition)?;
                }
                reject_append_array_read(&arm.value)?;
            }
        }
        Expr::InterpolatedString(_, _)
        | Expr::AnonymousFunction(_)
        | Expr::ShellExec { .. }
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

fn reject_array_literal_holes(expr: &Expr) -> Result<()> {
    match expr {
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    reject_array_literal_holes(key)?;
                }
                match &element.value {
                    ArrayElementValue::Hole(span) => {
                        return Err(Diagnostic::new("expected expression", Some(*span)));
                    }
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        reject_array_literal_holes(value)?;
                    }
                    ArrayElementValue::Reference(_) => {}
                }
            }
        }
        Expr::Grouped { expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Clone { expr, .. } => reject_array_literal_holes(expr)?,
        Expr::Binary { left, right, .. } => {
            reject_array_literal_holes(left)?;
            reject_array_literal_holes(right)?;
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            reject_array_literal_holes(condition)?;
            if let Some(if_true) = if_true {
                reject_array_literal_holes(if_true)?;
            }
            reject_array_literal_holes(if_false)?;
        }
        Expr::Call { arguments, .. }
        | Expr::DynamicCall { arguments, .. }
        | Expr::NewObject { arguments, .. }
        | Expr::MethodCall { arguments, .. }
        | Expr::DynamicMethodCall { arguments, .. }
        | Expr::DynamicNewObject { arguments, .. } => {
            for argument in arguments {
                reject_array_literal_holes(argument)?;
            }
        }
        Expr::ArrayAccess { array, index, .. } => {
            reject_array_literal_holes(array)?;
            if let Some(index) = index {
                reject_array_literal_holes(index)?;
            }
        }
        Expr::PropertyFetch { receiver, .. } | Expr::DynamicClassNameFetch { receiver, .. } => {
            reject_array_literal_holes(receiver)?;
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            reject_array_literal_holes(receiver)?;
            reject_array_literal_holes(name)?;
        }
        _ => {}
    }
    Ok(())
}

fn reject_append_array_read_in_inc_dec_target(target: &IncDecTarget) -> Result<()> {
    match target {
        IncDecTarget::ArrayDim(target) => {
            for dimension in &target.dimensions {
                if let Some(dimension) = dimension {
                    reject_append_array_read(dimension)?;
                } else {
                    return Err(Diagnostic::new(
                        "increment/decrement cannot use append array access",
                        Some(target.span),
                    ));
                }
            }
        }
        IncDecTarget::DynamicArrayDim {
            dimensions, span, ..
        } => {
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    reject_append_array_read(dimension)?;
                } else {
                    return Err(Diagnostic::new(
                        "increment/decrement cannot use append array access",
                        Some(*span),
                    ));
                }
            }
        }
        IncDecTarget::PropertyArrayDim {
            receiver,
            dimensions,
            span,
            ..
        } => {
            reject_append_array_read(receiver)?;
            for dimension in dimensions {
                if let Some(dimension) = dimension {
                    reject_append_array_read(dimension)?;
                } else {
                    return Err(Diagnostic::new(
                        "increment/decrement cannot use append array access",
                        Some(*span),
                    ));
                }
            }
        }
        IncDecTarget::Property { receiver, .. } => reject_append_array_read(receiver)?,
        IncDecTarget::Variable { .. }
        | IncDecTarget::DynamicVariable { .. }
        | IncDecTarget::StaticProperty { .. } => {}
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
        TokenStringPart::LegacyDollarBraceVariable(name) => {
            StringPart::LegacyDollarBraceVariable(name)
        }
        TokenStringPart::PropertyFetch { variable, property } => {
            StringPart::PropertyFetch { variable, property }
        }
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

fn dynamic_class_name_fetch_has_illegal_literal_receiver(expr: &Expr) -> bool {
    match expr {
        Expr::Grouped { expr, .. } => dynamic_class_name_fetch_has_illegal_literal_receiver(expr),
        Expr::String(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_) => true,
        _ => false,
    }
}

fn is_supported_global_const_expr(expr: &Expr) -> bool {
    is_supported_global_const_expr_with_options(expr, false)
}

fn is_supported_const_declaration_expr(expr: &Expr) -> bool {
    is_supported_global_const_expr_with_options(expr, true)
}

fn is_supported_global_const_expr_with_options(
    expr: &Expr,
    allow_const_array_unpack_error_operands: bool,
) -> bool {
    match expr {
        Expr::String(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::ClassConstantFetch { .. } => true,
        Expr::Array { elements, .. } => elements.iter().all(|element| {
            element.key.as_ref().is_none_or(|key| {
                is_supported_global_const_expr_with_options(
                    key,
                    allow_const_array_unpack_error_operands,
                )
            }) && match &element.value {
                ArrayElementValue::Hole(_) => false,
                ArrayElementValue::Value(value) => is_supported_global_const_expr_with_options(
                    value,
                    allow_const_array_unpack_error_operands,
                ),
                ArrayElementValue::Unpack(value) => {
                    is_supported_global_const_expr_with_options(
                        value,
                        allow_const_array_unpack_error_operands,
                    ) || (allow_const_array_unpack_error_operands
                        && is_supported_const_array_unpack_error_operand(value))
                }
                ArrayElementValue::Reference(_) => false,
            }
        }),
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Grouped { expr, .. } => {
            is_supported_global_const_expr_with_options(
                expr,
                allow_const_array_unpack_error_operands,
            )
        }
        Expr::FirstClassCallable { callable, .. } => {
            is_supported_first_class_callable_const_target(callable)
        }
        Expr::Binary { left, right, .. } => {
            is_supported_global_const_expr_with_options(
                left,
                allow_const_array_unpack_error_operands,
            ) && is_supported_global_const_expr_with_options(
                right,
                allow_const_array_unpack_error_operands,
            )
        }
        Expr::Ternary { .. } | Expr::Match { .. } => false,
        Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Variable(_, _)
        | Expr::DynamicVariable { .. }
        | Expr::IncDec { .. }
        | Expr::Assign { .. }
        | Expr::AssignRef { .. }
        | Expr::List(_)
        | Expr::Print { .. }
        | Expr::Include { .. }
        | Expr::Throw { .. }
        | Expr::Yield { .. }
        | Expr::AnonymousFunction(_)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::DynamicNewObject { .. }
        | Expr::Clone { .. }
        | Expr::PropertyFetch { .. }
        | Expr::NullsafePropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::DynamicClassNameFetch { .. }
        | Expr::InstanceOf { .. }
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. }
        | Expr::PipeValue { .. } => false,
    }
}

fn is_supported_const_array_unpack_error_operand(expr: &Expr) -> bool {
    match expr {
        Expr::NewObject {
            arguments,
            argument_names,
            ..
        } => {
            argument_names.iter().all(Option::is_none)
                && arguments.iter().all(is_supported_global_const_expr)
        }
        Expr::Grouped { expr, .. } => is_supported_const_array_unpack_error_operand(expr),
        _ => false,
    }
}

fn is_supported_property_default_expr(expr: &Expr) -> bool {
    match expr {
        Expr::FirstClassCallable { .. } => false,
        Expr::Grouped { expr, .. } => is_supported_property_default_expr(expr),
        _ => is_supported_global_const_expr(expr),
    }
}

fn is_supported_first_class_callable_const_target(callable: &Expr) -> bool {
    matches!(callable, Expr::String(_, _))
        || matches!(
            callable,
            Expr::Grouped { expr, .. } if is_supported_first_class_callable_const_target(expr)
        )
}

fn is_supported_parameter_default_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::ClassConstantFetch { .. } => true,
        Expr::Array { elements, .. } => elements.iter().all(|element| {
            element
                .key
                .as_ref()
                .is_none_or(is_supported_parameter_default_expr)
                && match &element.value {
                    ArrayElementValue::Hole(_) => false,
                    ArrayElementValue::Value(value) => is_supported_parameter_default_expr(value),
                    ArrayElementValue::Unpack(value) => is_supported_parameter_default_expr(value),
                    ArrayElementValue::Reference(_) => false,
                }
        }),
        Expr::Unary { expr, .. } | Expr::Grouped { expr, .. } => {
            is_supported_parameter_default_expr(expr)
        }
        Expr::FirstClassCallable { callable, .. } => {
            is_supported_first_class_callable_const_target(callable)
        }
        Expr::Cast { .. } => false,
        Expr::Binary { left, right, .. } => {
            is_supported_parameter_default_expr(left) && is_supported_parameter_default_expr(right)
        }
        Expr::Ternary { .. }
        | Expr::Match { .. }
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Variable(_, _)
        | Expr::DynamicVariable { .. }
        | Expr::IncDec { .. }
        | Expr::Assign { .. }
        | Expr::AssignRef { .. }
        | Expr::List(_)
        | Expr::Print { .. }
        | Expr::Include { .. }
        | Expr::Throw { .. }
        | Expr::Yield { .. }
        | Expr::AnonymousFunction(_)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::DynamicNewObject { .. }
        | Expr::Clone { .. }
        | Expr::PropertyFetch { .. }
        | Expr::NullsafePropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::DynamicClassNameFetch { .. }
        | Expr::InstanceOf { .. }
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. }
        | Expr::PipeValue { .. } => false,
    }
}

fn validate_class_scoped_constant_exprs(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        for constant in &class.constants {
            validate_class_scoped_constant_expr(&constant.value, class.parent_name.as_deref())?;
        }
        for property in &class.properties {
            if let Some(value) = &property.value {
                validate_class_scoped_constant_expr(value, class.parent_name.as_deref())?;
            }
        }
        for property in &class.static_properties {
            if let Some(value) = &property.value {
                validate_class_scoped_constant_expr(value, class.parent_name.as_deref())?;
            }
        }
        for method in &class.methods {
            for parameter in &method.parameters {
                if let Some(default_value) = &parameter.default_value {
                    validate_class_scoped_constant_expr(
                        default_value,
                        class.parent_name.as_deref(),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_class_scoped_constant_expr(expr: &Expr, parent_name: Option<&str>) -> Result<()> {
    match expr {
        Expr::ClassConstantFetch {
            class_name: fetch_class_name,
            name,
            span,
        } if name.eq_ignore_ascii_case("class") => {
            if fetch_class_name.eq_ignore_ascii_case("static") {
                return Err(Diagnostic::new(
                    "static::class cannot be used for compile-time class name resolution",
                    Some(*span),
                ));
            }
            if fetch_class_name.eq_ignore_ascii_case("parent") && parent_name.is_none() {
                return Err(Diagnostic::new(
                    "Cannot use \"parent\" when current class scope has no parent",
                    Some(*span),
                ));
            }
            Ok(())
        }
        Expr::Array { elements, .. } => {
            for element in elements {
                if let Some(key) = &element.key {
                    validate_class_scoped_constant_expr(key, parent_name)?;
                }
                match &element.value {
                    ArrayElementValue::Hole(_) => {}
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        validate_class_scoped_constant_expr(value, parent_name)?;
                    }
                    ArrayElementValue::Reference(_) => {}
                }
            }
            Ok(())
        }
        Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Grouped { expr, .. }
        | Expr::PipeValue { expr, .. } => validate_class_scoped_constant_expr(expr, parent_name),
        Expr::Binary { left, right, .. } => {
            validate_class_scoped_constant_expr(left, parent_name)?;
            validate_class_scoped_constant_expr(right, parent_name)
        }
        Expr::InstanceOf { expr, target, .. } => {
            validate_class_scoped_constant_expr(expr, parent_name)?;
            if let InstanceOfTarget::Expr(target) = target {
                validate_class_scoped_constant_expr(target, parent_name)?;
            }
            Ok(())
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            validate_class_scoped_constant_expr(condition, parent_name)?;
            if let Some(if_true) = if_true {
                validate_class_scoped_constant_expr(if_true, parent_name)?;
            }
            validate_class_scoped_constant_expr(if_false, parent_name)
        }
        Expr::Match { subject, arms, .. } => {
            validate_class_scoped_constant_expr(subject, parent_name)?;
            for arm in arms {
                for condition in &arm.conditions {
                    validate_class_scoped_constant_expr(condition, parent_name)?;
                }
                validate_class_scoped_constant_expr(&arm.value, parent_name)?;
            }
            Ok(())
        }
        Expr::FirstClassCallable { callable, .. } => {
            validate_class_scoped_constant_expr(callable, parent_name)
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::DynamicVariable { .. }
        | Expr::AnonymousFunction(_)
        | Expr::IncDec { .. }
        | Expr::Assign { .. }
        | Expr::AssignRef { .. }
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::DynamicNewObject { .. }
        | Expr::Clone { .. }
        | Expr::PropertyFetch { .. }
        | Expr::NullsafePropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. }
        | Expr::DynamicClassNameFetch { .. }
        | Expr::List(_)
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. }
        | Expr::Print { .. }
        | Expr::Include { .. }
        | Expr::Throw { .. }
        | Expr::Yield { .. } => Ok(()),
    }
}

fn validate_function_parameter_defaults(parameters: &[FunctionParameter]) -> Result<()> {
    let mut seen_default = false;
    for parameter in parameters {
        if parameter.is_variadic && parameter.default_value.is_some() {
            return Err(Diagnostic::new(
                "variadic function parameter cannot have a default value",
                Some(parameter.span),
            ));
        }
        if parameter.default_value.is_some() {
            seen_default = true;
        } else if seen_default && !parameter.is_variadic {
            return Err(Diagnostic::new(
                "required function parameter cannot follow an optional parameter",
                Some(parameter.span),
            ));
        }
    }
    Ok(())
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
