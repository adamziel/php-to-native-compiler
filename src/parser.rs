use std::collections::{HashMap, HashSet};

use crate::ast::{
    AnonymousFunction, ArrayDimTarget, ArrayElement, ArrayElementValue, AssignmentOp,
    AssignmentTarget, AttributeConstantReference, AttributeMetadata, BinaryOp, CastKind,
    CatchClause, ClassConstantDecl, ClassDecl, ClosureUseCapture, CompileWarning,
    CompileWarningKind, ConstDeclaration, Expr, FunctionDecl, FunctionParameter, IncDecOp,
    IncDecResult, IncDecTarget, IncludeKind, InstanceOfTarget, ListAssignmentElement,
    ListAssignmentElementTarget, ListAssignmentTarget, ListExpr, ListExprElement,
    ListExprElementTarget, MagicConstantKind, MatchArm, MethodDecl, Program, PromotedProperty,
    PropertyDecl, PropertyTypeHint, PropertyTypeKind, PropertyVisibility, ReferenceTarget,
    Statement, StaticLocalDeclaration, StaticPropertyDecl, StringInterpolationIndex, StringPart,
    SwitchCase, TraitAdaptation, TraitAliasAdaptation, TraitDecl, TraitMethodReference,
    TraitPrecedenceAdaptation, TraitUseDecl, TypeHint, UnaryOp, UnsetTarget,
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
    parse_with_runtime_class_aliases(source, &HashMap::new())
}

pub(crate) fn parse_with_runtime_class_aliases(
    source: &str,
    runtime_class_aliases: &HashMap<String, String>,
) -> Result<Program> {
    let tokens = lex(source)?;
    let compiler_halt_offset = find_compiler_halt_offset(&tokens);
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
        runtime_class_aliases: runtime_class_aliases.clone(),
        declared_functions: HashSet::new(),
        declared_constants: HashSet::new(),
        declared_class_names: HashMap::new(),
        anonymous_classes: Vec::new(),
        nested_functions: Vec::new(),
        anonymous_class_name_counts: HashMap::new(),
        allow_append_array_read: false,
        active_type_scope: None,
        allow_unscoped_relative_types: 0,
        return_by_ref_stack: Vec::new(),
        strict_types: false,
        compiler_halt_offset,
        compile_warnings: Vec::new(),
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
    declared_constants: HashSet<String>,
    declared_class_names: HashMap<String, SourceSpan>,
    anonymous_classes: Vec<ClassDecl>,
    nested_functions: Vec<FunctionDecl>,
    anonymous_class_name_counts: HashMap<String, usize>,
    allow_append_array_read: bool,
    active_type_scope: Option<ActiveTypeScope>,
    allow_unscoped_relative_types: usize,
    return_by_ref_stack: Vec<bool>,
    strict_types: bool,
    compiler_halt_offset: Option<i64>,
    compile_warnings: Vec<CompileWarning>,
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

fn find_compiler_halt_offset(tokens: &[Token]) -> Option<i64> {
    tokens.windows(4).find_map(|window| {
        let [name, left, right, semicolon] = window else {
            return None;
        };
        if token_is_identifier_named(name, "__halt_compiler")
            && matches!(left.kind, TokenKind::LeftParen)
            && matches!(right.kind, TokenKind::RightParen)
            && matches!(semicolon.kind, TokenKind::Semicolon)
        {
            Some(semicolon.span.byte_end as i64)
        } else {
            None
        }
    })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FunctionLikeKind {
    Function,
    Method,
}

impl FunctionLikeKind {
    fn label(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Method => "method",
        }
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
        self.visibility_span.is_some()
            || self.set_visibility_span.is_some()
            || self.is_readonly
            || self.is_final
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
        validate_interface_references(&classes, &mut self.compile_warnings)?;
        validate_interface_method_conflicts(&classes)?;
        validate_override_attributes(&classes, &traits)?;
        validate_traversable_implementations(&classes)?;
        validate_method_signature_compatibility(
            &classes,
            &self.runtime_class_aliases,
            &mut self.compile_warnings,
        )?;
        validate_abstract_methods(&classes)?;
        validate_final_class_inheritance(&classes)?;
        validate_readonly_class_inheritance(&classes)?;
        validate_property_override_set_visibility(&classes)?;
        validate_property_type_invariance(&classes)?;
        validate_class_constant_overrides(&classes)?;
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
                    validate_void_returns_in_statements(&method.body, FunctionLikeKind::Method)?;
                }
                if matches!(&method.return_type, Some(TypeHint::Never)) {
                    validate_never_returns_in_statements(&method.body, FunctionLikeKind::Method)?;
                }
                if let Some(return_type) = &method.return_type {
                    validate_generator_return_type(&method.body, return_type, method.span)?;
                    validate_typed_returns_have_values_in_statements(
                        &method.body,
                        return_type,
                        FunctionLikeKind::Method,
                    )?;
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
        validate_never_returns(&functions)?;
        validate_typed_returns_have_values(&functions)?;
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
            compile_warnings: self.compile_warnings.clone(),
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
            if token_is_identifier_named(self.peek(), "__halt_compiler") {
                self.parse_halt_compiler_statement(scope)?;
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
                    if self.namespace_declaration_uses_semicolon() {
                        return Err(Diagnostic::new(
                            "Cannot mix bracketed namespace declarations with unbracketed namespace declarations",
                            Some(self.peek().span),
                        ));
                    }
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
            } else if token_is_identifier_named(self.peek(), "enum") {
                self.reject_code_outside_bracketed_namespace(scope)?;
                let class = self.parse_enum_decl(attributes)?;
                self.register_declared_class_name(&class)?;
                classes.push(class);
            } else if self.peek_starts_class_decl() {
                self.reject_code_outside_bracketed_namespace(scope)?;
                let class = self.parse_class_decl(attributes)?;
                self.register_declared_class_name(&class)?;
                classes.push(class);
            } else {
                self.reject_code_outside_bracketed_namespace(scope)?;
                let statement = self.parse_statement_with_attributes(attributes)?;
                self.note_runtime_class_alias_statement(&statement);
                statements.push(statement);
            }
        }
        Ok(())
    }

    fn namespace_declaration_uses_semicolon(&self) -> bool {
        for token in self.tokens.iter().skip(self.index + 1) {
            match token.kind {
                TokenKind::Semicolon => return true,
                TokenKind::LeftBrace | TokenKind::Eof => return false,
                _ => {}
            }
        }
        false
    }

    fn parse_halt_compiler_statement(&mut self, scope: TopLevelScope) -> Result<()> {
        let name_span = self.advance().span;
        if scope != TopLevelScope::Program || self.block_depth != 0 || self.function_depth != 0 {
            return Err(Diagnostic::new(
                "__HALT_COMPILER() can only be used from the outermost scope",
                Some(name_span),
            ));
        }
        self.expect_left_paren()?;
        self.expect_right_paren()?;
        self.expect_semicolon()?;
        if !self.tokens.is_empty() {
            self.index = self.tokens.len() - 1;
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
        if kind == UseDeclarationKind::Class
            && self.current_namespace.is_none()
            && !target.name.contains('\\')
            && alias.eq_ignore_ascii_case(&target.name)
            && matches!(
                target.resolution,
                NameResolution::Unqualified | NameResolution::FullyQualified
            )
        {
            self.compile_warnings.push(CompileWarning {
                message: format!(
                    "The use statement with non-compound name '{}' has no effect",
                    target.name
                ),
                span: target.span,
                kind: CompileWarningKind::Warning,
            });
            return Ok(());
        }
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
            if outer_kind != UseDeclarationKind::Class
                && (matches!(self.peek().kind, TokenKind::Function)
                    || matches!(self.peek().kind, TokenKind::Const))
            {
                let token = self.peek().clone();
                return Err(Diagnostic::parse_error(
                    format!(
                        "syntax error, unexpected token \"{}\", expecting \"}}\"",
                        token_text(&token.kind)
                    ),
                    Some(token.span),
                ));
            }
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
            } else if outer_kind != UseDeclarationKind::Class
                && matches!(self.peek().kind, TokenKind::Function | TokenKind::Const)
            {
                return Err(Diagnostic::parse_error(
                    format!(
                        "syntax error, unexpected token \"{}\", expecting \"}}\"",
                        token_text(&self.peek().kind)
                    ),
                    Some(self.peek().span),
                ));
            } else {
                outer_kind
            };
            let item = self.parse_name("expected imported name")?;
            if item.resolution == NameResolution::FullyQualified {
                let text = self
                    .source
                    .get(item.span.byte_start..item.span.byte_end)
                    .unwrap_or("");
                return Err(Diagnostic::parse_error(
                    format!(
                        "syntax error, unexpected fully qualified name \"{text}\", expecting identifier or namespaced name or \"function\" or \"const\""
                    ),
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
            if matches!(self.peek().kind, TokenKind::LeftBrace) {
                return Err(Diagnostic::parse_error(
                    "syntax error, unexpected token \"{\", expecting \"}\"",
                    Some(self.peek().span),
                ));
            }
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
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            return Err(Diagnostic::parse_error(
                "syntax error, unexpected token \"{\", expecting \"}\"",
                Some(self.peek().span),
            ));
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
        let Some(first_segment) = name_segment_from_token(&first_token.kind) else {
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
            let separator_span = self.advance().span;
            let segment_token = self.advance().clone();
            if self.has_namespace_separator_whitespace(span, separator_span, segment_token.span) {
                return Err(Diagnostic::parse_error(
                    "syntax error, unexpected token \"\\\"",
                    Some(separator_span),
                ));
            }
            let Some(segment) = name_segment_from_token(&segment_token.kind) else {
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
        let (name, span, _) = self.parse_resolved_class_name_with_source(expected)?;
        Ok((name, span))
    }

    fn parse_resolved_class_name_with_source(
        &mut self,
        expected: &str,
    ) -> Result<(String, SourceSpan, String)> {
        let parsed = self.parse_name(expected)?;
        let span = parsed.span;
        let source_name = self.parsed_name_source_text(&parsed);
        let name = self.resolve_class_name(&parsed);
        Ok((
            self.resolve_runtime_class_alias_name(&name),
            span,
            source_name,
        ))
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
        let Some(first_segment) = name_segment_from_token(&first_token.kind) else {
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
            let separator_span = self.advance().span;
            let segment_token = self.advance().clone();
            if self.has_namespace_separator_whitespace(span, separator_span, segment_token.span) {
                return Err(Diagnostic::parse_error(
                    "syntax error, unexpected token \"\\\"",
                    Some(separator_span),
                ));
            }
            let Some(segment) = name_segment_from_token(&segment_token.kind) else {
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
        resolved
    }

    fn resolve_constant_name(&self, parsed: &ParsedName) -> String {
        match parsed.resolution {
            NameResolution::FullyQualified => parsed.name.clone(),
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&parsed.name),
            NameResolution::Unqualified => {
                let alias_key = parsed.name.to_ascii_lowercase();
                let namespaced = self.qualify_current_namespace(&parsed.name);
                if let Some(target) = self.constant_aliases.get(&alias_key) {
                    target.clone()
                } else if namespaced != parsed.name
                    && self
                        .declared_constants
                        .contains(&namespaced.to_ascii_lowercase())
                {
                    namespaced
                } else if is_modeled_global_constant_name(&parsed.name) {
                    parsed.name.clone()
                } else {
                    namespaced
                }
            }
            NameResolution::Qualified => {
                self.resolve_aliasable_name(&parsed.name, &self.class_aliases)
            }
        }
    }

    fn parsed_name_source_text(&self, parsed: &ParsedName) -> String {
        self.source
            .get(parsed.span.byte_start..parsed.span.byte_end)
            .unwrap_or(parsed.name.as_str())
            .to_string()
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

    fn has_namespace_separator_whitespace(
        &self,
        previous_span: SourceSpan,
        separator_span: SourceSpan,
        next_span: SourceSpan,
    ) -> bool {
        let before = self
            .source
            .get(previous_span.byte_end..separator_span.byte_start)
            .unwrap_or("");
        let after = self
            .source
            .get(separator_span.byte_end..next_span.byte_start)
            .unwrap_or("");
        before.chars().any(char::is_whitespace) || after.chars().any(char::is_whitespace)
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

    fn parse_enum_decl(&mut self, attributes: ParsedAttributes) -> Result<ClassDecl> {
        let enum_token = self.advance().clone();
        let TokenKind::Identifier(keyword) = &enum_token.kind else {
            return Err(Diagnostic::new("expected enum", Some(enum_token.span)));
        };
        if !keyword.eq_ignore_ascii_case("enum") {
            return Err(Diagnostic::new("expected enum", Some(enum_token.span)));
        }

        let (class_name, _) = self.parse_declaration_name("expected enum name")?;
        if matches!(self.peek().kind, TokenKind::Colon) {
            return Err(Diagnostic::new(
                "backed enum declarations are unsupported",
                Some(self.peek().span),
            ));
        }

        let mut interfaces = Vec::new();
        if token_is_identifier_named(self.peek(), "implements") {
            self.advance();
            interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                interfaces.push(self.parse_resolved_class_name("expected interface name")?.0);
            }
        }

        self.expect_left_brace()?;
        let mut constants = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            let attributes = self.parse_attribute_groups()?;
            if !matches!(self.peek().kind, TokenKind::Case) {
                return Err(syntax_error_unexpected(self.peek(), Some("case")));
            }
            self.advance();
            let token = self.advance().clone();
            let TokenKind::Identifier(name) = token.kind else {
                return Err(Diagnostic::new("expected enum case name", Some(token.span)));
            };
            if matches!(self.peek().kind, TokenKind::Equal) {
                return Err(Diagnostic::new(
                    "backed enum case values are unsupported",
                    Some(self.peek().span),
                ));
            }
            validate_builtin_attributes_for_target(
                &attributes,
                AttributeTarget::ClassConstant,
                token.span,
            )?;
            self.expect_semicolon()?;
            constants.push(ClassConstantDecl {
                name,
                visibility: PropertyVisibility::Public,
                attributes,
                value: Expr::Null(token.span),
                is_enum_case: true,
                is_final: false,
                span: token.span,
            });
        }
        let right_span = self.expect_right_brace()?;
        Ok(ClassDecl {
            name: class_name,
            parent_name: None,
            interfaces,
            trait_uses: Vec::new(),
            attributes,
            is_abstract: false,
            is_final: true,
            is_interface: false,
            is_readonly: false,
            properties: Vec::new(),
            static_properties: Vec::new(),
            constants,
            methods: Vec::new(),
            span: combine_spans(enum_token.span, right_span),
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
                    method.trait_method_name = Some(method.name.clone());
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
        let mut trait_names = Vec::new();
        loop {
            let (name, span) = self.parse_resolved_class_name("expected trait name")?;
            trait_names.push((name, span));
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        let adaptations = if matches!(self.peek().kind, TokenKind::LeftBrace) {
            self.parse_trait_adaptations()?
        } else {
            self.expect_semicolon()?;
            Vec::new()
        };
        Ok(trait_names
            .into_iter()
            .map(|(name, span)| TraitUseDecl {
                name,
                adaptations: adaptations.clone(),
                span,
            })
            .collect())
    }

    fn parse_trait_adaptations(&mut self) -> Result<Vec<TraitAdaptation>> {
        self.expect_left_brace()?;
        let mut adaptations = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace) {
            let method = self.parse_trait_method_reference()?;
            if matches!(self.peek().kind, TokenKind::As) {
                let span_start = method.span;
                self.advance();
                let mut visibility = None;
                let mut alias = None;
                if let Some(parsed_visibility) = self.parse_optional_trait_alias_visibility() {
                    visibility = Some(parsed_visibility);
                    if !matches!(self.peek().kind, TokenKind::Semicolon) {
                        alias = Some(self.parse_trait_adaptation_method_name()?);
                    }
                } else if !matches!(self.peek().kind, TokenKind::Semicolon) {
                    alias = Some(self.parse_trait_adaptation_method_name()?);
                }
                let semicolon = self.expect_semicolon()?;
                adaptations.push(TraitAdaptation::Alias(TraitAliasAdaptation {
                    method,
                    alias,
                    visibility,
                    span: combine_spans(span_start, semicolon),
                }));
                continue;
            }
            if token_is_identifier_named(self.peek(), "insteadof") {
                let span_start = method.span;
                self.advance();
                let mut instead_of = Vec::new();
                loop {
                    let (name, _) = self.parse_resolved_class_name("expected trait name")?;
                    instead_of.push(name);
                    if !matches!(self.peek().kind, TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }
                let semicolon = self.expect_semicolon()?;
                adaptations.push(TraitAdaptation::Precedence(TraitPrecedenceAdaptation {
                    method,
                    instead_of,
                    span: combine_spans(span_start, semicolon),
                }));
                continue;
            }
            return Err(Diagnostic::new(
                "expected trait adaptation operator",
                Some(self.peek().span),
            ));
        }
        self.expect_right_brace()?;
        Ok(adaptations)
    }

    fn parse_trait_method_reference(&mut self) -> Result<TraitMethodReference> {
        let start = self.peek().span;
        if self.trait_method_reference_has_scope() {
            let (trait_name, trait_span) = self.parse_resolved_class_name("expected trait name")?;
            let double_colon = self.advance();
            if !matches!(double_colon.kind, TokenKind::DoubleColon) {
                return Err(Diagnostic::new("expected ::", Some(double_colon.span)));
            }
            let method_name = self.parse_trait_adaptation_method_name()?;
            return Ok(TraitMethodReference {
                trait_name: Some(trait_name),
                method_name,
                span: combine_spans(trait_span, self.previous_span()),
            });
        }
        let method_name = self.parse_trait_adaptation_method_name()?;
        Ok(TraitMethodReference {
            trait_name: None,
            method_name,
            span: combine_spans(start, self.previous_span()),
        })
    }

    fn trait_method_reference_has_scope(&self) -> bool {
        let mut index = self.index;
        while index < self.tokens.len() {
            let token = &self.tokens[index];
            if matches!(token.kind, TokenKind::DoubleColon) {
                return true;
            }
            if matches!(
                token.kind,
                TokenKind::As | TokenKind::Semicolon | TokenKind::RightBrace
            ) || token_is_identifier_named(token, "insteadof")
            {
                return false;
            }
            index += 1;
        }
        false
    }

    fn parse_optional_trait_alias_visibility(&mut self) -> Option<PropertyVisibility> {
        let TokenKind::Identifier(name) = &self.peek().kind else {
            return None;
        };
        let visibility = match name.to_ascii_lowercase().as_str() {
            "public" => PropertyVisibility::Public,
            "protected" => PropertyVisibility::Protected,
            "private" => PropertyVisibility::Private,
            _ => return None,
        };
        self.advance();
        Some(visibility)
    }

    fn parse_trait_adaptation_method_name(&mut self) -> Result<String> {
        let token = self.advance();
        if let Some(name) = method_name_from_token(&token.kind) {
            return Ok(name);
        }
        Err(Diagnostic::new("expected method name", Some(token.span)))
    }

    fn previous_span(&self) -> SourceSpan {
        self.tokens
            .get(self.index.saturating_sub(1))
            .map(|token| token.span)
            .unwrap_or_else(|| SourceSpan::new(0, 0, 0, 0))
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
            if class_is_interface && modifiers.visibility != PropertyVisibility::Public {
                return Err(Diagnostic::new(
                    format!("Access type for interface constant {class_name} must be public"),
                    modifiers.visibility_span,
                ));
            }
            return Ok(ParsedClassMember::Constants(
                self.parse_class_constant_declarations(
                    modifiers.visibility,
                    modifiers.is_final,
                    attributes,
                )?,
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
                        modifiers.is_final,
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
                    modifiers.is_final,
                    attributes,
                    true,
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
                        modifiers.is_final,
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
                    modifiers.is_final,
                    attributes,
                    true,
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
        is_final: bool,
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
            is_final,
            type_hint.clone(),
            attributes.clone(),
            class_name,
        )?];
        validate_builtin_attributes_for_target(
            &attributes,
            AttributeTarget::Property,
            properties
                .first()
                .map(|property| property.span)
                .unwrap_or_else(|| self.previous_span()),
        )?;
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            properties.push(self.parse_static_property_declaration(
                visibility,
                set_visibility,
                is_final,
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
        is_final: bool,
        attributes: ParsedAttributes,
    ) -> Result<Vec<ClassConstantDecl>> {
        self.expect_const()?;
        let mut constants = vec![self.parse_class_constant_declaration(
            visibility,
            is_final,
            attributes.clone(),
        )?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            if attributes.total_count > 0 {
                return Err(Diagnostic::new(
                    "Cannot apply attributes to multiple constants at once",
                    Some(self.peek().span),
                ));
            }
            self.advance();
            constants.push(self.parse_class_constant_declaration(
                visibility,
                is_final,
                AttributeMetadata::default(),
            )?);
        }
        validate_builtin_attributes_for_target(
            &attributes,
            AttributeTarget::ClassConstant,
            constants
                .first()
                .map(|constant| constant.span)
                .unwrap_or_else(|| self.previous_span()),
        )?;
        self.expect_const_statement_terminator()?;
        Ok(constants)
    }

    fn parse_class_constant_declaration(
        &mut self,
        visibility: PropertyVisibility,
        is_final: bool,
        attributes: ParsedAttributes,
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
        let Some(name) = name_segment_from_token(&token.kind) else {
            return Err(Diagnostic::new(
                "expected class constant name",
                Some(token.span),
            ));
        };
        self.expect_equal()?;
        let value = self.parse_const_context_expr()?;
        if const_expr_contains_new_object(&value) {
            return Err(Diagnostic::new(
                "New expressions are not supported in this context",
                Some(value.span()),
            ));
        }
        if !is_supported_const_declaration_expr(&value) {
            return Err(Diagnostic::new(
                "class constant value must be a supported constant expression",
                Some(value.span()),
            ));
        }
        Ok(ClassConstantDecl {
            name,
            visibility,
            attributes,
            value,
            is_enum_case: false,
            is_final,
            span: token.span,
        })
    }

    fn parse_property_declarations(
        &mut self,
        visibility: PropertyVisibility,
        set_visibility: PropertyVisibility,
        set_visibility_span: Option<SourceSpan>,
        is_readonly: bool,
        is_final: bool,
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
            is_final,
            is_readonly,
            type_hint.clone(),
            attributes.clone(),
            allow_property_hooks,
            class_name,
        )?;
        let mut properties = vec![first_property];
        validate_builtin_attributes_for_target(
            &attributes,
            AttributeTarget::Property,
            properties
                .first()
                .map(|property| property.span)
                .unwrap_or_else(|| self.previous_span()),
        )?;
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
                is_final,
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
        is_final: bool,
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
            let (is_virtual, hook_get_value) = self.parse_property_hook_block(&name)?;
            return Ok((
                PropertyDecl {
                    name,
                    visibility,
                    set_visibility,
                    is_final,
                    is_readonly,
                    has_hooks: true,
                    is_virtual,
                    hook_get_value,
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
                is_final,
                is_readonly,
                has_hooks: false,
                is_virtual: false,
                hook_get_value: None,
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
        is_final: bool,
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
            is_final,
            type_hint,
            has_override_attribute: attributes.has_override,
            value,
            span: token.span,
        })
    }

    fn parse_property_hook_block(&mut self, property_name: &str) -> Result<(bool, Option<Expr>)> {
        self.expect_left_brace()?;
        let mut is_virtual = true;
        let mut get_value = None;
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            let token = self.advance().clone();
            match token.kind {
                TokenKind::Identifier(name) if name.eq_ignore_ascii_case("get") => {
                    if matches!(self.peek().kind, TokenKind::DoubleArrow) {
                        self.advance();
                        let value = self.parse_expr()?;
                        if !is_supported_property_default_expr(&value) {
                            return Err(Diagnostic::new(
                                "property hook get expression must be a supported constant expression",
                                Some(value.span()),
                            ));
                        }
                        self.expect_semicolon()?;
                        get_value = Some(value);
                    } else if self.skip_property_hook_body("get", property_name)? {
                        is_virtual = false;
                    }
                }
                TokenKind::Identifier(name) => {
                    if self.skip_property_hook_body(&name, property_name)? {
                        is_virtual = false;
                    }
                }
                TokenKind::LeftBrace => {
                    if self.skip_braced_property_hook_body(property_name)? {
                        is_virtual = false;
                    }
                }
                TokenKind::Eof => {
                    return Err(Diagnostic::new(
                        "unterminated property hook block",
                        Some(token.span),
                    ));
                }
                _ => {}
            }
        }
        self.expect_right_brace()?;
        Ok((is_virtual, get_value))
    }

    fn skip_property_hook_body(&mut self, hook_name: &str, property_name: &str) -> Result<bool> {
        if matches!(self.peek().kind, TokenKind::DoubleArrow) {
            self.advance();
            let _ = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(hook_name.eq_ignore_ascii_case("set"));
        }
        if matches!(self.peek().kind, TokenKind::LeftBrace) {
            return self.skip_braced_property_hook_body(property_name);
        }
        if matches!(self.peek().kind, TokenKind::Semicolon) {
            self.advance();
        }
        Ok(false)
    }

    fn skip_braced_property_hook_body(&mut self, property_name: &str) -> Result<bool> {
        self.expect_left_brace()?;
        let mut depth = 1usize;
        let mut uses_property = false;
        while depth > 0 {
            let token = self.advance().clone();
            match token.kind {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => depth -= 1,
                TokenKind::Variable(variable)
                    if variable.eq_ignore_ascii_case("this")
                        && matches!(self.peek().kind, TokenKind::ObjectOperator)
                        && matches!(
                            &self.peek_n(1).kind,
                            TokenKind::Identifier(name) if name == property_name
                        ) =>
                {
                    uses_property = true;
                }
                TokenKind::InterpolatedString(parts)
                    if token_string_parts_use_this_property(&parts, property_name) =>
                {
                    uses_property = true;
                }
                TokenKind::Eof => {
                    return Err(Diagnostic::new(
                        "unterminated property hook block",
                        Some(token.span),
                    ));
                }
                _ => {}
            }
        }
        Ok(uses_property)
    }

    fn parse_optional_property_type_hint(&mut self) -> Result<Option<PropertyTypeHint>> {
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            return Ok(None);
        }
        if !self.peek_is_type_hint() {
            return Ok(None);
        }
        let parsed = self.parse_type_hint_with_context(TypeHintContext::Parameter)?;
        Ok(Some(property_type_hint_from_type_hint(&parsed.type_hint)))
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
        if allow_promoted_properties
            && modifiers.is_abstract
            && parameters
                .iter()
                .any(|parameter| parameter.promoted_property.is_some())
        {
            return Err(Diagnostic::new(
                "Cannot declare promoted property in an abstract constructor",
                parameters
                    .iter()
                    .find_map(|parameter| parameter.promoted_property.as_ref())
                    .map(|promoted| promoted.span),
            ));
        }
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
            trait_method_name: None,
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
        self.parse_statement_with_attributes(attributes)
    }

    fn parse_statement_with_attributes(
        &mut self,
        attributes: ParsedAttributes,
    ) -> Result<Statement> {
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
            TokenKind::Const => self.parse_const(attributes),
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
            TokenKind::Identifier(ref name)
                if is_exit_construct_name(name)
                    && !matches!(self.peek_next().kind, TokenKind::Colon) =>
            {
                self.parse_exit()
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
            TokenKind::Identifier(ref name)
                if name.eq_ignore_ascii_case("__halt_compiler")
                    && matches!(self.peek_next().kind, TokenKind::LeftParen) =>
            {
                Err(Diagnostic::new(
                    "__HALT_COMPILER() can only be used from the outermost scope",
                    Some(self.peek().span),
                ))
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
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if is_exit_construct_name(name) {
                return Err(exit_reserved_name_syntax_error(self.peek(), "\"(\""));
            }
        }
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
                if modifiers.is_static {
                    return Err(Diagnostic::new(
                        "Cannot use the static modifier on a parameter",
                        modifiers.static_span,
                    ));
                }
                if modifiers.is_abstract {
                    return Err(Diagnostic::new(
                        "Cannot use the abstract modifier on a parameter",
                        modifiers.abstract_span,
                    ));
                }
                Some(modifiers)
            } else {
                None
            }
        } else {
            if self.peek_starts_promoted_parameter_modifier() {
                let modifiers = self.parse_class_modifiers()?;
                if modifiers.has_promoted_property_modifier() {
                    return Err(Diagnostic::new(
                        "Cannot declare promoted property outside a constructor",
                        modifiers
                            .visibility_span
                            .or(modifiers.readonly_span)
                            .or(modifiers.final_span)
                            .or(modifiers.set_visibility_span),
                    ));
                }
            }
            None
        };
        let type_hint = if self.peek_is_type_hint() {
            let parsed = self.parse_type_hint_with_context(TypeHintContext::Parameter)?;
            validate_parameter_type_hint(&parsed.type_hint, parsed.span)?;
            Some(parsed.type_hint)
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
            if is_variadic {
                return Err(Diagnostic::new(
                    "Cannot declare variadic promoted property",
                    Some(token.span),
                ));
            }
            if matches!(type_hint, Some(TypeHint::Callable)) {
                return Err(Diagnostic::new(
                    format!("Property {class_name}::${name} cannot have type callable"),
                    Some(token.span),
                ));
            }
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
                is_final: modifiers.is_final,
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
        if promoted_property.is_some()
            && matches!(default_value, Some(Expr::Null(_)))
            && type_hint
                .as_ref()
                .is_some_and(|type_hint| !type_hint_accepts_null_default(type_hint))
        {
            return Err(Diagnostic::new(
                format!(
                    "Cannot use null as default value for parameter ${name} of type {}",
                    type_hint_display(type_hint.as_ref().expect("type hint checked above"))
                ),
                Some(token.span),
            ));
        }
        Ok(FunctionParameter {
            name,
            attributes,
            type_hint,
            by_ref,
            is_variadic,
            default_value,
            promoted_property,
            span: token.span,
        })
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
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("void") => {
                self.advance();
                TypeHint::Void
            }
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("never") => {
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
            TokenKind::StringType | TokenKind::BinaryType
                if !matches!(self.peek_next().kind, TokenKind::Backslash) =>
            {
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
            _ if name_segment_from_token(&token.kind).is_some()
                && matches!(self.peek_next().kind, TokenKind::Backslash) =>
            {
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
                    || name.eq_ignore_ascii_case("void")
                    || name.eq_ignore_ascii_case("never")
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
        let op_span = self.peek().span;
        let op = self.expect_assignment_op()?;
        let target = AssignmentTarget::Variable {
            name: name.clone(),
            span: token.span,
        };
        validate_expression_assignment_target(op, &target, op_span)?;
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

    fn parse_const(&mut self, attributes: ParsedAttributes) -> Result<Statement> {
        let span = self.expect_const()?;
        if self.block_depth != 0 {
            return Err(Diagnostic::new(
                "constant declarations must be at global scope",
                Some(span),
            ));
        }

        let mut declarations = vec![self.parse_const_declaration(attributes.clone())?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            if attributes.total_count > 0 {
                return Err(Diagnostic::new(
                    "Cannot apply attributes to multiple constants at once",
                    Some(self.peek().span),
                ));
            }
            self.advance();
            declarations.push(self.parse_const_declaration(AttributeMetadata::default())?);
        }
        validate_builtin_attributes_for_target(&attributes, AttributeTarget::Constant, span)?;
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

    fn parse_const_declaration(
        &mut self,
        attributes: ParsedAttributes,
    ) -> Result<ConstDeclaration> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if is_exit_construct_name(name) {
                return Err(exit_reserved_name_syntax_error(self.peek(), "identifier"));
            }
        }
        let (name, token_span) = self.parse_declaration_name("expected constant name")?;
        if is_reserved_compile_time_constant_declaration_name(&name) {
            let source_name = self
                .source
                .get(token_span.byte_start..token_span.byte_end)
                .unwrap_or(&name);
            let local_name = source_name.rsplit('\\').next().unwrap_or(source_name);
            return Err(Diagnostic::new(
                format!("Cannot redeclare constant '{local_name}'"),
                Some(token_span),
            ));
        }
        self.declared_constants.insert(name.to_ascii_lowercase());
        self.expect_equal()?;
        let value = self.parse_const_context_expr()?;
        if !is_supported_const_declaration_expr(&value) {
            return Err(Diagnostic::new(
                "constant expression contains invalid operation",
                Some(value.span()),
            ));
        }
        Ok(ConstDeclaration {
            name,
            attributes,
            value,
            span: token_span,
        })
    }

    fn parse_if(&mut self) -> Result<Statement> {
        let span = self.expect_if_like()?;
        self.expect_left_paren()?;
        let condition = self.parse_expr()?;
        self.expect_right_paren()?;
        if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            return self.parse_alternative_if(span, condition);
        }
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

    fn parse_alternative_if(&mut self, span: SourceSpan, condition: Expr) -> Result<Statement> {
        let then_body = self.parse_alternative_statements_until(|parser| {
            matches!(parser.peek().kind, TokenKind::Elseif | TokenKind::Else)
                || parser.peek_is_identifier("endif")
        })?;
        let else_body = match self.peek().kind {
            TokenKind::Elseif => vec![self.parse_if()?],
            TokenKind::Else => {
                self.advance();
                self.expect_colon()?;
                let body = self.parse_alternative_statements_until(|parser| {
                    parser.peek_is_identifier("endif")
                })?;
                self.expect_alternative_end_keyword("endif")?;
                body
            }
            _ => {
                self.expect_alternative_end_keyword("endif")?;
                Vec::new()
            }
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
        let body = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            let body = self.parse_alternative_statements_until(|parser| {
                parser.peek_is_identifier("endwhile")
            })?;
            self.expect_alternative_end_keyword("endwhile")?;
            body
        } else {
            self.parse_statement_body()?
        };
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
        let body = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            let body = self
                .parse_alternative_statements_until(|parser| parser.peek_is_identifier("endfor"))?;
            self.expect_alternative_end_keyword("endfor")?;
            body
        } else {
            self.parse_statement_body()?
        };

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
        let body = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            let body = self.parse_alternative_statements_until(|parser| {
                parser.peek_is_identifier("endforeach")
            })?;
            self.expect_alternative_end_keyword("endforeach")?;
            body
        } else {
            self.parse_statement_body()?
        };
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
        let target_expr = self.parse_expr_allowing_append_array_read()?;
        let target_span = target_expr.span();
        let target = assignment_target_from_expr(target_expr).map_err(|diagnostic| {
            if diagnostic.message == "Cannot use empty list" {
                diagnostic
            } else {
                Diagnostic::new("expected foreach variable", Some(target_span))
            }
        })?;
        reject_append_array_read_in_assignment_target(&target)?;
        reject_this_foreach_target(&target)?;
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
        let alternative = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            true
        } else {
            self.expect_left_brace()?;
            false
        };

        let mut cases = Vec::new();
        let mut seen_default = false;
        while !self.peek_ends_switch(alternative) {
            self.skip_php_tags();
            if self.peek_ends_switch(alternative) {
                break;
            }
            let case_span = self.peek().span;
            let condition = match self.peek().kind {
                TokenKind::Case => {
                    self.advance();
                    let condition = self.parse_expr()?;
                    self.expect_switch_case_separator()?;
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
                    self.expect_switch_case_separator()?;
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
            while !self.peek_ends_switch_case(alternative) {
                self.skip_php_tags();
                if self.peek_ends_switch_case(alternative) {
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

        if alternative {
            self.expect_alternative_end_keyword("endswitch")?;
        } else {
            self.expect_right_brace()?;
        }
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

    fn parse_exit(&mut self) -> Result<Statement> {
        let start_span = self.expect_exit()?;
        let mut span = start_span;
        let value = if matches!(
            self.peek().kind,
            TokenKind::Semicolon | TokenKind::CloseTag | TokenKind::Eof
        ) {
            None
        } else if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.expect_left_paren()?;
            if matches!(self.peek().kind, TokenKind::RightParen) {
                let right_span = self.expect_right_paren()?;
                span = combine_spans(start_span, right_span);
                None
            } else {
                if token_is_identifier_named(self.peek(), "status")
                    && matches!(self.peek_next().kind, TokenKind::Colon)
                {
                    self.advance();
                    self.expect_colon()?;
                }
                let value = self.parse_expr()?;
                if matches!(self.peek().kind, TokenKind::Comma) {
                    return Err(Diagnostic::new(
                        "exit() expects at most 1 argument",
                        Some(self.peek().span),
                    ));
                }
                let right_span = self.expect_right_paren()?;
                span = combine_spans(start_span, right_span);
                Some(value)
            }
        } else {
            let value = self.parse_expr()?;
            span = combine_spans(start_span, value.span());
            Some(value)
        };
        self.expect_statement_terminator()?;
        Ok(Statement::Exit { value, span })
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
                "Cannot use try without catch or finally",
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
        let mut names = vec![self.parse_catch_type_name()?];
        while matches!(self.peek().kind, TokenKind::Pipe) {
            self.advance();
            names.push(self.parse_catch_type_name()?);
        }
        Ok(names)
    }

    fn parse_catch_type_name(&mut self) -> Result<String> {
        let (name, span) = self.parse_resolved_class_name("expected catch type name")?;
        if name.eq_ignore_ascii_case("static") {
            return Err(Diagnostic::new(
                "Bad class name in the catch statement",
                Some(span),
            ));
        }
        Ok(name)
    }

    fn parse_goto(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let span = token.span;
        let label = self.advance().clone();
        let TokenKind::Identifier(label_name) = &label.kind else {
            return Err(Diagnostic::new("expected goto label", Some(label.span)));
        };
        if is_exit_construct_name(label_name) {
            return Err(exit_reserved_name_syntax_error(&label, "identifier"));
        }
        let label = label_name.clone();
        self.expect_statement_terminator()?;
        Ok(Statement::Goto { label, span })
    }

    fn parse_label(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        let TokenKind::Identifier(name) = token.kind else {
            return Err(Diagnostic::new("expected label", Some(token.span)));
        };
        if is_exit_construct_name(&name) {
            return Err(syntax_error_unexpected(self.peek(), None));
        }
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

    fn parse_alternative_statements_until(
        &mut self,
        mut is_end: impl FnMut(&Self) -> bool,
    ) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        loop {
            self.skip_php_tags();
            if is_end(self) || matches!(self.peek().kind, TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_nested_statement()?);
        }
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

    fn expect_alternative_end_keyword(&mut self, expected: &str) -> Result<SourceSpan> {
        let token = self.advance().clone();
        if matches!(&token.kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case(expected))
        {
            self.expect_statement_terminator()?;
            Ok(token.span)
        } else {
            Err(Diagnostic::new(
                format!("expected {expected}"),
                Some(token.span),
            ))
        }
    }

    fn expect_switch_case_separator(&mut self) -> Result<SourceSpan> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Colon => Ok(token.span),
            TokenKind::Semicolon => {
                self.compile_warnings.push(CompileWarning {
                    message: "Case statements followed by a semicolon (;) are deprecated, use a colon (:) instead".to_string(),
                    span: token.span,
                    kind: CompileWarningKind::Deprecation,
                });
                Ok(token.span)
            }
            _ => Err(Diagnostic::new(
                "expected switch case separator",
                Some(token.span),
            )),
        }
    }

    fn peek_ends_switch(&self, alternative: bool) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
            || if alternative {
                self.peek_is_identifier("endswitch")
            } else {
                matches!(self.peek().kind, TokenKind::RightBrace)
            }
    }

    fn peek_ends_switch_case(&self, alternative: bool) -> bool {
        matches!(self.peek().kind, TokenKind::Case | TokenKind::Default)
            || self.peek_ends_switch(alternative)
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
            let span = token.span;
            match token.kind {
                TokenKind::Identifier(name)
                    if bracket_depth == 1 && paren_depth == 0 && collecting_name =>
                {
                    name_segments.push(name);
                }
                TokenKind::Variable(name)
                    if bracket_depth == 1 && paren_depth == 0 && collecting_name =>
                {
                    return Err(Diagnostic::parse_error(
                        format!("syntax error, unexpected variable \"${name}\""),
                        Some(span),
                    ));
                }
                TokenKind::Backslash
                    if bracket_depth == 1 && paren_depth == 0 && collecting_name => {}
                TokenKind::LeftParen if bracket_depth == 1 => {
                    collecting_name = false;
                    paren_depth += 1;
                }
                TokenKind::Ellipsis if bracket_depth == 1 && paren_depth > 0 => {
                    return Err(Diagnostic::new(
                        "Cannot use unpacking in attribute argument list",
                        Some(span),
                    ));
                }
                TokenKind::Variable(_) if bracket_depth == 1 && paren_depth > 0 => {
                    return Err(Diagnostic::new(
                        "Constant expression contains invalid operations",
                        Some(span),
                    ));
                }
                TokenKind::ObjectOperator if bracket_depth == 1 && paren_depth > 0 => {
                    return Err(Diagnostic::new(
                        "Dynamic class names are not allowed in compile-time class constant references",
                        Some(span),
                    ));
                }
                TokenKind::Identifier(name) if bracket_depth == 1 && paren_depth == 1 => {
                    let parsed_name = self.parse_name_from_first(
                        name,
                        span,
                        None,
                        "expected attribute argument",
                    )?;
                    if matches!(self.peek().kind, TokenKind::Colon) {
                        pending_argument_name = Some(parsed_name.name);
                    } else if matches!(self.peek().kind, TokenKind::LeftParen) {
                        return Err(Diagnostic::new(
                            "Constant expression contains invalid operations",
                            Some(parsed_name.span),
                        ));
                    } else if matches!(self.peek().kind, TokenKind::DoubleColon) {
                        self.advance();
                        let member = self.advance().clone();
                        let Some(member_name) = name_segment_from_token(&member.kind) else {
                            return Err(Diagnostic::new(
                                "expected class constant name",
                                Some(member.span),
                            ));
                        };
                        let class_name = self.resolve_class_name(&parsed_name);
                        let class_name = self.resolve_runtime_class_alias_name(&class_name);
                        let text = format!("{class_name}::{member_name}");
                        let kind = parsed_attribute_class_constant_kind(&class_name, &member_name);
                        arguments.record_value(
                            pending_argument_name.take(),
                            ParsedAttributeArgumentValue {
                                text,
                                kind,
                                span: parsed_name.span,
                                constant_reference: Some(
                                    AttributeConstantReference::ClassConstant {
                                        class_name,
                                        name: member_name,
                                    },
                                ),
                            },
                        );
                    } else {
                        let name = self.resolve_constant_name(&parsed_name);
                        arguments.record_value(
                            pending_argument_name.take(),
                            ParsedAttributeArgumentValue {
                                text: name.clone(),
                                kind: ParsedAttributeArgumentKind::Constant,
                                span: parsed_name.span,
                                constant_reference: Some(AttributeConstantReference::Constant(
                                    name,
                                )),
                            },
                        );
                    }
                }
                TokenKind::Backslash if bracket_depth == 1 && paren_depth == 1 => {
                    let first_token = self.advance().clone();
                    let Some(first_segment) = name_segment_from_token(&first_token.kind) else {
                        return Err(Diagnostic::new(
                            "expected attribute argument",
                            Some(first_token.span),
                        ));
                    };
                    let parsed_name = self.parse_name_from_first(
                        first_segment,
                        first_token.span,
                        Some(span),
                        "expected attribute argument",
                    )?;
                    if matches!(self.peek().kind, TokenKind::LeftParen) {
                        return Err(Diagnostic::new(
                            "Constant expression contains invalid operations",
                            Some(parsed_name.span),
                        ));
                    } else if matches!(self.peek().kind, TokenKind::DoubleColon) {
                        self.advance();
                        let member = self.advance().clone();
                        let Some(member_name) = name_segment_from_token(&member.kind) else {
                            return Err(Diagnostic::new(
                                "expected class constant name",
                                Some(member.span),
                            ));
                        };
                        let class_name = self.resolve_class_name(&parsed_name);
                        let class_name = self.resolve_runtime_class_alias_name(&class_name);
                        let text = format!("{class_name}::{member_name}");
                        let kind = parsed_attribute_class_constant_kind(&class_name, &member_name);
                        arguments.record_value(
                            pending_argument_name.take(),
                            ParsedAttributeArgumentValue {
                                text,
                                kind,
                                span: parsed_name.span,
                                constant_reference: Some(
                                    AttributeConstantReference::ClassConstant {
                                        class_name,
                                        name: member_name,
                                    },
                                ),
                            },
                        );
                    } else {
                        let name = self.resolve_constant_name(&parsed_name);
                        arguments.record_value(
                            pending_argument_name.take(),
                            ParsedAttributeArgumentValue {
                                text: name.clone(),
                                kind: ParsedAttributeArgumentKind::Constant,
                                span: parsed_name.span,
                                constant_reference: Some(AttributeConstantReference::Constant(
                                    name,
                                )),
                            },
                        );
                    }
                }
                TokenKind::String(value) if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        value,
                        ParsedAttributeArgumentKind::String,
                        span,
                    );
                }
                TokenKind::Int(value) if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        value.to_string(),
                        ParsedAttributeArgumentKind::Int,
                        span,
                    );
                }
                TokenKind::Float(value) if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        value.to_string(),
                        ParsedAttributeArgumentKind::Float,
                        span,
                    );
                }
                TokenKind::True if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        "1".to_string(),
                        ParsedAttributeArgumentKind::Bool,
                        span,
                    );
                }
                TokenKind::False if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        String::new(),
                        ParsedAttributeArgumentKind::Bool,
                        span,
                    );
                }
                TokenKind::Null if bracket_depth == 1 && paren_depth == 1 => {
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        String::new(),
                        ParsedAttributeArgumentKind::Null,
                        span,
                    );
                }
                TokenKind::LeftBracket if bracket_depth == 1 && paren_depth == 1 => {
                    if matches!(self.peek().kind, TokenKind::Comma) {
                        return Err(Diagnostic::new(
                            "Cannot use empty array elements in arrays",
                            Some(span),
                        ));
                    }
                    arguments.record_typed_text(
                        pending_argument_name.take(),
                        "Array".to_string(),
                        ParsedAttributeArgumentKind::Array,
                        span,
                    );
                    bracket_depth += 1;
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
                    apply_parsed_attribute(
                        &mut attributes,
                        &name_segments,
                        &arguments,
                        self.strict_types,
                    )?;
                    name_segments.clear();
                    arguments = ParsedAttributeArguments::default();
                    pending_argument_name = None;
                    collecting_name = true;
                }
                TokenKind::LeftBracket => {
                    if paren_depth > 0 && matches!(self.peek().kind, TokenKind::Comma) {
                        return Err(Diagnostic::new(
                            "Cannot use empty array elements in arrays",
                            Some(span),
                        ));
                    }
                    bracket_depth += 1;
                }
                TokenKind::RightBracket => {
                    if bracket_depth == 1 && paren_depth == 0 {
                        apply_parsed_attribute(
                            &mut attributes,
                            &name_segments,
                            &arguments,
                            self.strict_types,
                        )?;
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

    fn parse_const_context_expr(&mut self) -> Result<Expr> {
        self.parse_expr().map_err(|error| {
            if error.message == CLASS_CONSTANT_FETCH_UNSUPPORTED {
                Diagnostic::new(
                    "Dynamic class names are not allowed in compile-time class constant references",
                    error.span,
                )
            } else {
                error
            }
        })
    }

    fn parse_expr_allowing_append_array_read(&mut self) -> Result<Expr> {
        let previous_allow_append_array_read = self.allow_append_array_read;
        self.allow_append_array_read = true;
        let expr = self.parse_expr();
        self.allow_append_array_read = previous_allow_append_array_read;
        expr
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
            Err(diagnostic)
                if diagnostic.message == "Can't use function return value in write context" =>
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
        reject_append_array_read_in_assignment_target(&target)?;
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
                    let (name, span, source_name) =
                        self.parse_resolved_class_name_with_source("expected class name")?;
                    InstanceOfTarget::ClassName {
                        name,
                        source_name,
                        span,
                    }
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
            let right = if self.peek_is_expression_assignment_op() {
                self.parse_assignment_expr_from_left(right)?
            } else {
                right
            };
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
        let expr = self.parse_primary_expr()?;
        self.parse_postfix_expr_from(expr, true)
    }

    fn parse_postfix_expr_from(&mut self, mut expr: Expr, allow_calls: bool) -> Result<Expr> {
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
                    if !matches!(self.peek().kind, TokenKind::LeftParen) || !allow_calls {
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
                    if !allow_calls {
                        break;
                    }
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
                        _ if name_segment_from_token(&member.kind).is_some() => {
                            let member_name = name_segment_from_token(&member.kind)
                                .expect("name segment checked above");
                            if member_name.eq_ignore_ascii_case("class")
                                && !matches!(self.peek().kind, TokenKind::LeftParen)
                            {
                                if let Some(type_name) =
                                    constant_expression_class_name_fetch_error_type(&expr)
                                {
                                    return Err(Diagnostic::new(
                                        format!("Cannot use \"::class\" on {type_name}"),
                                        Some(start_span),
                                    ));
                                }
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
                        if let Some(Expr::Variable(property_name, property_span)) =
                            dynamic_name.as_ref()
                        {
                            expr = Expr::DynamicStaticPropertyFetch {
                                receiver: Box::new(expr),
                                name: property_name.clone(),
                                span: combine_spans(start_span, *property_span),
                            };
                            continue;
                        }
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
                    let class_name = self.resolve_runtime_class_alias_name(&class_name);
                    self.parse_static_member_expr(class_name, parsed_name.span)
                } else if matches!(self.peek().kind, TokenKind::LeftParen) {
                    if unqualified && lowercase == "__halt_compiler" {
                        return Err(Diagnostic::new(
                            "__HALT_COMPILER() can only be used from the outermost scope",
                            Some(parsed_name.span),
                        ));
                    }
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
                    if lowercase == "__compiler_halt_offset__" {
                        if let Some(offset) = self.compiler_halt_offset {
                            Ok(Expr::Int(offset, parsed_name.span))
                        } else {
                            Ok(Expr::Constant(
                                self.resolve_constant_name(&parsed_name),
                                parsed_name.span,
                            ))
                        }
                    } else if let Some(kind) = magic_constant_kind(&parsed_name.name) {
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
                let Some(first_segment) = name_segment_from_token(&first_token.kind) else {
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
                if parsed_name
                    .name
                    .eq_ignore_ascii_case("__compiler_halt_offset__")
                    && !matches!(
                        self.peek().kind,
                        TokenKind::DoubleColon | TokenKind::LeftParen
                    )
                {
                    if let Some(offset) = self.compiler_halt_offset {
                        return Ok(Expr::Int(offset, parsed_name.span));
                    }
                    return Ok(Expr::Constant(
                        self.resolve_constant_name(&parsed_name),
                        parsed_name.span,
                    ));
                }
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    let class_name = self.resolve_class_name(&parsed_name);
                    return self.parse_static_member_expr(
                        self.resolve_runtime_class_alias_name(&class_name),
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
            if self.return_by_ref_stack.last().copied().unwrap_or(false) {
                return Err(Diagnostic::new(
                    "Cannot use \"yield from\" inside a by-reference generator",
                    Some(self.peek().span),
                ));
            }
            self.advance();
            let expr = self.parse_expr()?;
            let span = combine_spans(start_span, expr.span());
            return Ok(Expr::YieldFrom {
                expr: Box::new(expr),
                span,
            });
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
        let Some(member_name) = name_segment_from_token(&member.kind) else {
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
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            let token = self.advance().clone();
            let TokenKind::Variable(name) = token.kind else {
                unreachable!("variable token was just matched")
            };
            let class_name =
                self.parse_postfix_expr_from(Expr::Variable(name, token.span), false)?;
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
        if matches!(self.peek().kind, TokenKind::Dollar | TokenKind::LeftParen) {
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

        let (class_name, class_span, source_name) = self.parse_new_object_class_name()?;
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
            source_name,
            arguments,
            argument_names,
            argument_unpacks,
            anonymous_class_source: None,
            span,
        })
    }

    fn parse_new_object_class_name(&mut self) -> Result<(String, SourceSpan, String)> {
        self.parse_resolved_class_name_with_source("expected class name")
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
                ParsedClassMember::Method(method) => {
                    if method.is_abstract {
                        return Err(Diagnostic::new(
                            format!(
                                "Anonymous class method {}() must not be abstract",
                                method.name
                            ),
                            Some(method.span),
                        ));
                    }
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
            source_name: class_name.clone(),
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

        let first = self.parse_expr_allowing_append_array_read()?;
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
        Ok(ListExprElementTarget::Value(
            self.parse_expr_allowing_append_array_read()?,
        ))
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

        let first = self.parse_expr_allowing_append_array_read()?;
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
        Ok(ArrayElementValue::Value(
            self.parse_expr_allowing_append_array_read()?,
        ))
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

    fn peek_starts_promoted_parameter_modifier(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokenKind::Identifier(name)
                if name.eq_ignore_ascii_case("public")
                    || name.eq_ignore_ascii_case("protected")
                    || name.eq_ignore_ascii_case("private")
                    || name.eq_ignore_ascii_case("readonly")
                    || name.eq_ignore_ascii_case("final")
        )
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

    fn expect_exit(&mut self) -> Result<SourceSpan> {
        let token = self.advance();
        if matches!(&token.kind, TokenKind::Identifier(name) if is_exit_construct_name(name)) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("expected exit", Some(token.span)))
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

fn property_type_hint_from_type_hint(type_hint: &TypeHint) -> PropertyTypeHint {
    let mut property_type_hint = match type_hint {
        TypeHint::Null => PropertyTypeHint {
            text: "null".to_string(),
            kind: PropertyTypeKind::Null,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Array => PropertyTypeHint {
            text: "array".to_string(),
            kind: PropertyTypeKind::Array,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Int => PropertyTypeHint {
            text: "int".to_string(),
            kind: PropertyTypeKind::Int,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Float => PropertyTypeHint {
            text: "float".to_string(),
            kind: PropertyTypeKind::Float,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::String => PropertyTypeHint {
            text: "string".to_string(),
            kind: PropertyTypeKind::String,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Bool => PropertyTypeHint {
            text: "bool".to_string(),
            kind: PropertyTypeKind::Bool,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::True | TypeHint::False | TypeHint::Static => PropertyTypeHint {
            text: type_hint_display(type_hint),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Mixed => PropertyTypeHint {
            text: "mixed".to_string(),
            kind: PropertyTypeKind::Mixed,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Object => PropertyTypeHint {
            text: "object".to_string(),
            kind: PropertyTypeKind::Object,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Callable
        | TypeHint::Iterable
        | TypeHint::Union(_)
        | TypeHint::Intersection(_) => PropertyTypeHint {
            text: type_hint_display_canonical(type_hint),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Void | TypeHint::Never => PropertyTypeHint {
            text: type_hint_display(type_hint),
            kind: PropertyTypeKind::Unsupported,
            allows_null: false,
            semantic_type: None,
        },
        TypeHint::Nullable(inner) => {
            let mut nested = property_type_hint_from_type_hint(inner);
            nested.text = format!("?{}", nested.text);
            nested.allows_null = true;
            nested.semantic_type = None;
            nested
        }
        TypeHint::Class(name) => PropertyTypeHint {
            text: name.clone(),
            kind: PropertyTypeKind::Class(name.clone()),
            allows_null: false,
            semantic_type: None,
        },
    };
    property_type_hint.semantic_type = Some(type_hint.clone());
    property_type_hint
}

fn validate_parameter_type_hint(type_hint: &TypeHint, span: SourceSpan) -> Result<()> {
    match type_hint {
        TypeHint::Void => Err(Diagnostic::new(
            "void cannot be used as a parameter type",
            Some(span),
        )),
        TypeHint::Never => Err(Diagnostic::new(
            "never cannot be used as a parameter type",
            Some(span),
        )),
        TypeHint::Nullable(inner) => validate_parameter_type_hint(inner, span),
        TypeHint::Union(types) | TypeHint::Intersection(types) => {
            for type_hint in types {
                validate_parameter_type_hint(type_hint, span)?;
            }
            Ok(())
        }
        _ => Ok(()),
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
            "Void can only be used as a standalone type",
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

fn name_segment_from_token(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Identifier(name) => Some(name.clone()),
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
        | TokenKind::BooleanType => Some(token_text(kind).to_string()),
        _ => None,
    }
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
        Expr::DynamicStaticPropertyFetch { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
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
        Expr::YieldFrom { expr, .. } => {
            collect_arrow_captures_from_expr(expr, exclusions, seen, captures);
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
        AssignmentTarget::DynamicStaticPropertyArrayDim {
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
        AssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            collect_arrow_captures_from_expr(array, exclusions, seen, captures);
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
        AssignmentTarget::DynamicStaticProperty { receiver, .. } => {
            collect_arrow_captures_from_expr(receiver, exclusions, seen, captures);
        }
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
        ReferenceTarget::DynamicVariable { name, .. } => {
            collect_arrow_captures_from_expr(name, exclusions, seen, captures);
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
        StringPart::PropertyFetch { variable, .. } | StringPart::PropertyChain { variable, .. } => {
            add_arrow_capture(
                variable,
                SourceSpan::new(0, 0, 0, 0),
                exclusions,
                seen,
                captures,
            );
        }
        StringPart::MethodCall { variable, .. } => {
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

#[derive(Clone)]
struct ParsedAttributeArgumentValue {
    text: String,
    kind: ParsedAttributeArgumentKind,
    span: SourceSpan,
    constant_reference: Option<AttributeConstantReference>,
}

#[derive(Clone)]
enum ParsedAttributeArgumentKind {
    String,
    Int,
    Float,
    Bool,
    Null,
    Array,
    Constant,
    ClassConstant,
    NativeEnumCase {
        class_name: String,
        case_name: String,
    },
}

#[derive(Default)]
struct ParsedAttributeArguments {
    positional: Vec<ParsedAttributeArgumentValue>,
    message: Option<ParsedAttributeArgumentValue>,
    since: Option<ParsedAttributeArgumentValue>,
}

impl ParsedAttributeArguments {
    fn record_typed_text(
        &mut self,
        name: Option<String>,
        value: String,
        kind: ParsedAttributeArgumentKind,
        span: SourceSpan,
    ) {
        self.record_value(
            name,
            ParsedAttributeArgumentValue {
                text: value,
                kind,
                span,
                constant_reference: None,
            },
        );
    }

    fn record_value(&mut self, name: Option<String>, value: ParsedAttributeArgumentValue) {
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

    fn message_value(&self) -> Option<&ParsedAttributeArgumentValue> {
        self.message.as_ref().or_else(|| self.positional.first())
    }

    fn message(&self) -> Option<String> {
        self.message_value().map(|value| value.text.clone())
    }

    fn message_constant_reference(&self) -> Option<AttributeConstantReference> {
        self.message_value()
            .and_then(|value| value.constant_reference.clone())
    }

    fn since_value(&self) -> Option<&ParsedAttributeArgumentValue> {
        self.since.as_ref().or_else(|| self.positional.get(1))
    }

    fn since(&self) -> Option<String> {
        self.since_value().map(|value| value.text.clone())
    }
}

fn parsed_attribute_class_constant_kind(
    class_name: &str,
    member_name: &str,
) -> ParsedAttributeArgumentKind {
    if member_name.eq_ignore_ascii_case("class") {
        ParsedAttributeArgumentKind::String
    } else if is_known_internal_enum_class(class_name) {
        ParsedAttributeArgumentKind::NativeEnumCase {
            class_name: class_name.trim_start_matches('\\').to_string(),
            case_name: member_name.to_string(),
        }
    } else {
        ParsedAttributeArgumentKind::ClassConstant
    }
}

fn is_known_internal_enum_class(class_name: &str) -> bool {
    matches!(
        class_name
            .trim_start_matches('\\')
            .to_ascii_lowercase()
            .as_str(),
        "random\\intervalboundary" | "roundingmode"
    )
}

fn validate_builtin_attribute_nullable_string_argument(
    attribute_name: &str,
    position: usize,
    parameter_name: &str,
    value: Option<&ParsedAttributeArgumentValue>,
    strict_types: bool,
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(given_type) = attribute_argument_type_error_given(value, strict_types) else {
        return Ok(());
    };
    Err(Diagnostic::uncaught_fatal(
        "TypeError",
        format!(
            "{attribute_name}::__construct(): Argument #{position} (${parameter_name}) must be of type ?string, {given_type} given"
        ),
        Some(value.span),
        Some(format!(
            "{attribute_name}->__construct({})",
            attribute_argument_stack_display(value)
        )),
    ))
}

fn attribute_argument_type_error_given(
    value: &ParsedAttributeArgumentValue,
    strict_types: bool,
) -> Option<String> {
    match &value.kind {
        ParsedAttributeArgumentKind::Array => Some("array".to_string()),
        ParsedAttributeArgumentKind::NativeEnumCase { class_name, .. } => Some(class_name.clone()),
        ParsedAttributeArgumentKind::Int if strict_types => Some("int".to_string()),
        ParsedAttributeArgumentKind::Float if strict_types => Some("float".to_string()),
        ParsedAttributeArgumentKind::Bool if strict_types => Some("bool".to_string()),
        _ => None,
    }
}

fn attribute_argument_stack_display(value: &ParsedAttributeArgumentValue) -> String {
    match &value.kind {
        ParsedAttributeArgumentKind::Array => "Array".to_string(),
        ParsedAttributeArgumentKind::NativeEnumCase {
            class_name,
            case_name,
        } => format!("{class_name}::{case_name}"),
        ParsedAttributeArgumentKind::Bool if value.text.is_empty() => "false".to_string(),
        ParsedAttributeArgumentKind::Bool => "true".to_string(),
        ParsedAttributeArgumentKind::Null => "NULL".to_string(),
        _ => value.text.clone(),
    }
}

fn apply_parsed_attribute(
    attributes: &mut AttributeMetadata,
    name_segments: &[String],
    arguments: &ParsedAttributeArguments,
    strict_types: bool,
) -> Result<()> {
    if name_segments.len() != 1 {
        attributes.total_count = attributes.total_count.saturating_add(1);
        return Ok(());
    }
    attributes.total_count = attributes.total_count.saturating_add(1);
    let name = &name_segments[0];
    if name.eq_ignore_ascii_case("Override") {
        attributes.override_count = attributes.override_count.saturating_add(1);
        attributes.has_override = true;
    } else if name.eq_ignore_ascii_case("Attribute") {
        attributes.attribute_count = attributes.attribute_count.saturating_add(1);
        attributes.has_attribute = true;
    } else if name.eq_ignore_ascii_case("AllowDynamicProperties") {
        attributes.allow_dynamic_properties_count =
            attributes.allow_dynamic_properties_count.saturating_add(1);
        attributes.has_allow_dynamic_properties = true;
    } else if name.eq_ignore_ascii_case("DelayedTargetValidation") {
        attributes.delayed_target_validation_count =
            attributes.delayed_target_validation_count.saturating_add(1);
    } else if name.eq_ignore_ascii_case("Deprecated") {
        attributes.deprecated_count = attributes.deprecated_count.saturating_add(1);
        validate_builtin_attribute_nullable_string_argument(
            "Deprecated",
            1,
            "message",
            arguments.message_value(),
            strict_types,
        )?;
        validate_builtin_attribute_nullable_string_argument(
            "Deprecated",
            2,
            "since",
            arguments.since_value(),
            strict_types,
        )?;
        attributes.deprecated_message = Some(arguments.message().unwrap_or_default());
        attributes.deprecated_message_constant = arguments.message_constant_reference();
        attributes.deprecated_since = arguments.since();
    } else if name.eq_ignore_ascii_case("NoDiscard") {
        attributes.no_discard_count = attributes.no_discard_count.saturating_add(1);
        validate_builtin_attribute_nullable_string_argument(
            "NoDiscard",
            1,
            "message",
            arguments.message_value(),
            strict_types,
        )?;
        attributes.no_discard_message = Some(arguments.message().unwrap_or_default());
    } else if name.eq_ignore_ascii_case("ReturnTypeWillChange") {
        attributes.return_type_will_change_count =
            attributes.return_type_will_change_count.saturating_add(1);
    }
    Ok(())
}

fn merge_parsed_attributes(attributes: &mut AttributeMetadata, group: AttributeMetadata) {
    attributes.total_count = attributes.total_count.saturating_add(group.total_count);
    attributes.attribute_count = attributes
        .attribute_count
        .saturating_add(group.attribute_count);
    attributes.allow_dynamic_properties_count = attributes
        .allow_dynamic_properties_count
        .saturating_add(group.allow_dynamic_properties_count);
    attributes.delayed_target_validation_count = attributes
        .delayed_target_validation_count
        .saturating_add(group.delayed_target_validation_count);
    attributes.deprecated_count = attributes
        .deprecated_count
        .saturating_add(group.deprecated_count);
    attributes.no_discard_count = attributes
        .no_discard_count
        .saturating_add(group.no_discard_count);
    attributes.override_count = attributes
        .override_count
        .saturating_add(group.override_count);
    attributes.return_type_will_change_count = attributes
        .return_type_will_change_count
        .saturating_add(group.return_type_will_change_count);
    attributes.has_override |= group.has_override;
    attributes.has_attribute |= group.has_attribute;
    attributes.has_allow_dynamic_properties |= group.has_allow_dynamic_properties;
    if group.deprecated_message.is_some() {
        attributes.deprecated_message = group.deprecated_message;
        attributes.deprecated_message_constant = group.deprecated_message_constant;
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

fn is_modeled_builtin_date_class_name(name: &str) -> bool {
    matches!(
        name.trim_start_matches('\\').to_ascii_lowercase().as_str(),
        "datetime" | "datetimeimmutable" | "datetimezone"
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
                is_final: promoted.is_final,
                is_readonly: class_is_readonly || promoted.is_readonly,
                has_hooks: false,
                is_virtual: false,
                hook_get_value: None,
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
            let target = AssignmentTarget::Property {
                receiver: Box::new(Expr::Variable("this".to_string(), span)),
                name: parameter.name.clone(),
                span,
            };
            if parameter.by_ref {
                return Statement::Expression {
                    expression: Expr::AssignRef {
                        target,
                        source: Box::new(Expr::Variable(parameter.name.clone(), span)),
                        span,
                    },
                    span,
                };
            }
            Statement::Expression {
                expression: Expr::Assign {
                    target,
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

fn is_exit_construct_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("exit") || name.eq_ignore_ascii_case("die")
}

fn exit_reserved_name_syntax_error(token: &Token, expecting: &str) -> Diagnostic {
    Diagnostic::parse_error(
        format!("syntax error, unexpected token \"exit\", expecting {expecting}"),
        Some(token.span),
    )
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
        import_trait_members_into_trait(&mut composed, &used_trait, trait_use)?;
    }
    visiting.remove(&lookup_name);
    cache.insert(lookup_name, composed.clone());
    Ok(composed)
}

fn import_trait_members_into_trait(
    target: &mut TraitDecl,
    source: &TraitDecl,
    trait_use: &TraitUseDecl,
) -> Result<()> {
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
        import_trait_method_into_method_list(&mut target.methods, source, trait_use, method)?;
    }
    Ok(())
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
                &trait_use,
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
    trait_use: &TraitUseDecl,
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
        import_trait_method_into_class(
            class,
            trait_decl,
            trait_use,
            method,
            own_method_names,
            imported_method_names,
        )?;
    }
    Ok(())
}

fn import_trait_method_into_method_list(
    methods: &mut Vec<MethodDecl>,
    trait_decl: &TraitDecl,
    trait_use: &TraitUseDecl,
    method: &MethodDecl,
) -> Result<()> {
    let own_method_names = methods
        .iter()
        .map(|candidate| candidate.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let mut imported_method_names = own_method_names.clone();
    let mut pseudo_class = ClassDecl {
        name: trait_decl.name.clone(),
        parent_name: None,
        interfaces: Vec::new(),
        trait_uses: Vec::new(),
        attributes: AttributeMetadata::default(),
        is_abstract: false,
        is_final: false,
        is_interface: false,
        is_readonly: false,
        properties: Vec::new(),
        static_properties: Vec::new(),
        constants: Vec::new(),
        methods: methods.clone(),
        span: method.span,
    };
    import_trait_method_into_class(
        &mut pseudo_class,
        trait_decl,
        trait_use,
        method,
        &own_method_names,
        &mut imported_method_names,
    )?;
    *methods = pseudo_class.methods;
    Ok(())
}

fn import_trait_method_into_class(
    class: &mut ClassDecl,
    trait_decl: &TraitDecl,
    trait_use: &TraitUseDecl,
    method: &MethodDecl,
    own_method_names: &HashSet<String>,
    imported_method_names: &mut HashSet<String>,
) -> Result<()> {
    let original_excluded =
        trait_method_excluded_by_precedence(&trait_use.adaptations, &trait_decl.name, &method.name);
    let method_key = method.name.to_ascii_lowercase();
    if !original_excluded && !own_method_names.contains(&method_key) {
        let imported = adapted_original_trait_method(method, trait_decl, &trait_use.adaptations);
        if !imported_method_names.insert(method_key) {
            return Err(Diagnostic::new(
                format!(
                    "Trait method {}::{} has not been applied because of a collision",
                    trait_decl.name, method.name
                ),
                Some(class.span),
            ));
        }
        class.methods.push(imported);
    }

    for alias in matching_trait_aliases(&trait_use.adaptations, &trait_decl.name, &method.name) {
        let Some(alias_name) = alias.alias.as_ref() else {
            continue;
        };
        let alias_key = alias_name.to_ascii_lowercase();
        if own_method_names.contains(&alias_key) {
            continue;
        }
        if !imported_method_names.insert(alias_key) {
            return Err(Diagnostic::new(
                format!(
                    "Trait method {}::{} has not been applied because of a collision",
                    trait_decl.name, alias_name
                ),
                Some(alias.span),
            ));
        }
        let mut imported = method_with_trait_origin(method, trait_decl);
        imported.name = alias_name.clone();
        if let Some(visibility) = alias.visibility {
            imported.visibility = visibility;
        }
        class.methods.push(imported);
    }
    Ok(())
}

fn adapted_original_trait_method(
    method: &MethodDecl,
    trait_decl: &TraitDecl,
    adaptations: &[TraitAdaptation],
) -> MethodDecl {
    let mut imported = method_with_trait_origin(method, trait_decl);
    for alias in matching_trait_aliases(adaptations, &trait_decl.name, &method.name) {
        if alias.alias.is_none() {
            if let Some(visibility) = alias.visibility {
                imported.visibility = visibility;
            }
        }
    }
    imported
}

fn matching_trait_aliases<'a>(
    adaptations: &'a [TraitAdaptation],
    trait_name: &str,
    method_name: &str,
) -> Vec<&'a TraitAliasAdaptation> {
    adaptations
        .iter()
        .filter_map(|adaptation| match adaptation {
            TraitAdaptation::Alias(alias)
                if trait_method_reference_matches(&alias.method, trait_name, method_name) =>
            {
                Some(alias)
            }
            _ => None,
        })
        .collect()
}

fn trait_method_excluded_by_precedence(
    adaptations: &[TraitAdaptation],
    trait_name: &str,
    method_name: &str,
) -> bool {
    adaptations.iter().any(|adaptation| {
        let TraitAdaptation::Precedence(precedence) = adaptation else {
            return false;
        };
        precedence
            .method
            .method_name
            .eq_ignore_ascii_case(method_name)
            && precedence
                .instead_of
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(trait_name))
    })
}

fn trait_method_reference_matches(
    reference: &TraitMethodReference,
    trait_name: &str,
    method_name: &str,
) -> bool {
    reference.method_name.eq_ignore_ascii_case(method_name)
        && reference
            .trait_name
            .as_deref()
            .is_none_or(|reference_trait| reference_trait.eq_ignore_ascii_case(trait_name))
}

fn method_with_trait_origin(method: &MethodDecl, trait_decl: &TraitDecl) -> MethodDecl {
    let mut imported = method.clone();
    if imported.trait_name.is_none() {
        imported.trait_name = Some(trait_decl.name.clone());
    }
    if imported.trait_method_name.is_none() {
        imported.trait_method_name = Some(method.name.clone());
    }
    imported
}

fn validate_class_names(classes: &[ClassDecl], traits: &[TraitDecl]) -> Result<()> {
    let mut names = HashSet::new();
    for class in classes {
        let lookup_name = class.name.to_ascii_lowercase();
        if is_reserved_class_name(&lookup_name) {
            return Err(Diagnostic::new(
                format!(
                    "Cannot use \"{}\" as a class name as it is reserved",
                    class.name
                ),
                Some(class.span),
            ));
        }
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

fn is_reserved_class_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("mixed")
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
            || is_modeled_builtin_date_class_name(parent_name)
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

fn validate_interface_references(
    classes: &[ClassDecl],
    compile_warnings: &mut Vec<CompileWarning>,
) -> Result<()> {
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
                compile_warnings.push(CompileWarning {
                    message: format!("Interface \"{interface_name}\" not found"),
                    span: class.span,
                    kind: CompileWarningKind::UncaughtError,
                });
                return Ok(());
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

fn validate_interface_method_conflicts(classes: &[ClassDecl]) -> Result<()> {
    for interface in classes.iter().filter(|class| class.is_interface) {
        let mut seen_interfaces = HashSet::new();
        let mut inherited_methods = Vec::new();
        for parent_name in &interface.interfaces {
            collect_interface_hierarchy_methods(
                parent_name,
                classes,
                &mut seen_interfaces,
                &mut inherited_methods,
            );
        }
        inherited_methods.extend(interface.methods.iter().map(|method| (interface, method)));
        for (index, (left_interface, left_method)) in inherited_methods.iter().enumerate() {
            for (right_interface, right_method) in inherited_methods.iter().skip(index + 1) {
                if !left_method.name.eq_ignore_ascii_case(&right_method.name)
                    || left_method.is_static == right_method.is_static
                {
                    continue;
                }
                let (non_static_interface, static_interface, static_method) =
                    if left_method.is_static {
                        (*right_interface, *left_interface, *left_method)
                    } else {
                        (*left_interface, *right_interface, *right_method)
                    };
                return Err(Diagnostic::new(
                    format!(
                        "Cannot make non static method {}::{}() static in class {}",
                        non_static_interface.name, static_method.name, static_interface.name
                    ),
                    Some(static_method.span),
                ));
            }
        }
    }
    Ok(())
}

fn collect_interface_hierarchy_methods<'a>(
    interface_name: &str,
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
    methods.extend(interface.methods.iter().map(|method| (interface, method)));
    for parent_name in &interface.interfaces {
        collect_interface_hierarchy_methods(parent_name, classes, seen, methods);
    }
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

fn validate_method_signature_compatibility(
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
    compile_warnings: &mut Vec<CompileWarning>,
) -> Result<()> {
    for class in classes {
        for method in &class.methods {
            if method.visibility == PropertyVisibility::Private {
                continue;
            }
            let has_visible_parent_method = if let Some((parent_class, parent_method)) =
                find_visible_parent_method(class, &method.name, classes)
            {
                if method_requires_parent_signature_compatibility(method, parent_method) {
                    validate_method_signature_pair(
                        class,
                        method,
                        parent_class,
                        parent_method,
                        classes,
                        runtime_class_aliases,
                    )?;
                }
                true
            } else {
                false
            };

            if !has_visible_parent_method {
                if let Some(tentative_method) =
                    find_tentative_internal_parent_method(class, &method.name, classes)
                {
                    validate_tentative_internal_return_signature(
                        class,
                        method,
                        &tentative_method,
                        classes,
                        runtime_class_aliases,
                        compile_warnings,
                    )?;
                }
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
                    runtime_class_aliases,
                )?;
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
struct TentativeInternalMethod {
    class_name: &'static str,
    signature: &'static str,
    return_type: TypeHint,
    is_static: bool,
}

fn find_tentative_internal_parent_method(
    class: &ClassDecl,
    method_name: &str,
    classes: &[ClassDecl],
) -> Option<TentativeInternalMethod> {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        let normalized = name.trim_start_matches('\\').to_ascii_lowercase();
        if !seen.insert(normalized.clone()) {
            break;
        }
        if let Some(method) = tentative_internal_method(&normalized, method_name) {
            return Some(method);
        }
        parent_name = find_class(classes, name).and_then(|parent| parent.parent_name.as_deref());
    }
    None
}

fn tentative_internal_method(
    normalized_class_name: &str,
    method_name: &str,
) -> Option<TentativeInternalMethod> {
    match (normalized_class_name, method_name.to_ascii_lowercase().as_str()) {
        ("datetimezone", "listidentifiers") => Some(TentativeInternalMethod {
            class_name: "DateTimeZone",
            signature:
                "listIdentifiers(int $timezoneGroup = DateTimeZone::ALL, ?string $countryCode = null): array",
            return_type: TypeHint::Array,
            is_static: true,
        }),
        ("datetime", "createfromformat") => Some(TentativeInternalMethod {
            class_name: "DateTime",
            signature:
                "createFromFormat(string $format, string $datetime, ?DateTimeZone $timezone = null): DateTime|false",
            return_type: TypeHint::Union(vec![
                TypeHint::Class("DateTime".to_string()),
                TypeHint::False,
            ]),
            is_static: true,
        }),
        ("datetime", "modify") => Some(TentativeInternalMethod {
            class_name: "DateTime",
            signature: "modify(string $modifier): DateTime|false",
            return_type: TypeHint::Union(vec![
                TypeHint::Class("DateTime".to_string()),
                TypeHint::False,
            ]),
            is_static: false,
        }),
        _ => None,
    }
}

fn validate_tentative_internal_return_signature(
    class: &ClassDecl,
    method: &MethodDecl,
    tentative_method: &TentativeInternalMethod,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
    compile_warnings: &mut Vec<CompileWarning>,
) -> Result<()> {
    if method.is_static != tentative_method.is_static {
        return Ok(());
    }
    if let Some(unavailable_name) = method
        .parameters
        .iter()
        .filter_map(|parameter| parameter.type_hint.as_ref())
        .find_map(|type_hint| first_unavailable_class_name(type_hint, classes))
    {
        return Err(
            method_signature_unresolved_compatibility_error_with_display(
                class,
                method,
                tentative_method.class_name,
                tentative_method.signature,
                &unavailable_name,
            ),
        );
    }
    if method.return_type.as_ref().is_some_and(|return_type| {
        type_hint_is_subtype(
            return_type,
            &tentative_method.return_type,
            classes,
            runtime_class_aliases,
        )
    }) {
        return Ok(());
    }
    if method.attributes.return_type_will_change_count > 0 {
        return Ok(());
    }
    if let Some(return_type) = method.return_type.as_ref() {
        if let Some(unavailable_name) = first_unavailable_class_name(return_type, classes) {
            return Err(
                method_signature_unresolved_compatibility_error_with_display(
                    class,
                    method,
                    tentative_method.class_name,
                    tentative_method.signature,
                    &unavailable_name,
                ),
            );
        }
    }
    compile_warnings.push(CompileWarning {
        message: format!(
            "Return type of {}::{} should either be compatible with {}::{}, or the #[\\ReturnTypeWillChange] attribute should be used to temporarily suppress the notice",
            class.name,
            method_signature_display(method),
            tentative_method.class_name,
            tentative_method.signature,
        ),
        span: method.span,
        kind: CompileWarningKind::Deprecation,
    });
    Ok(())
}

fn method_requires_parent_signature_compatibility(
    method: &MethodDecl,
    parent_method: &MethodDecl,
) -> bool {
    !method.name.eq_ignore_ascii_case("__construct") || parent_method.is_abstract
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
    runtime_class_aliases: &HashMap<String, String>,
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
        if !parameter_type_is_contravariant(
            parameter,
            parent_parameter,
            classes,
            runtime_class_aliases,
        ) {
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
        if !type_hint_is_subtype(
            return_type,
            parent_return_type,
            classes,
            runtime_class_aliases,
        ) {
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
    first_unavailable_class_name(candidate, classes)
        .or_else(|| first_unavailable_class_name(target, classes))
}

fn first_unavailable_class_name(type_hint: &TypeHint, classes: &[ClassDecl]) -> Option<String> {
    match type_hint {
        TypeHint::Class(name) if !class_type_name_is_available(name, classes) => Some(name.clone()),
        TypeHint::Nullable(inner) => first_unavailable_class_name(inner, classes),
        TypeHint::Union(types) | TypeHint::Intersection(types) => types
            .iter()
            .find_map(|member| first_unavailable_class_name(member, classes)),
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
        || name.eq_ignore_ascii_case("Closure")
        || name.eq_ignore_ascii_case("ArrayIterator")
        || name.eq_ignore_ascii_case("IteratorIterator")
        || name.eq_ignore_ascii_case("ArrayObject")
        || name.eq_ignore_ascii_case("Generator")
        || is_modeled_builtin_date_class_name(name)
        || is_modeled_builtin_exception_class_name(name)
        || find_class(classes, name).is_some()
}

fn parameter_type_is_contravariant(
    parameter: &FunctionParameter,
    parent_parameter: &FunctionParameter,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    match (&parameter.type_hint, &parent_parameter.type_hint) {
        (None, _) => true,
        (Some(TypeHint::Mixed), None) => true,
        (Some(_), None) => false,
        (Some(type_hint), Some(parent_type_hint)) => {
            type_hint_is_subtype(parent_type_hint, type_hint, classes, runtime_class_aliases)
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

fn method_signature_unresolved_compatibility_error_with_display(
    class: &ClassDecl,
    method: &MethodDecl,
    parent_class_name: &str,
    parent_method_signature: &str,
    unavailable_name: &str,
) -> Diagnostic {
    Diagnostic::new(
        format!(
            "Could not check compatibility between {}::{} and {}::{}, because class {} is not available",
            class.name,
            method_signature_display_canonical(method),
            parent_class_name,
            parent_method_signature,
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
        Expr::ClassConstantFetch {
            class_name, name, ..
        } => format!("{class_name}::{name}"),
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
    Void,
    Never,
    Static,
    Class(String),
}

#[derive(Debug, Clone)]
struct TypeAlternative {
    atoms: Vec<TypeAtom>,
}

fn type_hint_is_subtype(
    candidate: &TypeHint,
    target: &TypeHint,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    let candidate_alternatives = type_hint_alternatives(candidate);
    let target_alternatives = type_hint_alternatives(target);
    candidate_alternatives.iter().all(|candidate| {
        target_alternatives.iter().any(|target| {
            type_alternative_is_subtype(candidate, target, classes, runtime_class_aliases)
        })
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
        TypeHint::Mixed => vec![TypeAlternative {
            atoms: vec![TypeAtom::Mixed],
        }],
        TypeHint::Void => vec![TypeAlternative {
            atoms: vec![TypeAtom::Void],
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
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    target.atoms.iter().all(|target_atom| {
        candidate.atoms.iter().any(|candidate_atom| {
            type_atom_is_subtype(candidate_atom, target_atom, classes, runtime_class_aliases)
        })
    })
}

fn type_atom_is_subtype(
    candidate: &TypeAtom,
    target: &TypeAtom,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    if matches!(candidate, TypeAtom::Void) && matches!(target, TypeAtom::Mixed) {
        return false;
    }
    if matches!(candidate, TypeAtom::Never) || matches!(target, TypeAtom::Mixed) {
        return true;
    }
    if candidate == target {
        return true;
    }
    match (candidate, target) {
        (TypeAtom::True | TypeAtom::False, TypeAtom::Bool) => true,
        (TypeAtom::Static, TypeAtom::Object) => true,
        (TypeAtom::Class(candidate), TypeAtom::Callable)
            if candidate.eq_ignore_ascii_case("Closure") =>
        {
            true
        }
        (TypeAtom::Class(_), TypeAtom::Object) => true,
        (TypeAtom::Class(candidate), TypeAtom::Class(target)) => {
            class_type_name_is_subtype(candidate, target, classes, runtime_class_aliases)
        }
        _ => false,
    }
}

fn resolve_runtime_class_alias_from_map(
    name: &str,
    runtime_class_aliases: &HashMap<String, String>,
) -> String {
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
        let Some(target) = runtime_class_aliases.get(&key) else {
            break;
        };
        resolved = normalize_runtime_class_alias_target(target);
    }
    resolved
}

fn class_type_name_is_subtype(
    candidate_name: &str,
    target_name: &str,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    let candidate_name =
        resolve_runtime_class_alias_from_map(candidate_name, runtime_class_aliases);
    let target_name = resolve_runtime_class_alias_from_map(target_name, runtime_class_aliases);
    if candidate_name.eq_ignore_ascii_case(&target_name) {
        return true;
    }
    if builtin_class_type_is_subtype(&candidate_name, &target_name) {
        return true;
    }
    let Some(candidate) = find_class(classes, &candidate_name) else {
        return false;
    };
    if candidate.is_interface {
        return interface_type_extends(candidate, &target_name, classes, runtime_class_aliases);
    }
    class_type_extends_or_implements(candidate, &target_name, classes, runtime_class_aliases)
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
    } else if candidate_name.eq_ignore_ascii_case("DateTime")
        || candidate_name.eq_ignore_ascii_case("DateTimeImmutable")
    {
        &["DateTimeInterface"][..]
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
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    if target_name.eq_ignore_ascii_case("Stringable")
        && class
            .methods
            .iter()
            .any(|method| method.name.eq_ignore_ascii_case("__toString"))
    {
        return true;
    }
    if class.interfaces.iter().any(|interface| {
        interface_name_is_subtype(interface, target_name, classes, runtime_class_aliases)
    }) {
        return true;
    }
    let Some(parent_name) = class.parent_name.as_deref() else {
        return false;
    };
    let parent_name = resolve_runtime_class_alias_from_map(parent_name, runtime_class_aliases);
    if parent_name.eq_ignore_ascii_case(target_name)
        || builtin_class_type_is_subtype(&parent_name, target_name)
    {
        return true;
    }
    find_class(classes, &parent_name).is_some_and(|parent| {
        class_type_extends_or_implements(parent, target_name, classes, runtime_class_aliases)
    })
}

fn interface_name_is_subtype(
    interface_name: &str,
    target_name: &str,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    let interface_name =
        resolve_runtime_class_alias_from_map(interface_name, runtime_class_aliases);
    if interface_name.eq_ignore_ascii_case(target_name)
        || builtin_class_type_is_subtype(&interface_name, target_name)
    {
        return true;
    }
    find_class(classes, &interface_name)
        .filter(|interface| interface.is_interface)
        .is_some_and(|interface| {
            interface_type_extends(interface, target_name, classes, runtime_class_aliases)
        })
}

fn interface_type_extends(
    interface: &ClassDecl,
    target_name: &str,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    interface.interfaces.iter().any(|parent_name| {
        interface_name_is_subtype(parent_name, target_name, classes, runtime_class_aliases)
    })
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
        TypeHint::Union(types) => {
            let mut members = types
                .iter()
                .map(|member| type_hint_union_member_display(member))
                .collect::<Vec<_>>();
            if members
                .iter()
                .all(|member| union_builtin_display_rank(member).is_some())
            {
                members.sort_by_key(|member| union_builtin_display_rank(member).unwrap());
            }
            members.join("|")
        }
        TypeHint::Intersection(types) => types
            .iter()
            .map(type_hint_display)
            .collect::<Vec<_>>()
            .join("&"),
        TypeHint::Class(class_name) => class_name.clone(),
    }
}

fn type_hint_accepts_null_default(type_hint: &TypeHint) -> bool {
    match type_hint {
        TypeHint::Null | TypeHint::Mixed | TypeHint::Nullable(_) => true,
        TypeHint::Union(types) => types
            .iter()
            .any(|member| type_hint_accepts_null_default(member)),
        _ => false,
    }
}

fn union_builtin_display_rank(member: &str) -> Option<usize> {
    match member {
        "object" => Some(0),
        "array" => Some(1),
        "string" => Some(2),
        "int" => Some(3),
        "float" => Some(4),
        "bool" => Some(5),
        "false" => Some(6),
        "true" => Some(7),
        "null" => Some(8),
        _ => None,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttributeTarget {
    Class,
    Function,
    Method,
    Parameter,
    Property,
    ClassConstant,
    Constant,
}

fn validate_builtin_attributes_for_target(
    attributes: &AttributeMetadata,
    target: AttributeTarget,
    span: SourceSpan,
) -> Result<()> {
    validate_builtin_attribute_repetition("Attribute", attributes.attribute_count, span)?;
    validate_builtin_attribute_repetition(
        "AllowDynamicProperties",
        attributes.allow_dynamic_properties_count,
        span,
    )?;
    validate_builtin_attribute_repetition(
        "DelayedTargetValidation",
        attributes.delayed_target_validation_count,
        span,
    )?;
    validate_builtin_attribute_repetition("Deprecated", attributes.deprecated_count, span)?;
    validate_builtin_attribute_repetition("NoDiscard", attributes.no_discard_count, span)?;
    validate_builtin_attribute_repetition("Override", attributes.override_count, span)?;
    validate_builtin_attribute_repetition(
        "ReturnTypeWillChange",
        attributes.return_type_will_change_count,
        span,
    )?;

    if attributes.delayed_target_validation_count > 0 {
        return Ok(());
    }

    validate_builtin_attribute_target(
        "Attribute",
        attributes.attribute_count,
        target,
        &[AttributeTarget::Class],
        "class",
        span,
    )?;
    validate_builtin_attribute_target(
        "AllowDynamicProperties",
        attributes.allow_dynamic_properties_count,
        target,
        &[AttributeTarget::Class],
        "class",
        span,
    )?;
    validate_builtin_attribute_target(
        "Deprecated",
        attributes.deprecated_count,
        target,
        &[
            AttributeTarget::Class,
            AttributeTarget::Function,
            AttributeTarget::Method,
            AttributeTarget::ClassConstant,
            AttributeTarget::Constant,
        ],
        "class, function, method, class constant, constant",
        span,
    )?;
    validate_builtin_attribute_target(
        "NoDiscard",
        attributes.no_discard_count,
        target,
        &[AttributeTarget::Function, AttributeTarget::Method],
        "function, method",
        span,
    )?;
    validate_builtin_attribute_target(
        "ReturnTypeWillChange",
        attributes.return_type_will_change_count,
        target,
        &[AttributeTarget::Method],
        "method",
        span,
    )?;
    Ok(())
}

fn validate_builtin_attribute_repetition(name: &str, count: u16, span: SourceSpan) -> Result<()> {
    if count > 1 {
        return Err(Diagnostic::new(
            format!("Attribute \"{name}\" must not be repeated"),
            Some(span),
        ));
    }
    Ok(())
}

fn validate_builtin_attribute_target(
    name: &str,
    count: u16,
    target: AttributeTarget,
    allowed_targets: &[AttributeTarget],
    allowed_display: &str,
    span: SourceSpan,
) -> Result<()> {
    if count == 0 || allowed_targets.contains(&target) {
        return Ok(());
    }
    Err(Diagnostic::new(
        format!(
            "Attribute \"{name}\" cannot target {} (allowed targets: {allowed_display})",
            attribute_target_name(target)
        ),
        Some(span),
    ))
}

fn attribute_target_name(target: AttributeTarget) -> &'static str {
    match target {
        AttributeTarget::Class => "class",
        AttributeTarget::Function => "function",
        AttributeTarget::Method => "method",
        AttributeTarget::Parameter => "parameter",
        AttributeTarget::Property => "property",
        AttributeTarget::ClassConstant => "class constant",
        AttributeTarget::Constant => "constant",
    }
}

fn attribute_delays_target_validation(attributes: &AttributeMetadata) -> bool {
    attributes.delayed_target_validation_count > 0
}

fn validate_builtin_attribute_targets(
    classes: &[ClassDecl],
    traits: &[TraitDecl],
    functions: &[FunctionDecl],
) -> Result<()> {
    for function in functions {
        validate_builtin_attributes_for_target(
            &function.attributes,
            AttributeTarget::Function,
            function.span,
        )?;
        for parameter in &function.parameters {
            validate_builtin_attributes_for_target(
                &parameter.attributes,
                AttributeTarget::Parameter,
                parameter.span,
            )?;
        }
        if function.attributes.has_attribute
            && !attribute_delays_target_validation(&function.attributes)
        {
            return Err(Diagnostic::new(
                "Attribute \"Attribute\" cannot target function (allowed targets: class)",
                Some(function.span),
            ));
        }
        if !attribute_delays_target_validation(&function.attributes) {
            validate_no_discard_function_target(
                function.attributes.no_discard_message.is_some(),
                function.return_type.as_ref(),
                &format!("Function {}", function.name),
                function.span,
            )?;
        }
    }
    for class in classes {
        validate_builtin_attributes_for_target(
            &class.attributes,
            AttributeTarget::Class,
            class.span,
        )?;
        if class.attributes.has_attribute && !attribute_delays_target_validation(&class.attributes)
        {
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
        if class.attributes.has_allow_dynamic_properties
            && class.is_interface
            && !attribute_delays_target_validation(&class.attributes)
        {
            return Err(Diagnostic::new(
                format!(
                    "Cannot apply #[\\AllowDynamicProperties] to interface {}",
                    class.name
                ),
                Some(class.span),
            ));
        }
        if class.attributes.deprecated_message.is_some()
            && class.is_interface
            && !attribute_delays_target_validation(&class.attributes)
        {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Deprecated] to interface {}", class.name),
                Some(class.span),
            ));
        }
        if class.attributes.deprecated_message.is_some()
            && !class.is_interface
            && !attribute_delays_target_validation(&class.attributes)
        {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Deprecated] to class {}", class.name),
                Some(class.span),
            ));
        }
        for constant in &class.constants {
            validate_builtin_attributes_for_target(
                &constant.attributes,
                AttributeTarget::ClassConstant,
                constant.span,
            )?;
        }
        for method in &class.methods {
            validate_builtin_attributes_for_target(
                &method.attributes,
                AttributeTarget::Method,
                method.span,
            )?;
            for parameter in &method.parameters {
                validate_builtin_attributes_for_target(
                    &parameter.attributes,
                    AttributeTarget::Parameter,
                    parameter.span,
                )?;
            }
            if method.attributes.has_attribute
                && !attribute_delays_target_validation(&method.attributes)
            {
                return Err(Diagnostic::new(
                    "Attribute \"Attribute\" cannot target method (allowed targets: class)",
                    Some(method.span),
                ));
            }
            if method.attributes.no_discard_message.is_some()
                && (method.name.eq_ignore_ascii_case("__construct")
                    || method.name.eq_ignore_ascii_case("__clone"))
                && !attribute_delays_target_validation(&method.attributes)
            {
                return Err(Diagnostic::new(
                    format!(
                        "Method {}::{} cannot be #[\\NoDiscard]",
                        class.name, method.name
                    ),
                    Some(method.span),
                ));
            }
            if !attribute_delays_target_validation(&method.attributes) {
                validate_no_discard_function_target(
                    method.attributes.no_discard_message.is_some(),
                    method.return_type.as_ref(),
                    &format!("Method {}::{}", class.name, method.name),
                    method.span,
                )?;
            }
        }
    }
    for trait_decl in traits {
        validate_builtin_attributes_for_target(
            &trait_decl.attributes,
            AttributeTarget::Class,
            trait_decl.span,
        )?;
        if trait_decl.attributes.has_attribute
            && !attribute_delays_target_validation(&trait_decl.attributes)
        {
            return Err(Diagnostic::new(
                format!("Cannot apply #[\\Attribute] to trait {}", trait_decl.name),
                Some(trait_decl.span),
            ));
        }
        if trait_decl.attributes.has_allow_dynamic_properties
            && !attribute_delays_target_validation(&trait_decl.attributes)
        {
            return Err(Diagnostic::new(
                format!(
                    "Cannot apply #[\\AllowDynamicProperties] to trait {}",
                    trait_decl.name
                ),
                Some(trait_decl.span),
            ));
        }
        for constant in &trait_decl.constants {
            validate_builtin_attributes_for_target(
                &constant.attributes,
                AttributeTarget::ClassConstant,
                constant.span,
            )?;
        }
        for method in &trait_decl.methods {
            validate_builtin_attributes_for_target(
                &method.attributes,
                AttributeTarget::Method,
                method.span,
            )?;
            for parameter in &method.parameters {
                validate_builtin_attributes_for_target(
                    &parameter.attributes,
                    AttributeTarget::Parameter,
                    parameter.span,
                )?;
            }
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
        if let Some(required_method) = inherited_unsatisfied_abstract_method(class, classes) {
            return Err(Diagnostic::new(
                format!(
                    "Class {} must implement 1 abstract method ({}::{})",
                    class.name, required_method.declaring_class, required_method.method_name
                ),
                Some(class.span),
            ));
        }
    }
    Ok(())
}

struct RequiredAbstractMethod {
    declaring_class: String,
    method_name: String,
}

fn inherited_unsatisfied_abstract_method(
    class: &ClassDecl,
    classes: &[ClassDecl],
) -> Option<RequiredAbstractMethod> {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let Some(parent) = find_class(classes, name) else {
            break;
        };
        for method in &parent.methods {
            if !method.is_abstract || method.trait_name.is_some() {
                continue;
            }
            if concrete_method_exists_in_class_or_ancestors(class, &method.name, classes) {
                continue;
            }
            return Some(RequiredAbstractMethod {
                declaring_class: parent.name.clone(),
                method_name: method.name.clone(),
            });
        }
        parent_name = parent.parent_name.as_deref();
    }
    None
}

fn concrete_method_exists_in_class_or_ancestors(
    class: &ClassDecl,
    method_name: &str,
    classes: &[ClassDecl],
) -> bool {
    let mut current = Some(class);
    let mut seen = HashSet::new();
    while let Some(candidate) = current {
        if !seen.insert(candidate.name.to_ascii_lowercase()) {
            break;
        }
        if candidate
            .methods
            .iter()
            .any(|method| method_can_satisfy_override(method, method_name) && !method.is_abstract)
        {
            return true;
        }
        current = candidate
            .parent_name
            .as_deref()
            .and_then(|name| find_class(classes, name));
    }
    false
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
            if parent_property.visibility == PropertyVisibility::Private {
                continue;
            }
            if parent_property.set_visibility == PropertyVisibility::Private
                && parent_property.visibility != PropertyVisibility::Private
            {
                return Err(Diagnostic::new(
                    format!(
                        "Cannot override final property {}::${}",
                        parent.name, parent_property.name
                    ),
                    Some(property.span),
                ));
            }
            if parent_property.set_visibility == parent_property.visibility
                && !parent_property.has_hooks
                && property.set_visibility != property.visibility
            {
                return Err(Diagnostic::new(
                    format!(
                        "Set access level of {}::${} must be omitted (as in class {})",
                        class.name, property.name, parent.name
                    ),
                    Some(property.span),
                ));
            }
            if parent_property.has_hooks
                && parent_property.set_visibility == parent_property.visibility
                && property.set_visibility != property.visibility
            {
                continue;
            }
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

fn validate_class_constant_overrides(classes: &[ClassDecl]) -> Result<()> {
    for class in classes {
        for constant in &class.constants {
            if constant.is_final && constant.visibility == PropertyVisibility::Private {
                return Err(Diagnostic::new(
                    format!(
                        "Private constant {}::{} cannot be final as it is not visible to other classes",
                        class.name, constant.name
                    ),
                    Some(constant.span),
                ));
            }

            let mut parent_name = class.parent_name.as_deref();
            while let Some(name) = parent_name {
                let Some(parent) = classes
                    .iter()
                    .find(|candidate| candidate.name.eq_ignore_ascii_case(name))
                else {
                    break;
                };
                let Some(parent_constant) = parent
                    .constants
                    .iter()
                    .find(|candidate| candidate.name == constant.name)
                else {
                    parent_name = parent.parent_name.as_deref();
                    continue;
                };
                if parent_constant.visibility == PropertyVisibility::Private {
                    parent_name = parent.parent_name.as_deref();
                    continue;
                }
                if parent_constant.is_final {
                    return Err(Diagnostic::new(
                        format!(
                            "{}::{} cannot override final constant {}::{}",
                            class.name, constant.name, parent.name, parent_constant.name
                        ),
                        Some(constant.span),
                    ));
                }
                if visibility_rank(constant.visibility)
                    > visibility_rank(parent_constant.visibility)
                {
                    let suffix = if parent_constant.visibility == PropertyVisibility::Public {
                        ""
                    } else {
                        " or weaker"
                    };
                    return Err(Diagnostic::new(
                        format!(
                            "Access level to {}::{} must be {} (as in class {}){}",
                            class.name,
                            constant.name,
                            property_visibility_name(parent_constant.visibility),
                            parent.name,
                            suffix
                        ),
                        Some(constant.span),
                    ));
                }
                break;
            }

            for interface_name in &class.interfaces {
                let Some(interface) = classes
                    .iter()
                    .find(|candidate| candidate.name.eq_ignore_ascii_case(interface_name))
                else {
                    continue;
                };
                let Some(interface_constant) = interface
                    .constants
                    .iter()
                    .find(|candidate| candidate.name == constant.name)
                else {
                    continue;
                };
                if constant.visibility != interface_constant.visibility {
                    return Err(Diagnostic::new(
                        format!(
                            "Access level to {}::{} must be {} (as in interface {})",
                            class.name,
                            constant.name,
                            property_visibility_name(interface_constant.visibility),
                            interface.name
                        ),
                        Some(constant.span),
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_property_type_invariance(classes: &[ClassDecl]) -> Result<()> {
    let runtime_class_aliases = HashMap::new();
    for class in classes {
        for property in &class.properties {
            let Some((parent_class, parent_property)) =
                find_visible_parent_property(class, &property.name, classes)
            else {
                continue;
            };
            validate_property_type_pair(
                class,
                &property.name,
                property.type_hint.as_ref(),
                property.span,
                parent_class,
                parent_property.type_hint.as_ref(),
                classes,
                &runtime_class_aliases,
            )?;
        }
        for property in &class.static_properties {
            let Some((parent_class, parent_property)) =
                find_visible_parent_static_property(class, &property.name, classes)
            else {
                continue;
            };
            validate_property_type_pair(
                class,
                &property.name,
                property.type_hint.as_ref(),
                property.span,
                parent_class,
                parent_property.type_hint.as_ref(),
                classes,
                &runtime_class_aliases,
            )?;
        }
    }
    Ok(())
}

fn validate_property_type_pair(
    class: &ClassDecl,
    property_name: &str,
    type_hint: Option<&PropertyTypeHint>,
    span: SourceSpan,
    parent_class: &ClassDecl,
    parent_type_hint: Option<&PropertyTypeHint>,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> Result<()> {
    match (type_hint, parent_type_hint) {
        (None, None) => Ok(()),
        (Some(type_hint), Some(parent_type_hint))
            if property_type_hints_are_invariant(
                type_hint,
                parent_type_hint,
                classes,
                runtime_class_aliases,
            ) =>
        {
            Ok(())
        }
        (_, Some(parent_type_hint)) => Err(Diagnostic::new(
            format!(
                "Type of {}::${} must be {} (as in class {})",
                class.name,
                property_name,
                property_type_hint_display(parent_type_hint),
                parent_class.name
            ),
            Some(span),
        )),
        (Some(_), None) => Err(Diagnostic::new(
            format!(
                "Type of {}::${} must be omitted to match the parent definition in class {}",
                class.name, property_name, parent_class.name
            ),
            Some(span),
        )),
    }
}

fn find_visible_parent_property<'a>(
    class: &ClassDecl,
    property_name: &str,
    classes: &'a [ClassDecl],
) -> Option<(&'a ClassDecl, &'a PropertyDecl)> {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let parent = find_class(classes, name)?;
        if let Some(property) = parent
            .properties
            .iter()
            .find(|candidate| candidate.name == property_name)
        {
            if property.visibility != PropertyVisibility::Private {
                return Some((parent, property));
            }
        }
        parent_name = parent.parent_name.as_deref();
    }
    None
}

fn find_visible_parent_static_property<'a>(
    class: &ClassDecl,
    property_name: &str,
    classes: &'a [ClassDecl],
) -> Option<(&'a ClassDecl, &'a StaticPropertyDecl)> {
    let mut parent_name = class.parent_name.as_deref();
    let mut seen = HashSet::new();
    while let Some(name) = parent_name {
        if !seen.insert(name.to_ascii_lowercase()) {
            break;
        }
        let parent = find_class(classes, name)?;
        if let Some(property) = parent
            .static_properties
            .iter()
            .find(|candidate| candidate.name == property_name)
        {
            if property.visibility != PropertyVisibility::Private {
                return Some((parent, property));
            }
        }
        parent_name = parent.parent_name.as_deref();
    }
    None
}

fn property_type_hints_are_invariant(
    left: &PropertyTypeHint,
    right: &PropertyTypeHint,
    classes: &[ClassDecl],
    runtime_class_aliases: &HashMap<String, String>,
) -> bool {
    if let (Some(left), Some(right)) = (&left.semantic_type, &right.semantic_type) {
        return type_hint_is_subtype(left, right, classes, runtime_class_aliases)
            && type_hint_is_subtype(right, left, classes, runtime_class_aliases);
    }
    property_type_hint_key(left) == property_type_hint_key(right)
}

fn property_type_hint_key(type_hint: &PropertyTypeHint) -> String {
    match &type_hint.kind {
        PropertyTypeKind::Null => "null".to_string(),
        PropertyTypeKind::Array => nullable_property_type_key("array", type_hint.allows_null),
        PropertyTypeKind::Int => nullable_property_type_key("int", type_hint.allows_null),
        PropertyTypeKind::Float => nullable_property_type_key("float", type_hint.allows_null),
        PropertyTypeKind::String => nullable_property_type_key("string", type_hint.allows_null),
        PropertyTypeKind::Bool => nullable_property_type_key("bool", type_hint.allows_null),
        PropertyTypeKind::Mixed => "mixed".to_string(),
        PropertyTypeKind::Object => nullable_property_type_key("object", type_hint.allows_null),
        PropertyTypeKind::Class(name) => nullable_property_type_key(
            &format!(
                "class:{}",
                name.trim_start_matches('\\').to_ascii_lowercase()
            ),
            type_hint.allows_null,
        ),
        PropertyTypeKind::Unsupported => property_type_text_key(&type_hint.text),
    }
}

fn nullable_property_type_key(base: &str, allows_null: bool) -> String {
    if allows_null {
        let mut members = vec![base.to_string(), "null".to_string()];
        members.sort();
        return format!("union:{}", members.join("|"));
    }
    base.to_string()
}

fn property_type_text_key(text: &str) -> String {
    let text = text.trim();
    if let Some(inner) = text.strip_prefix('?') {
        let mut members = vec![property_type_text_key(inner), "null".to_string()];
        members.sort();
        return format!("union:{}", members.join("|"));
    }
    if text.contains('|') {
        let mut members = text
            .split('|')
            .map(property_type_text_key)
            .collect::<Vec<_>>();
        members.sort();
        return format!("union:{}", members.join("|"));
    }
    if text.contains('&') {
        let mut members = text
            .split('&')
            .map(property_type_text_key)
            .collect::<Vec<_>>();
        members.sort();
        return format!("intersection:{}", members.join("&"));
    }
    property_type_atom_key(text)
}

fn property_type_atom_key(text: &str) -> String {
    match text
        .trim()
        .trim_start_matches('\\')
        .to_ascii_lowercase()
        .as_str()
    {
        "integer" => "int".to_string(),
        "double" => "float".to_string(),
        "boolean" => "bool".to_string(),
        "binary" => "string".to_string(),
        "null" | "array" | "int" | "float" | "string" | "bool" | "mixed" | "object"
        | "callable" | "false" | "true" | "iterable" | "never" | "static" | "void" => {
            text.trim().trim_start_matches('\\').to_ascii_lowercase()
        }
        class_name => format!("class:{class_name}"),
    }
}

fn property_type_hint_display(type_hint: &PropertyTypeHint) -> String {
    match &type_hint.kind {
        PropertyTypeKind::Null => "null".to_string(),
        PropertyTypeKind::Array => nullable_property_type_display("array", type_hint.allows_null),
        PropertyTypeKind::Int => nullable_property_type_display("int", type_hint.allows_null),
        PropertyTypeKind::Float => nullable_property_type_display("float", type_hint.allows_null),
        PropertyTypeKind::String => nullable_property_type_display("string", type_hint.allows_null),
        PropertyTypeKind::Bool => nullable_property_type_display("bool", type_hint.allows_null),
        PropertyTypeKind::Mixed => "mixed".to_string(),
        PropertyTypeKind::Object => nullable_property_type_display("object", type_hint.allows_null),
        PropertyTypeKind::Class(name) => {
            nullable_property_type_display(name, type_hint.allows_null)
        }
        PropertyTypeKind::Unsupported => property_type_text_display(&type_hint.text),
    }
}

fn nullable_property_type_display(base: &str, allows_null: bool) -> String {
    if allows_null {
        format!("?{base}")
    } else {
        base.to_string()
    }
}

fn property_type_text_display(text: &str) -> String {
    let text = text.trim();
    if let Some(inner) = text.strip_prefix('?') {
        return format!("?{}", property_type_text_display(inner));
    }
    if text.contains('|') {
        let mut members = text
            .split('|')
            .map(property_type_text_display)
            .collect::<Vec<_>>();
        if members
            .iter()
            .all(|member| property_union_builtin_display_rank(member).is_some())
        {
            members.sort_by_key(|member| property_union_builtin_display_rank(member).unwrap());
        }
        return members.join("|");
    }
    if text.contains('&') {
        return text
            .split('&')
            .map(property_type_text_display)
            .collect::<Vec<_>>()
            .join("&");
    }
    property_type_atom_display(text)
}

fn property_type_atom_display(text: &str) -> String {
    match text.trim().to_ascii_lowercase().as_str() {
        "integer" => "int".to_string(),
        "double" => "float".to_string(),
        "boolean" => "bool".to_string(),
        "binary" => "string".to_string(),
        "null" | "array" | "int" | "float" | "string" | "bool" | "mixed" | "object"
        | "callable" | "false" | "true" | "iterable" | "never" | "static" | "void" => {
            text.trim().to_ascii_lowercase()
        }
        _ => text.trim().to_string(),
    }
}

fn property_union_builtin_display_rank(member: &str) -> Option<usize> {
    match member {
        "object" => Some(0),
        "array" => Some(1),
        "string" => Some(2),
        "int" => Some(3),
        "float" => Some(4),
        "bool" => Some(5),
        "false" => Some(6),
        "true" => Some(7),
        "null" => Some(8),
        _ => None,
    }
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
    validate_control_transfers_in_statements_inner(statements, control_depth, &mut Vec::new())
}

fn validate_control_transfers_in_statements_inner(
    statements: &[Statement],
    control_depth: usize,
    finally_control_depths: &mut Vec<usize>,
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
            | Statement::Exit {
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
                validate_control_transfers_in_statements_inner(
                    statements,
                    control_depth,
                    finally_control_depths,
                )?;
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                validate_control_transfers_in_expr(condition)?;
                validate_control_transfers_in_statements_inner(
                    then_body,
                    control_depth,
                    finally_control_depths,
                )?;
                validate_control_transfers_in_statements_inner(
                    else_body,
                    control_depth,
                    finally_control_depths,
                )?;
            }
            Statement::While {
                condition, body, ..
            } => {
                validate_control_transfers_in_expr(condition)?;
                validate_control_transfers_in_statements_inner(
                    body,
                    control_depth + 1,
                    finally_control_depths,
                )?;
            }
            Statement::DoWhile {
                body, condition, ..
            } => {
                validate_control_transfers_in_statements_inner(
                    body,
                    control_depth + 1,
                    finally_control_depths,
                )?;
                validate_control_transfers_in_expr(condition)?;
            }
            Statement::For {
                initializers,
                condition,
                updates,
                body,
                ..
            } => {
                validate_control_transfers_in_statements_inner(
                    initializers,
                    control_depth,
                    finally_control_depths,
                )?;
                if let Some(condition) = condition {
                    validate_control_transfers_in_expr(condition)?;
                }
                validate_control_transfers_in_statements_inner(
                    updates,
                    control_depth,
                    finally_control_depths,
                )?;
                validate_control_transfers_in_statements_inner(
                    body,
                    control_depth + 1,
                    finally_control_depths,
                )?;
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
                validate_control_transfers_in_statements_inner(
                    body,
                    control_depth + 1,
                    finally_control_depths,
                )?;
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                validate_control_transfers_in_expr(expression)?;
                for case in cases {
                    if let Some(condition) = &case.condition {
                        validate_control_transfers_in_expr(condition)?;
                    }
                    validate_control_transfers_in_statements_inner(
                        &case.body,
                        control_depth + 1,
                        finally_control_depths,
                    )?;
                }
            }
            Statement::Break { level, span } => validate_control_transfer_target(
                ControlTransfer::Break,
                *level,
                *span,
                control_depth,
                finally_control_depths,
            )?,
            Statement::Continue { level, span } => validate_control_transfer_target(
                ControlTransfer::Continue,
                *level,
                *span,
                control_depth,
                finally_control_depths,
            )?,
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_control_transfers_in_statements_inner(
                    body,
                    control_depth,
                    finally_control_depths,
                )?;
                for catch in catches {
                    validate_control_transfers_in_statements_inner(
                        &catch.body,
                        control_depth,
                        finally_control_depths,
                    )?;
                }
                finally_control_depths.push(control_depth);
                let result = validate_control_transfers_in_statements_inner(
                    finally_body,
                    control_depth,
                    finally_control_depths,
                );
                finally_control_depths.pop();
                result?;
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
            | Statement::Exit { value: None, .. }
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
    finally_control_depths: &[usize],
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
    let target_depth = control_depth - level;
    if finally_control_depths
        .iter()
        .any(|finally_control_depth| target_depth < *finally_control_depth)
    {
        return Err(Diagnostic::new(
            "jump out of a finally block is disallowed",
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
        | Expr::DynamicStaticPropertyFetch { receiver: expr, .. }
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
        Expr::YieldFrom { expr, .. } => validate_control_transfers_in_expr(expr)?,
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
        AssignmentTarget::DynamicStaticProperty { receiver, .. } => {
            validate_control_transfers_in_expr(receiver)?;
        }
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
        AssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            validate_control_transfers_in_expr(receiver)?;
            validate_control_transfers_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            validate_control_transfers_in_expr(array)?;
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
        ReferenceTarget::DynamicVariable { name, .. } => {
            validate_control_transfers_in_expr(name)?;
        }
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
        AssignmentTarget::DynamicStaticPropertyArrayDim { .. } => return Ok(()),
        AssignmentTarget::ValueArrayDim { .. } => return Ok(()),
        AssignmentTarget::Property { .. } => return Ok(()),
        AssignmentTarget::DynamicProperty { .. } => return Ok(()),
        AssignmentTarget::StaticProperty { .. } => return Ok(()),
        AssignmentTarget::DynamicStaticProperty { .. } => return Ok(()),
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
        Expr::YieldFrom { expr, .. } => expr_array_literal_reference_to_variable(expr, variable),
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
        | Expr::DynamicStaticPropertyFetch { .. }
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
        ReferenceTarget::DynamicVariable { name, .. } => {
            expr_array_literal_reference_to_variable(name, variable)
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
        | Expr::DynamicVariable { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::DynamicMethodCall { .. }
        | Expr::PropertyFetch { .. }
        | Expr::DynamicPropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. } => Ok(()),
        Expr::Call { name, span, .. }
            if is_modeled_internal_function_name(name)
                && !modeled_internal_has_by_ref_parameter(name) =>
        {
            Err(Diagnostic::new(
                "Cannot use result of built-in function in write context",
                Some(*span),
            ))
        }
        Expr::Call { .. } => Ok(()),
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
            validate_void_returns_in_statements(&function.body, FunctionLikeKind::Function)?;
        }
    }
    Ok(())
}

fn validate_never_returns(functions: &[FunctionDecl]) -> Result<()> {
    for function in functions {
        if matches!(&function.return_type, Some(TypeHint::Never)) {
            validate_never_returns_in_statements(&function.body, FunctionLikeKind::Function)?;
        }
    }
    Ok(())
}

fn validate_typed_returns_have_values(functions: &[FunctionDecl]) -> Result<()> {
    for function in functions {
        if let Some(return_type) = &function.return_type {
            validate_generator_return_type(&function.body, return_type, function.span)?;
            validate_typed_returns_have_values_in_statements(
                &function.body,
                return_type,
                FunctionLikeKind::Function,
            )?;
        }
    }
    Ok(())
}

fn validate_void_returns_in_statements(
    statements: &[Statement],
    kind: FunctionLikeKind,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Return { value, span } => {
                if let Some(value) = value {
                    let message = if value_is_null_literal(value) {
                        format!(
                            "A void {} must not return a value (did you mean \"return;\" instead of \"return null;\"?)",
                            kind.label()
                        )
                    } else {
                        format!("A void {} must not return a value", kind.label())
                    };
                    return Err(Diagnostic::new(message, Some(*span)));
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_void_returns_in_statements(then_body, kind)?;
                validate_void_returns_in_statements(else_body, kind)?;
            }
            Statement::Block { statements, .. } => {
                validate_void_returns_in_statements(statements, kind)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_void_returns_in_statements(body, kind)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_void_returns_in_statements(initializers, kind)?;
                validate_void_returns_in_statements(updates, kind)?;
                validate_void_returns_in_statements(body, kind)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_void_returns_in_statements(body, kind)?;
                for catch in catches {
                    validate_void_returns_in_statements(&catch.body, kind)?;
                }
                validate_void_returns_in_statements(finally_body, kind)?;
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_void_returns_in_statements(&case.body, kind)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_typed_returns_have_values_in_statements(
    statements: &[Statement],
    return_type: &TypeHint,
    kind: FunctionLikeKind,
) -> Result<()> {
    if matches!(return_type, TypeHint::Void | TypeHint::Never)
        || statements_contain_yield(statements)
    {
        return Ok(());
    }
    for statement in statements {
        match statement {
            Statement::Return { value: None, span } => {
                let mut message =
                    format!("A {} with return type must return a value", kind.label());
                if type_hint_accepts_null_default(return_type) {
                    message.push_str(" (did you mean \"return null;\" instead of \"return;\"?)");
                }
                return Err(Diagnostic::new(message, Some(*span)));
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_typed_returns_have_values_in_statements(then_body, return_type, kind)?;
                validate_typed_returns_have_values_in_statements(else_body, return_type, kind)?;
            }
            Statement::Block { statements, .. } => {
                validate_typed_returns_have_values_in_statements(statements, return_type, kind)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_typed_returns_have_values_in_statements(body, return_type, kind)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_typed_returns_have_values_in_statements(initializers, return_type, kind)?;
                validate_typed_returns_have_values_in_statements(updates, return_type, kind)?;
                validate_typed_returns_have_values_in_statements(body, return_type, kind)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_typed_returns_have_values_in_statements(body, return_type, kind)?;
                for catch in catches {
                    validate_typed_returns_have_values_in_statements(
                        &catch.body,
                        return_type,
                        kind,
                    )?;
                }
                validate_typed_returns_have_values_in_statements(finally_body, return_type, kind)?;
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_typed_returns_have_values_in_statements(
                        &case.body,
                        return_type,
                        kind,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_never_returns_in_statements(
    statements: &[Statement],
    kind: FunctionLikeKind,
) -> Result<()> {
    for statement in statements {
        match statement {
            Statement::Return { span, .. } => {
                return Err(Diagnostic::new(
                    format!("A never-returning {} must not return", kind.label()),
                    Some(*span),
                ));
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                validate_never_returns_in_statements(then_body, kind)?;
                validate_never_returns_in_statements(else_body, kind)?;
            }
            Statement::Block { statements, .. } => {
                validate_never_returns_in_statements(statements, kind)?;
            }
            Statement::While { body, .. }
            | Statement::DoWhile { body, .. }
            | Statement::Foreach { body, .. } => {
                validate_never_returns_in_statements(body, kind)?;
            }
            Statement::For {
                initializers,
                updates,
                body,
                ..
            } => {
                validate_never_returns_in_statements(initializers, kind)?;
                validate_never_returns_in_statements(updates, kind)?;
                validate_never_returns_in_statements(body, kind)?;
            }
            Statement::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                validate_never_returns_in_statements(body, kind)?;
                for catch in catches {
                    validate_never_returns_in_statements(&catch.body, kind)?;
                }
                validate_never_returns_in_statements(finally_body, kind)?;
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    validate_never_returns_in_statements(&case.body, kind)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_generator_return_type(
    statements: &[Statement],
    return_type: &TypeHint,
    span: SourceSpan,
) -> Result<()> {
    if statements_contain_yield(statements) && !type_hint_accepts_generator_return(return_type) {
        return Err(Diagnostic::new(
            format!(
                "Generator return type must be a supertype of Generator, {} given",
                type_hint_display(return_type)
            ),
            Some(span),
        ));
    }
    Ok(())
}

fn type_hint_accepts_generator_return(type_hint: &TypeHint) -> bool {
    match type_hint {
        TypeHint::Mixed | TypeHint::Object | TypeHint::Iterable => true,
        TypeHint::Class(name) => matches!(
            name.rsplit('\\')
                .next()
                .unwrap_or(name)
                .to_ascii_lowercase()
                .as_str(),
            "generator" | "iterator" | "traversable"
        ),
        TypeHint::Nullable(inner) => type_hint_accepts_generator_return(inner),
        TypeHint::Union(types) => types.iter().any(type_hint_accepts_generator_return),
        _ => false,
    }
}

fn value_is_null_literal(value: &Expr) -> bool {
    match value {
        Expr::Null(_) => true,
        Expr::Grouped { expr, .. } => value_is_null_literal(expr),
        _ => false,
    }
}

fn statements_contain_yield(statements: &[Statement]) -> bool {
    statements.iter().any(statement_contains_yield)
}

fn statement_contains_yield(statement: &Statement) -> bool {
    match statement {
        Statement::Assign { value, .. } | Statement::ArrayAssign { value, .. } => {
            expr_contains_yield(value)
        }
        Statement::AssignRef { source, .. } | Statement::ArrayAssignRef { source, .. } => {
            expr_contains_yield(source)
        }
        Statement::Increment { target, .. } => inc_dec_target_contains_yield(target),
        Statement::Unset { targets, .. } => targets.iter().any(unset_target_contains_yield),
        Statement::Static { declarations, .. } => declarations
            .iter()
            .any(|declaration| declaration.value.as_ref().is_some_and(expr_contains_yield)),
        Statement::Call { arguments, .. }
        | Statement::Echo {
            expressions: arguments,
            ..
        } => arguments.iter().any(expr_contains_yield),
        Statement::Print { expression, .. } | Statement::Expression { expression, .. } => {
            expr_contains_yield(expression)
        }
        Statement::Const { declarations, .. } => declarations
            .iter()
            .any(|declaration| expr_contains_yield(&declaration.value)),
        Statement::Block { statements, .. } => statements_contain_yield(statements),
        Statement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_yield(condition)
                || statements_contain_yield(then_body)
                || statements_contain_yield(else_body)
        }
        Statement::While {
            condition, body, ..
        } => expr_contains_yield(condition) || statements_contain_yield(body),
        Statement::DoWhile {
            body, condition, ..
        } => statements_contain_yield(body) || expr_contains_yield(condition),
        Statement::For {
            initializers,
            condition,
            updates,
            body,
            ..
        } => {
            statements_contain_yield(initializers)
                || condition.as_ref().is_some_and(expr_contains_yield)
                || statements_contain_yield(updates)
                || statements_contain_yield(body)
        }
        Statement::Foreach {
            iterable,
            key,
            value,
            body,
            ..
        } => {
            expr_contains_yield(iterable)
                || key.as_ref().is_some_and(assignment_target_contains_yield)
                || assignment_target_contains_yield(value)
                || statements_contain_yield(body)
        }
        Statement::Switch {
            expression, cases, ..
        } => {
            expr_contains_yield(expression)
                || cases.iter().any(|case| {
                    case.condition.as_ref().is_some_and(expr_contains_yield)
                        || statements_contain_yield(&case.body)
                })
        }
        Statement::Return {
            value: Some(value), ..
        }
        | Statement::Exit {
            value: Some(value), ..
        }
        | Statement::Throw { value, .. } => expr_contains_yield(value),
        Statement::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            statements_contain_yield(body)
                || catches
                    .iter()
                    .any(|catch| statements_contain_yield(&catch.body))
                || statements_contain_yield(finally_body)
        }
        Statement::Empty { .. }
        | Statement::ClassDeclaration { .. }
        | Statement::FunctionDeclaration { .. }
        | Statement::Global { .. }
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::Return { value: None, .. }
        | Statement::Exit { value: None, .. }
        | Statement::Label { .. }
        | Statement::Goto { .. }
        | Statement::InlineHtml { .. } => false,
    }
}

fn expr_contains_yield(expr: &Expr) -> bool {
    match expr {
        Expr::Yield { .. } | Expr::YieldFrom { .. } => true,
        Expr::AnonymousFunction(_) => false,
        Expr::DynamicVariable { name, .. }
        | Expr::FirstClassCallable { callable: name, .. }
        | Expr::DynamicClassNameFetch { receiver: name, .. }
        | Expr::Clone { expr: name, .. }
        | Expr::Print {
            expression: name, ..
        }
        | Expr::Include { path: name, .. }
        | Expr::Throw { value: name, .. }
        | Expr::Unary { expr: name, .. }
        | Expr::Cast { expr: name, .. }
        | Expr::Grouped { expr: name, .. }
        | Expr::PipeValue { expr: name, .. } => expr_contains_yield(name),
        Expr::IncDec { target, .. } => inc_dec_target_contains_yield(target),
        Expr::Assign { target, value, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(value)
        }
        Expr::AssignRef { target, source, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(source)
        }
        Expr::Call { arguments, .. } | Expr::NewObject { arguments, .. } => {
            arguments.iter().any(expr_contains_yield)
        }
        Expr::DynamicCall {
            callee, arguments, ..
        } => expr_contains_yield(callee) || arguments.iter().any(expr_contains_yield),
        Expr::MethodCall {
            receiver,
            arguments,
            ..
        } => expr_contains_yield(receiver) || arguments.iter().any(expr_contains_yield),
        Expr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => {
            expr_contains_yield(receiver)
                || expr_contains_yield(name)
                || arguments.iter().any(expr_contains_yield)
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => expr_contains_yield(class_name) || arguments.iter().any(expr_contains_yield),
        Expr::PropertyFetch { receiver, .. }
        | Expr::NullsafePropertyFetch { receiver, .. }
        | Expr::DynamicStaticPropertyFetch { receiver, .. } => expr_contains_yield(receiver),
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        Expr::InstanceOf { expr, target, .. } => {
            expr_contains_yield(expr)
                || matches!(target, InstanceOfTarget::Expr(target) if expr_contains_yield(target))
        }
        Expr::Match { subject, arms, .. } => {
            expr_contains_yield(subject)
                || arms.iter().any(|arm| {
                    arm.conditions.iter().any(expr_contains_yield)
                        || expr_contains_yield(&arm.value)
                })
        }
        Expr::Array { elements, .. } => elements.iter().any(array_element_contains_yield),
        Expr::List(list) => list.elements.iter().any(list_expr_element_contains_yield),
        Expr::ArrayAccess { array, index, .. } => {
            expr_contains_yield(array)
                || index
                    .as_ref()
                    .is_some_and(|index| expr_contains_yield(index))
        }
        Expr::Isset { targets, .. } => targets.iter().any(expr_contains_yield),
        Expr::Empty { target, .. } => expr_contains_yield(target),
        Expr::Binary { left, right, .. } => expr_contains_yield(left) || expr_contains_yield(right),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            expr_contains_yield(condition)
                || if_true
                    .as_ref()
                    .is_some_and(|if_true| expr_contains_yield(if_true))
                || expr_contains_yield(if_false)
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
        | Expr::ClassConstantFetch { .. } => false,
    }
}

fn array_element_contains_yield(element: &ArrayElement) -> bool {
    element.key.as_ref().is_some_and(expr_contains_yield)
        || match &element.value {
            ArrayElementValue::Hole(_) => false,
            ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                expr_contains_yield(value)
            }
            ArrayElementValue::Reference(target) => reference_target_contains_yield(target),
        }
}

fn list_expr_element_contains_yield(element: &ListExprElement) -> bool {
    element.key.as_ref().is_some_and(expr_contains_yield)
        || element.target.as_ref().is_some_and(|target| match target {
            ListExprElementTarget::Value(value) => expr_contains_yield(value),
            ListExprElementTarget::Reference(target) => reference_target_contains_yield(target),
        })
}

fn assignment_target_contains_yield(target: &AssignmentTarget) -> bool {
    match target {
        AssignmentTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => expr_contains_yield(name) || dimensions.iter().flatten().any(expr_contains_yield),
        AssignmentTarget::ArrayDim(target) => {
            target.dimensions.iter().flatten().any(expr_contains_yield)
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        }
        | AssignmentTarget::ValueArrayDim {
            array: receiver,
            dimensions,
            ..
        }
        | AssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AssignmentTarget::Property { receiver, .. }
        | AssignmentTarget::DynamicStaticProperty { receiver, .. } => expr_contains_yield(receiver),
        AssignmentTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        AssignmentTarget::List(list) => list_assignment_target_contains_yield(list),
        AssignmentTarget::Variable { .. }
        | AssignmentTarget::StaticPropertyArrayDim { .. }
        | AssignmentTarget::StaticProperty { .. } => false,
    }
}

fn list_assignment_target_contains_yield(target: &ListAssignmentTarget) -> bool {
    target.elements.iter().any(|element| {
        element.key.as_ref().is_some_and(expr_contains_yield)
            || match &element.target {
                ListAssignmentElementTarget::Value(target) => {
                    assignment_target_contains_yield(target)
                }
                ListAssignmentElementTarget::Reference(target) => {
                    reference_target_contains_yield(target)
                }
            }
    })
}

fn reference_target_contains_yield(target: &ReferenceTarget) -> bool {
    match target {
        ReferenceTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        ReferenceTarget::ArrayDim(target) => {
            target.dimensions.iter().flatten().any(expr_contains_yield)
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        ReferenceTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        ReferenceTarget::Variable { .. } => false,
    }
}

fn inc_dec_target_contains_yield(target: &IncDecTarget) -> bool {
    match target {
        IncDecTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        IncDecTarget::DynamicArrayDim {
            name, dimensions, ..
        } => expr_contains_yield(name) || dimensions.iter().flatten().any(expr_contains_yield),
        IncDecTarget::ArrayDim(target) => {
            target.dimensions.iter().flatten().any(expr_contains_yield)
        }
        IncDecTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        IncDecTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        IncDecTarget::Variable { .. } | IncDecTarget::StaticProperty { .. } => false,
    }
}

fn unset_target_contains_yield(target: &UnsetTarget) -> bool {
    match target {
        UnsetTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        UnsetTarget::DynamicArrayDim {
            name, dimensions, ..
        } => expr_contains_yield(name) || dimensions.iter().any(expr_contains_yield),
        UnsetTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().any(expr_contains_yield),
        UnsetTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        UnsetTarget::ArrayDim(target) => {
            target.dimensions.iter().flatten().any(expr_contains_yield)
        }
        UnsetTarget::Variable { .. } => false,
    }
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
            | Statement::Exit {
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
            | Statement::Exit { value: None, .. }
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
                validate_void_returns_in_statements(&function.body, FunctionLikeKind::Function)?;
            }
            if !function.is_arrow && matches!(&function.return_type, Some(TypeHint::Never)) {
                validate_never_returns_in_statements(&function.body, FunctionLikeKind::Function)?;
            }
            if !function.is_arrow {
                if let Some(return_type) = &function.return_type {
                    validate_generator_return_type(&function.body, return_type, function.span)?;
                    validate_typed_returns_have_values_in_statements(
                        &function.body,
                        return_type,
                        FunctionLikeKind::Function,
                    )?;
                }
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
        Expr::DynamicStaticPropertyFetch { receiver, .. } => {
            validate_anonymous_functions_in_expr(receiver, functions)?;
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
        Expr::YieldFrom { expr, .. } => validate_anonymous_functions_in_expr(expr, functions)?,
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
            | "copy"
            | "crypt"
            | "crc32"
            | "disk_free_space"
            | "disk_total_space"
            | "diskfreespace"
            | "md5"
            | "md5_file"
            | "sha1"
            | "sha1_file"
            | "similar_text"
            | "substr"
            | "dirname"
            | "bin2hex"
            | "hex2bin"
            | "exec"
            | "parse_url"
            | "urlencode"
            | "urldecode"
            | "rawurlencode"
            | "rawurldecode"
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
            | "system"
            | "fdiv"
            | "fnmatch"
            | "fclose"
            | "feof"
            | "fflush"
            | "fgetc"
            | "fgetcsv"
            | "fgets"
            | "fscanf"
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
            | "passthru"
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
            | "get_defined_constants"
            | "get_loaded_extensions"
            | "get_html_translation_table"
            | "get_meta_tags"
            | "get_current_user"
            | "gethostname"
            | "highlight_file"
            | "highlight_string"
            | "show_source"
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
            | "memory_get_peak_usage"
            | "memory_get_usage"
            | "memory_reset_peak_usage"
            | "php_ini_scanned_files"
            | "php_sapi_name"
            | "php_uname"
            | "phpversion"
            | "preg_match"
            | "print_r"
            | "readlink"
            | "realpath_cache_get"
            | "realpath_cache_size"
            | "realpath"
            | "rename"
            | "rewind"
            | "set_time_limit"
            | "sleep"
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
            | "register_shutdown_function"
            | "register_tick_function"
            | "arsort"
            | "asort"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "error_reporting"
            | "escapeshellarg"
            | "explode"
            | "func_get_arg"
            | "func_get_args"
            | "func_num_args"
            | "get_resource_type"
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
            | "link"
            | "linkinfo"
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
            | "glob"
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
            | "shell_exec"
            | "str_shuffle"
            | "str_ireplace"
            | "str_replace"
            | "strrchr"
            | "strtok"
            | "str_getcsv"
            | "strtr"
            | "str_word_count"
            | "symlink"
            | "sys_get_temp_dir"
            | "tempnam"
            | "umask"
            | "uniqid"
            | "unlink"
            | "usleep"
            | "call_user_func"
            | "call_user_func_array"
            | "class_alias"
            | "class_exists"
            | "debug_zval_dump"
            | "is_callable"
            | "is_subclass_of"
            | "method_exists"
            | "microtime"
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
            | "FNM_NOESCAPE"
            | "FNM_PATHNAME"
            | "FNM_PERIOD"
            | "FNM_CASEFOLD"
            | "GLOB_MARK"
            | "GLOB_NOSORT"
            | "GLOB_NOCHECK"
            | "GLOB_NOESCAPE"
            | "GLOB_BRACE"
            | "GLOB_ONLYDIR"
            | "GLOB_ERR"
            | "PATHINFO_DIRNAME"
            | "PATHINFO_BASENAME"
            | "PATHINFO_EXTENSION"
            | "PATHINFO_FILENAME"
            | "PATHINFO_ALL"
            | "HTML_SPECIALCHARS"
            | "HTML_ENTITIES"
            | "JSON_ERROR_NONE"
            | "JSON_ERROR_DEPTH"
            | "JSON_ERROR_STATE_MISMATCH"
            | "JSON_ERROR_CTRL_CHAR"
            | "JSON_ERROR_SYNTAX"
            | "JSON_ERROR_UTF8"
            | "JSON_ERROR_RECURSION"
            | "JSON_ERROR_INF_OR_NAN"
            | "JSON_ERROR_UNSUPPORTED_TYPE"
            | "JSON_ERROR_INVALID_PROPERTY_NAME"
            | "JSON_ERROR_UTF16"
            | "JSON_ERROR_NON_BACKED_ENUM"
            | "JSON_HEX_TAG"
            | "JSON_HEX_AMP"
            | "JSON_HEX_APOS"
            | "JSON_HEX_QUOT"
            | "JSON_FORCE_OBJECT"
            | "JSON_NUMERIC_CHECK"
            | "JSON_UNESCAPED_SLASHES"
            | "JSON_PRETTY_PRINT"
            | "JSON_UNESCAPED_UNICODE"
            | "JSON_PARTIAL_OUTPUT_ON_ERROR"
            | "JSON_PRESERVE_ZERO_FRACTION"
            | "JSON_UNESCAPED_LINE_TERMINATORS"
            | "JSON_OBJECT_AS_ARRAY"
            | "JSON_BIGINT_AS_STRING"
            | "JSON_INVALID_UTF8_IGNORE"
            | "JSON_INVALID_UTF8_SUBSTITUTE"
            | "JSON_THROW_ON_ERROR"
            | "PREG_PATTERN_ORDER"
            | "PREG_SET_ORDER"
            | "PREG_OFFSET_CAPTURE"
            | "PREG_UNMATCHED_AS_NULL"
            | "PREG_SPLIT_NO_EMPTY"
            | "PREG_SPLIT_DELIM_CAPTURE"
            | "PREG_SPLIT_OFFSET_CAPTURE"
            | "PREG_GREP_INVERT"
            | "PREG_NO_ERROR"
            | "PREG_INTERNAL_ERROR"
            | "PREG_BACKTRACK_LIMIT_ERROR"
            | "PREG_RECURSION_LIMIT_ERROR"
            | "PREG_BAD_UTF8_ERROR"
            | "PREG_BAD_UTF8_OFFSET_ERROR"
            | "PREG_JIT_STACKLIMIT_ERROR"
            | "PHP_URL_SCHEME"
            | "PHP_URL_HOST"
            | "PHP_URL_PORT"
            | "PHP_URL_USER"
            | "PHP_URL_PASS"
            | "PHP_URL_PATH"
            | "PHP_URL_QUERY"
            | "PHP_URL_FRAGMENT"
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

fn modeled_internal_has_by_ref_parameter(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_multisort"
            | "array_pop"
            | "array_push"
            | "array_shift"
            | "array_splice"
            | "array_unshift"
            | "array_walk"
            | "array_walk_recursive"
            | "arsort"
            | "asort"
            | "end"
            | "exec"
            | "fscanf"
            | "is_callable"
            | "krsort"
            | "ksort"
            | "natcasesort"
            | "natsort"
            | "next"
            | "parse_str"
            | "passthru"
            | "preg_match"
            | "preg_match_all"
            | "preg_replace"
            | "preg_replace_callback"
            | "prev"
            | "reset"
            | "rsort"
            | "settype"
            | "shuffle"
            | "similar_text"
            | "sort"
            | "sscanf"
            | "str_ireplace"
            | "str_replace"
            | "system"
            | "uasort"
            | "uksort"
            | "usort"
    )
}

fn is_reserved_compile_time_constant_declaration_name(name: &str) -> bool {
    name.rsplit('\\').next().is_some_and(|local| {
        matches!(
            local.to_ascii_lowercase().as_str(),
            "true" | "false" | "null"
        )
    })
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
                if !target_finally_path_stays_inside_current(finally_path, &target.finally_path) {
                    return Err(Diagnostic::new(
                        "jump out of a finally block is disallowed",
                        Some(*span),
                    ));
                }
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

fn target_finally_path_stays_inside_current(current: &[usize], target: &[usize]) -> bool {
    current.len() <= target.len()
        && current
            .iter()
            .zip(target)
            .all(|(current, target)| current == target)
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
        Expr::DynamicVariable { name, span } => Ok(ReferenceTarget::DynamicVariable { name, span }),
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
        Expr::DynamicStaticPropertyFetch {
            receiver,
            name,
            span,
        } => Ok(AssignmentTarget::DynamicStaticProperty {
            receiver,
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
        Expr::Call { span, .. } => Err(Diagnostic::new(
            "Can't use function return value in write context",
            Some(span),
        )),
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
            Expr::DynamicStaticPropertyFetch {
                receiver,
                name,
                span: property_span,
            } => {
                dimensions.reverse();
                return Ok(AssignmentTarget::DynamicStaticPropertyArrayDim {
                    receiver,
                    name,
                    dimensions,
                    span: combine_spans(property_span, span),
                });
            }
            Expr::Call { .. }
            | Expr::DynamicCall { .. }
            | Expr::MethodCall { .. }
            | Expr::DynamicMethodCall { .. } => {
                let value_span = current.span();
                dimensions.reverse();
                return Ok(AssignmentTarget::ValueArrayDim {
                    array: Box::new(current),
                    dimensions,
                    span: combine_spans(value_span, span),
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
        | AssignmentTarget::DynamicStaticPropertyArrayDim { span, .. }
        | AssignmentTarget::ValueArrayDim { span, .. }
        | AssignmentTarget::Property { span, .. }
        | AssignmentTarget::DynamicProperty { span, .. }
        | AssignmentTarget::StaticProperty { span, .. }
        | AssignmentTarget::DynamicStaticProperty { span, .. } => *span,
        AssignmentTarget::ArrayDim(target) => target.span,
        AssignmentTarget::List(target) => target.span,
    }
}

fn reference_target_span(target: &ReferenceTarget) -> SourceSpan {
    match target {
        ReferenceTarget::Variable { span, .. } => *span,
        ReferenceTarget::DynamicVariable { span, .. } => *span,
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

fn reject_this_foreach_target(target: &AssignmentTarget) -> Result<()> {
    match target {
        AssignmentTarget::Variable { name, span } if name.eq_ignore_ascii_case("this") => {
            Err(Diagnostic::new("Cannot re-assign $this", Some(*span)))
        }
        AssignmentTarget::List(list) => reject_this_foreach_list_target(list),
        _ => Ok(()),
    }
}

fn reject_this_foreach_list_target(target: &ListAssignmentTarget) -> Result<()> {
    for element in &target.elements {
        match &element.target {
            ListAssignmentElementTarget::Value(target) => reject_this_foreach_target(target)?,
            ListAssignmentElementTarget::Reference(target) => {
                reject_this_foreach_reference_target(target)?
            }
        }
    }
    Ok(())
}

fn reject_this_foreach_reference_target(target: &ReferenceTarget) -> Result<()> {
    match target {
        ReferenceTarget::Variable { name, span } if name.eq_ignore_ascii_case("this") => {
            Err(Diagnostic::new("Cannot re-assign $this", Some(*span)))
        }
        _ => Ok(()),
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
        AssignmentTarget::DynamicStaticPropertyArrayDim { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::ValueArrayDim { array, .. } => {
            if let Some(call_span) = modeled_internal_write_context_error_span(array) {
                return Err(Diagnostic::new(
                    "Cannot use result of built-in function in write context",
                    Some(call_span),
                ));
            }
            Err(Diagnostic::new(
                "null coalescing assignment currently supports variables, array/string offsets, and properties",
                Some(span),
            ))
        }
        AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::DynamicProperty { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::StaticProperty { .. } => Ok(()),
        AssignmentTarget::DynamicStaticProperty { .. } => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::List(_) => Err(Diagnostic::new(
            "null coalescing assignment currently supports variables and array/string offsets",
            Some(span),
        )),
    }
}

fn modeled_internal_write_context_error_span(expr: &Expr) -> Option<SourceSpan> {
    match expr {
        Expr::Call { name, span, .. }
            if is_modeled_internal_function_name(name)
                && !modeled_internal_has_by_ref_parameter(name) =>
        {
            Some(*span)
        }
        Expr::Grouped { expr, .. } => modeled_internal_write_context_error_span(expr),
        _ => None,
    }
}

fn validate_expression_assignment_target(
    op: AssignmentOp,
    target: &AssignmentTarget,
    span: SourceSpan,
) -> Result<()> {
    validate_coalesce_assignment_target(op, target, span)?;
    reject_this_assignment_target(target)?;

    if matches!(op, AssignmentOp::Assign | AssignmentOp::CoalesceAssign) {
        return Ok(());
    }

    match target {
        AssignmentTarget::Variable { .. } => Ok(()),
        AssignmentTarget::ArrayDim(_)
        | AssignmentTarget::DynamicArrayDim { .. }
        | AssignmentTarget::PropertyArrayDim { .. }
        | AssignmentTarget::StaticPropertyArrayDim { .. }
        | AssignmentTarget::DynamicStaticPropertyArrayDim { .. } => Ok(()),
        AssignmentTarget::ValueArrayDim { .. } => Err(Diagnostic::new(
            "compound assignment expressions currently support variables, array/string offsets, and properties",
            Some(span),
        )),
        AssignmentTarget::Property { .. } => Ok(()),
        AssignmentTarget::DynamicProperty { .. } => Ok(()),
        AssignmentTarget::StaticProperty { .. } => Ok(()),
        AssignmentTarget::DynamicStaticProperty { .. } => Ok(()),
        AssignmentTarget::DynamicVariable { .. } | AssignmentTarget::List(_) => Err(
            Diagnostic::new(
                "compound assignment expressions currently support variables, array/string offsets, and properties",
                Some(span),
            ),
        ),
    }
}

fn reject_this_assignment_target(target: &AssignmentTarget) -> Result<()> {
    match target {
        AssignmentTarget::Variable { name, span } if name.eq_ignore_ascii_case("this") => {
            Err(Diagnostic::new("Cannot re-assign $this", Some(*span)))
        }
        AssignmentTarget::List(list) => reject_this_assignment_list_target(list),
        _ => Ok(()),
    }
}

fn reject_this_assignment_list_target(target: &ListAssignmentTarget) -> Result<()> {
    for element in &target.elements {
        match &element.target {
            ListAssignmentElementTarget::Value(target) => reject_this_assignment_target(target)?,
            ListAssignmentElementTarget::Reference(target) => {
                reject_this_assignment_reference_target(target)?
            }
        }
    }
    Ok(())
}

fn reject_this_assignment_reference_target(target: &ReferenceTarget) -> Result<()> {
    match target {
        ReferenceTarget::Variable { name, span } if name.eq_ignore_ascii_case("this") => {
            Err(Diagnostic::new("Cannot re-assign $this", Some(*span)))
        }
        _ => Ok(()),
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
        AssignmentTarget::DynamicStaticPropertyArrayDim { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::ValueArrayDim { .. } => {
            return Err(Diagnostic::new(
                "unsupported by-reference assignment target",
                Some(span),
            ));
        }
        AssignmentTarget::Property { .. } => {}
        AssignmentTarget::DynamicProperty { .. } => {}
        AssignmentTarget::StaticProperty { .. } => {
            return Ok(());
        }
        AssignmentTarget::DynamicStaticProperty { .. } => {
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
                return Err(Diagnostic::new("Cannot use [] for reading", Some(*span)));
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
        Expr::DynamicStaticPropertyFetch { receiver, .. } => {
            reject_append_array_read(receiver)?;
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
        Expr::YieldFrom { expr, .. } => reject_append_array_read(expr)?,
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

fn reject_append_array_read_in_optional_exprs(dimensions: &[Option<Expr>]) -> Result<()> {
    for dimension in dimensions {
        if let Some(dimension) = dimension {
            reject_append_array_read(dimension)?;
        }
    }
    Ok(())
}

fn reject_append_array_read_in_array_dim_target(target: &ArrayDimTarget) -> Result<()> {
    reject_append_array_read_in_optional_exprs(&target.dimensions)
}

fn reject_append_array_read_in_reference_target(target: &ReferenceTarget) -> Result<()> {
    match target {
        ReferenceTarget::Variable { .. } => {}
        ReferenceTarget::DynamicVariable { name, .. } => {
            reject_append_array_read(name)?;
        }
        ReferenceTarget::ArrayDim(target) => {
            reject_append_array_read_in_array_dim_target(target)?;
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        ReferenceTarget::Property { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read(name)?;
        }
    }
    Ok(())
}

fn reject_append_array_read_in_list_assignment_target(target: &ListAssignmentTarget) -> Result<()> {
    for element in &target.elements {
        if let Some(key) = &element.key {
            reject_append_array_read(key)?;
        }
        match &element.target {
            ListAssignmentElementTarget::Value(target) => {
                reject_append_array_read_in_assignment_target(target)?;
            }
            ListAssignmentElementTarget::Reference(target) => {
                reject_append_array_read_in_reference_target(target)?;
            }
        }
    }
    Ok(())
}

fn reject_append_array_read_in_assignment_target(target: &AssignmentTarget) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. } | AssignmentTarget::StaticProperty { .. } => {}
        AssignmentTarget::DynamicStaticProperty { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        AssignmentTarget::DynamicVariable { name, .. } => {
            reject_append_array_read(name)?;
        }
        AssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            reject_append_array_read(name)?;
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::ArrayDim(target) => {
            reject_append_array_read_in_array_dim_target(target)?;
        }
        AssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            reject_append_array_read(array)?;
            reject_append_array_read_in_optional_exprs(dimensions)?;
        }
        AssignmentTarget::Property { receiver, .. } => {
            reject_append_array_read(receiver)?;
        }
        AssignmentTarget::DynamicProperty { receiver, name, .. } => {
            reject_append_array_read(receiver)?;
            reject_append_array_read(name)?;
        }
        AssignmentTarget::List(target) => {
            reject_append_array_read_in_list_assignment_target(target)?;
        }
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
        TokenStringPart::PropertyChain {
            variable,
            properties,
        } => StringPart::PropertyChain {
            variable,
            properties,
        },
        TokenStringPart::MethodCall { variable, method } => {
            StringPart::MethodCall { variable, method }
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

fn constant_expression_class_name_fetch_error_type(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::Grouped { expr, .. } => constant_expression_class_name_fetch_error_type(expr),
        Expr::Binary {
            op, left, right, ..
        } => {
            let left_type = scalar_constant_expr_type(left)?;
            let right_type = scalar_constant_expr_type(right)?;
            match op {
                BinaryOp::Concat => Some("string"),
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Identical
                | BinaryOp::NotIdentical
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::And
                | BinaryOp::Xor
                | BinaryOp::Or => Some("bool"),
                BinaryOp::Divide => Some("float"),
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Power => {
                    if left_type == "float" || right_type == "float" {
                        Some("float")
                    } else {
                        Some("int")
                    }
                }
                BinaryOp::Modulo
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftRight
                | BinaryOp::BitwiseAnd
                | BinaryOp::BitwiseXor
                | BinaryOp::BitwiseOr
                | BinaryOp::Spaceship => Some("int"),
                BinaryOp::Coalesce => None,
            }
        }
        Expr::Unary { op, expr, .. } => {
            let inner = scalar_constant_expr_type(expr)?;
            match op {
                UnaryOp::Positive | UnaryOp::Negate => Some(inner),
                UnaryOp::Not => Some("bool"),
                UnaryOp::BitwiseNot => Some(if inner == "string" { "string" } else { "int" }),
                UnaryOp::ErrorSuppress => None,
            }
        }
        Expr::Cast { kind, expr, .. } => {
            scalar_constant_expr_type(expr)?;
            match kind {
                CastKind::Int | CastKind::Integer => Some("int"),
                CastKind::Float | CastKind::Double => Some("float"),
                CastKind::String | CastKind::Binary => Some("string"),
                CastKind::Bool | CastKind::Boolean => Some("bool"),
                CastKind::Array | CastKind::Object | CastKind::Void => None,
            }
        }
        _ => None,
    }
}

fn scalar_constant_expr_type(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::Grouped { expr, .. } => scalar_constant_expr_type(expr),
        Expr::String(_, _) | Expr::ShellExec { .. } => Some("string"),
        Expr::Int(_, _) => Some("int"),
        Expr::Float(_, _) => Some("float"),
        Expr::Bool(_, _) => Some("bool"),
        Expr::Null(_) => Some("null"),
        Expr::Binary { .. } | Expr::Unary { .. } | Expr::Cast { .. } => {
            constant_expression_class_name_fetch_error_type(expr)
        }
        _ => None,
    }
}

fn is_supported_global_const_expr(expr: &Expr) -> bool {
    is_supported_global_const_expr_with_options(expr, false, false)
}

fn is_supported_const_declaration_expr(expr: &Expr) -> bool {
    is_supported_global_const_expr_with_options(expr, true, true)
}

fn is_supported_global_const_expr_with_options(
    expr: &Expr,
    allow_const_array_unpack_error_operands: bool,
    allow_array_access: bool,
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
                    allow_array_access,
                )
            }) && match &element.value {
                ArrayElementValue::Hole(_) => false,
                ArrayElementValue::Value(value) => is_supported_global_const_expr_with_options(
                    value,
                    allow_const_array_unpack_error_operands,
                    allow_array_access,
                ),
                ArrayElementValue::Unpack(value) => {
                    is_supported_global_const_expr_with_options(
                        value,
                        allow_const_array_unpack_error_operands,
                        allow_array_access,
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
                allow_array_access,
            )
        }
        Expr::FirstClassCallable { callable, .. } => {
            is_supported_first_class_callable_const_target(callable)
        }
        Expr::Binary { left, right, .. } => {
            is_supported_global_const_expr_with_options(
                left,
                allow_const_array_unpack_error_operands,
                allow_array_access,
            ) && is_supported_global_const_expr_with_options(
                right,
                allow_const_array_unpack_error_operands,
                allow_array_access,
            )
        }
        Expr::ArrayAccess {
            array,
            index: Some(index),
            ..
        } if allow_array_access => {
            is_supported_global_const_expr_with_options(
                array,
                allow_const_array_unpack_error_operands,
                allow_array_access,
            ) && is_supported_global_const_expr_with_options(
                index,
                allow_const_array_unpack_error_operands,
                allow_array_access,
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
        | Expr::YieldFrom { .. }
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
        | Expr::DynamicStaticPropertyFetch { .. }
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

fn const_expr_contains_new_object(expr: &Expr) -> bool {
    match expr {
        Expr::NewObject { .. } | Expr::DynamicNewObject { .. } => true,
        Expr::Grouped { expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::ArrayAccess {
            array: expr,
            index: None,
            ..
        } => const_expr_contains_new_object(expr),
        Expr::ArrayAccess {
            array,
            index: Some(index),
            ..
        } => const_expr_contains_new_object(array) || const_expr_contains_new_object(index),
        Expr::Binary { left, right, .. } => {
            const_expr_contains_new_object(left) || const_expr_contains_new_object(right)
        }
        Expr::Array { elements, .. } => elements.iter().any(|element| {
            element
                .key
                .as_ref()
                .is_some_and(const_expr_contains_new_object)
                || match &element.value {
                    ArrayElementValue::Value(value) | ArrayElementValue::Unpack(value) => {
                        const_expr_contains_new_object(value)
                    }
                    ArrayElementValue::Reference(target) => {
                        reference_target_contains_new_object(target)
                    }
                    ArrayElementValue::Hole(_) => false,
                }
        }),
        _ => false,
    }
}

fn reference_target_contains_new_object(target: &ReferenceTarget) -> bool {
    match target {
        ReferenceTarget::DynamicVariable { name, .. } => const_expr_contains_new_object(name),
        ReferenceTarget::ArrayDim(target) => target.dimensions.iter().any(|dimension| {
            dimension
                .as_ref()
                .is_some_and(const_expr_contains_new_object)
        }),
        ReferenceTarget::Property { receiver, .. } => const_expr_contains_new_object(receiver),
        ReferenceTarget::DynamicProperty { receiver, name, .. } => {
            const_expr_contains_new_object(receiver) || const_expr_contains_new_object(name)
        }
        ReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => {
            const_expr_contains_new_object(receiver)
                || dimensions.iter().any(|dimension| {
                    dimension
                        .as_ref()
                        .is_some_and(const_expr_contains_new_object)
                })
        }
        ReferenceTarget::Variable { .. } => false,
    }
}

fn is_supported_property_default_expr(expr: &Expr) -> bool {
    match expr {
        Expr::FirstClassCallable { .. } => false,
        Expr::Grouped { expr, .. } => is_supported_property_default_expr(expr),
        _ => is_supported_global_const_expr(expr),
    }
}

fn token_string_parts_use_this_property(parts: &[TokenStringPart], property_name: &str) -> bool {
    parts.iter().any(|part| match part {
        TokenStringPart::PropertyFetch { variable, property } => {
            variable.eq_ignore_ascii_case("this") && property == property_name
        }
        TokenStringPart::PropertyChain {
            variable,
            properties,
        } => {
            variable.eq_ignore_ascii_case("this")
                && properties
                    .first()
                    .is_some_and(|property| property == property_name)
        }
        _ => false,
    })
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
        Expr::ArrayAccess {
            array,
            index: Some(index),
            ..
        } => {
            is_supported_parameter_default_expr(array) && is_supported_parameter_default_expr(index)
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
        | Expr::YieldFrom { .. }
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
        | Expr::DynamicStaticPropertyFetch { .. }
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
        | Expr::DynamicStaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. }
        | Expr::DynamicClassNameFetch { .. }
        | Expr::List(_)
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. }
        | Expr::Print { .. }
        | Expr::Include { .. }
        | Expr::Throw { .. }
        | Expr::Yield { .. }
        | Expr::YieldFrom { .. } => Ok(()),
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
