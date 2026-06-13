use std::collections::{HashMap, HashSet};

use crate::ast::{
    AnonymousFunction, ArrayDimTarget, ArrayElement, ArrayElementValue, AssignmentOp,
    AssignmentTarget, BinaryOp, CastKind, CatchClause, ClassConstantDecl, ClassDecl,
    ClosureUseCapture, ConstDeclaration, Expr, FunctionDecl, FunctionParameter, IncDecOp,
    IncDecResult, IncDecTarget, IncludeKind, ListAssignmentElement, ListAssignmentElementTarget,
    ListAssignmentTarget, MagicConstantKind, MethodDecl, Program, PropertyDecl, PropertyVisibility,
    ReferenceTarget, Statement, StaticPropertyDecl, StringInterpolationIndex, StringPart,
    SwitchCase, TypeHint, UnaryOp, UnsetTarget,
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
        current_namespace: None,
        seen_namespace_declaration: false,
        namespace_declaration_style: None,
        class_aliases: HashMap::new(),
        function_aliases: HashMap::new(),
        constant_aliases: HashMap::new(),
    }
    .parse_program()
}

struct Parser {
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

struct ClassModifiers {
    is_static: bool,
    visibility: PropertyVisibility,
    visibility_span: Option<SourceSpan>,
}

impl Default for ClassModifiers {
    fn default() -> Self {
        Self {
            is_static: false,
            visibility: PropertyVisibility::Public,
            visibility_span: None,
        }
    }
}

enum ParsedClassMember {
    Method(MethodDecl),
    Properties(Vec<PropertyDecl>),
    StaticProperties(Vec<StaticPropertyDecl>),
    Constants(Vec<ClassConstantDecl>),
}

impl Parser {
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
        let mut functions = Vec::new();
        let mut statements = Vec::new();
        self.parse_top_level_items(
            &mut classes,
            &mut functions,
            &mut statements,
            TopLevelScope::Program,
        )?;
        validate_class_names(&classes)?;
        validate_parent_class_names(&classes)?;
        for class in &classes {
            validate_method_names(class)?;
            validate_class_constant_names(class)?;
            for method in &class.methods {
                if method.return_by_ref {
                    validate_by_reference_returns_in_statements(
                        &method.body,
                        &format!("{}::{}", class.name, method.name),
                    )?;
                }
                if method.return_type == Some(TypeHint::Void) {
                    validate_void_returns_in_statements(&method.body)?;
                }
                validate_anonymous_functions_in_statements(&method.body, &functions)?;
                validate_reference_assignment_sources(&method.body, &functions)?;
                validate_goto_labels(&method.body)?;
            }
        }
        validate_function_names(&functions)?;
        validate_by_reference_returns(&functions)?;
        validate_void_returns(&functions)?;
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

    fn parse_top_level_items(
        &mut self,
        classes: &mut Vec<ClassDecl>,
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
            if token_is_identifier_named(self.peek(), "namespace") {
                if scope == TopLevelScope::NamespaceBlock {
                    return Err(Diagnostic::new(
                        "Namespace declarations cannot be nested",
                        Some(self.peek().span),
                    ));
                }
                if !self.seen_namespace_declaration
                    && (!classes.is_empty() || !functions.is_empty() || !statements.is_empty())
                {
                    return Err(Diagnostic::new(
                        "Namespace declaration statement has to be the very first statement or after any declare call in the script",
                        Some(self.peek().span),
                    ));
                }
                self.parse_namespace_declaration(classes, functions, statements)?;
            } else if token_is_identifier_named(self.peek(), "use") {
                self.reject_code_outside_bracketed_namespace(scope)?;
                self.parse_use_declarations()?;
            } else if self.peek_starts_function_decl() {
                self.reject_code_outside_bracketed_namespace(scope)?;
                functions.push(self.parse_function_decl()?);
            } else if token_is_identifier_named(self.peek(), "class") {
                self.reject_code_outside_bracketed_namespace(scope)?;
                classes.push(self.parse_class_decl()?);
            } else {
                self.reject_code_outside_bracketed_namespace(scope)?;
                statements.push(self.parse_statement()?);
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
            return self.parse_bracketed_namespace_block(namespace, classes, functions, statements);
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
            NameResolution::NamespaceRelative => Err(Diagnostic::new(
                "namespace declarations cannot use namespace-relative names",
                Some(parsed.span),
            )),
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
        let alias = if matches!(self.peek().kind, TokenKind::As) {
            self.advance();
            let token = self.advance().clone();
            let TokenKind::Identifier(alias) = token.kind else {
                return Err(Diagnostic::new("expected import alias", Some(token.span)));
            };
            alias
        } else {
            target
                .name
                .rsplit('\\')
                .next()
                .unwrap_or(&target.name)
                .to_string()
        };
        self.register_use_import(kind, target, alias);
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
            let alias = if matches!(self.peek().kind, TokenKind::As) {
                self.advance();
                let token = self.advance().clone();
                let TokenKind::Identifier(alias) = token.kind else {
                    return Err(Diagnostic::new("expected import alias", Some(token.span)));
                };
                alias
            } else {
                item.name
                    .rsplit('\\')
                    .next()
                    .unwrap_or(&item.name)
                    .to_string()
            };
            let grouped_target = ParsedName {
                name: format!("{}\\{}", prefix.name, item.name),
                span: combine_spans(prefix.span, item.span),
                resolution: prefix.resolution,
            };
            self.register_use_import(item_kind, grouped_target, alias);
            if !matches!(self.peek().kind, TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.expect_right_brace()?;
        Ok(())
    }

    fn register_use_import(&mut self, kind: UseDeclarationKind, target: ParsedName, alias: String) {
        let target_name = match target.resolution {
            NameResolution::FullyQualified => target.name,
            NameResolution::NamespaceRelative => self.qualify_current_namespace(&target.name),
            NameResolution::Unqualified | NameResolution::Qualified => target.name,
        };
        let alias_key = alias.to_ascii_lowercase();
        match kind {
            UseDeclarationKind::Class => {
                self.class_aliases.insert(alias_key, target_name);
            }
            UseDeclarationKind::Function => {
                self.function_aliases.insert(alias_key, target_name);
            }
            UseDeclarationKind::Constant => {
                self.constant_aliases.insert(alias_key, target_name);
            }
        }
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
        Ok((self.resolve_class_name(&parsed), span))
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
                if let Some(target) = self.function_aliases.get(&alias_key) {
                    target.clone()
                } else if is_modeled_internal_function_name(&alias_key) {
                    alias_key
                } else {
                    self.qualify_current_namespace(&parsed.name)
                }
            }
            NameResolution::Qualified => self.qualify_current_namespace(&parsed.name),
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
            NameResolution::Qualified => self.qualify_current_namespace(&parsed.name),
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

    fn parse_class_decl(&mut self) -> Result<ClassDecl> {
        let class_token = self.advance().clone();
        let TokenKind::Identifier(keyword) = &class_token.kind else {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        };
        if !keyword.eq_ignore_ascii_case("class") {
            return Err(Diagnostic::new("expected class", Some(class_token.span)));
        }

        let (class_name, _) = self.parse_declaration_name("expected class name")?;
        let parent_name = if token_is_identifier_named(self.peek(), "extends") {
            self.advance();
            Some(
                self.parse_resolved_class_name("expected parent class name")?
                    .0,
            )
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
        let mut properties = Vec::new();
        let mut static_properties = Vec::new();
        let mut constants = Vec::new();
        let mut methods = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RightBrace | TokenKind::Eof) {
            match self.parse_class_member()? {
                ParsedClassMember::Method(method) => methods.push(method),
                ParsedClassMember::Properties(parsed_properties) => {
                    properties.extend(parsed_properties);
                }
                ParsedClassMember::StaticProperties(properties) => {
                    static_properties.extend(properties);
                }
                ParsedClassMember::Constants(parsed_constants) => {
                    constants.extend(parsed_constants);
                }
            }
        }
        self.expect_right_brace()?;
        Ok(ClassDecl {
            name: class_name,
            parent_name,
            properties,
            static_properties,
            constants,
            methods,
            span: class_token.span,
        })
    }

    fn parse_class_member(&mut self) -> Result<ParsedClassMember> {
        let modifiers = self.parse_class_modifiers()?;
        if matches!(self.peek().kind, TokenKind::Const) {
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
            return Ok(ParsedClassMember::Constants(
                self.parse_class_constant_declarations(modifiers.visibility)?,
            ));
        }
        if matches!(self.peek().kind, TokenKind::Variable(_)) {
            if modifiers.is_static {
                return Ok(ParsedClassMember::StaticProperties(
                    self.parse_static_property_declarations(modifiers.visibility)?,
                ));
            }
            return Ok(ParsedClassMember::Properties(
                self.parse_property_declarations(modifiers.visibility)?,
            ));
        }
        if !matches!(self.peek().kind, TokenKind::Function) {
            return Err(Diagnostic::new(
                "unsupported class member",
                Some(self.peek().span),
            ));
        }
        if modifiers.visibility != PropertyVisibility::Public {
            return Err(Diagnostic::new(
                "non-public class methods are unsupported",
                modifiers.visibility_span,
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
                    modifiers.visibility = PropertyVisibility::Public;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "static" => {
                    modifiers.is_static = true;
                    self.advance();
                }
                "private" => {
                    modifiers.visibility = PropertyVisibility::Private;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
                }
                "protected" => {
                    modifiers.visibility = PropertyVisibility::Protected;
                    modifiers.visibility_span = Some(self.peek().span);
                    self.advance();
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

    fn parse_static_property_declarations(
        &mut self,
        visibility: PropertyVisibility,
    ) -> Result<Vec<StaticPropertyDecl>> {
        let mut properties = vec![self.parse_static_property_declaration(visibility)?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            properties.push(self.parse_static_property_declaration(visibility)?);
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
        if !is_supported_global_const_expr(&value) {
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
    ) -> Result<Vec<PropertyDecl>> {
        let mut properties = vec![self.parse_property_declaration(visibility)?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            properties.push(self.parse_property_declaration(visibility)?);
        }
        self.expect_semicolon()?;
        Ok(properties)
    }

    fn parse_property_declaration(
        &mut self,
        visibility: PropertyVisibility,
    ) -> Result<PropertyDecl> {
        let token = self.advance().clone();
        let TokenKind::Variable(name) = token.kind else {
            return Err(Diagnostic::new("expected property name", Some(token.span)));
        };
        let value = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();
            let value = self.parse_expr()?;
            if !is_supported_global_const_expr(&value) {
                return Err(Diagnostic::new(
                    "property default value must be a supported constant expression",
                    Some(value.span()),
                ));
            }
            Some(value)
        } else {
            None
        };
        Ok(PropertyDecl {
            name,
            visibility,
            value,
            span: token.span,
        })
    }

    fn parse_static_property_declaration(
        &mut self,
        visibility: PropertyVisibility,
    ) -> Result<StaticPropertyDecl> {
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
            visibility,
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
            Some(self.parse_return_type_hint()?)
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
            TokenKind::Try => self.parse_try(),
            TokenKind::Goto => self.parse_goto(),
            TokenKind::Const => self.parse_const(),
            TokenKind::Global => self.parse_global(),
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

    fn parse_empty_statement(&mut self) -> Result<Statement> {
        let token = self.advance().clone();
        Ok(Statement::Empty { span: token.span })
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
        let (name, _) = self.parse_declaration_name("expected function name")?;
        let parameters = self.parse_function_parameters()?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
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
        let captures = self.parse_closure_use_captures()?;
        let return_type = if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            Some(self.parse_return_type_hint()?)
        } else {
            None
        };
        self.function_depth += 1;
        let body = self.parse_block();
        self.function_depth -= 1;
        let body = body?;
        Ok(Expr::AnonymousFunction(AnonymousFunction {
            parameters,
            captures,
            return_type,
            return_by_ref,
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
        loop {
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
            captures.push(ClosureUseCapture {
                name: name.clone(),
                by_ref,
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

    fn parse_return_type_hint(&mut self) -> Result<TypeHint> {
        let token = self.advance();
        match &token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("void") => Ok(TypeHint::Void),
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

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        if let Some(void_span) = self.try_parse_void_cast_statement_prefix() {
            let expression = self.parse_expr()?;
            let span = combine_spans(void_span, expression.span());
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

    fn parse_const_declaration(&mut self) -> Result<ConstDeclaration> {
        let (name, token_span) = self.parse_declaration_name("expected constant name")?;
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
        let target = assignment_target_from_expr(target_expr)
            .map_err(|_| Diagnostic::new("expected foreach variable", Some(target_span)))?;
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
        let (arguments, argument_names, _) = self.parse_call_arguments()?;
        validate_mutating_array_internal_call(&name, &arguments, token_span)?;
        Ok(Statement::Call {
            name,
            arguments,
            argument_names,
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
        Ok(self
            .parse_resolved_class_name("expected catch type name")?
            .0)
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
        let (name, span) = self.parse_resolved_function_name("expected function name")?;
        let (arguments, argument_names, _) = self.parse_call_arguments()?;
        validate_mutating_array_internal_call(&name, &arguments, span)?;
        self.expect_statement_terminator()?;
        Ok(Statement::Call {
            name,
            arguments,
            argument_names,
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
            Expr::ArrayAccess { .. } => unset_array_dim_target_from_expr(target),
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
            reject_append_array_read(&left)?;
            return Ok(left);
        }

        let operator = self.peek().clone();
        let op = self.expect_assignment_op()?;
        let left_span = left.span();
        let target = assignment_target_from_expr(left).map_err(|_| {
            Diagnostic::new(
                "assignment expression target must be a variable, array dimension, or list",
                Some(operator.span),
            )
        })?;
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
            TokenKind::PlusPlus | TokenKind::MinusMinus => self.parse_prefix_increment_expr(),
            TokenKind::Print => self.parse_print_expr(),
            TokenKind::Include => self.parse_include_expr(IncludeKind::Include),
            TokenKind::IncludeOnce => self.parse_include_expr(IncludeKind::IncludeOnce),
            TokenKind::Require => self.parse_include_expr(IncludeKind::Require),
            TokenKind::RequireOnce => self.parse_include_expr(IncludeKind::RequireOnce),
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
                    let (arguments, argument_names, right_span) = self.parse_call_arguments()?;
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        name: name.to_ascii_lowercase(),
                        arguments,
                        argument_names,
                        span: combine_spans(start_span, right_span),
                    };
                }
                TokenKind::LeftParen => {
                    let start_span = expr.span();
                    let (arguments, argument_names, right_span) = self.parse_call_arguments()?;
                    expr = Expr::DynamicCall {
                        callee: Box::new(expr),
                        arguments,
                        argument_names,
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
            TokenKind::Dollar => self.parse_dynamic_variable_expr(token.span),
            TokenKind::Function => self.parse_anonymous_function_expr(token.span),
            TokenKind::New => self.parse_new_object_expr(token.span),
            TokenKind::Identifier(name) => {
                let parsed_name =
                    self.parse_name_from_first(name, token.span, None, "expected name")?;
                let unqualified = matches!(parsed_name.resolution, NameResolution::Unqualified);
                let lowercase = parsed_name.name.to_ascii_lowercase();
                if matches!(self.peek().kind, TokenKind::DoubleColon) {
                    let class_name = self.resolve_class_name(&parsed_name);
                    self.parse_static_member_expr(class_name, parsed_name.span)
                } else if matches!(self.peek().kind, TokenKind::LeftParen) {
                    match (unqualified, lowercase.as_str()) {
                        (true, "array") => self.parse_long_array_literal(parsed_name.span),
                        (true, "isset") => self.parse_isset_expr(parsed_name.span),
                        (true, "empty") => self.parse_empty_expr(parsed_name.span),
                        _ => {
                            let (arguments, argument_names, right_span) =
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
                    let (arguments, argument_names, right_span) = self.parse_call_arguments()?;
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
                        span: combine_spans(parsed_name.span, right_span),
                    });
                }
                Ok(Expr::Constant(
                    self.resolve_constant_name(&parsed_name),
                    parsed_name.span,
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
            return Ok(Expr::ClassConstantFetch {
                class_name,
                name: member_name,
                span: combine_spans(class_span, member.span),
            });
        }
        let (arguments, argument_names, right_span) = self.parse_call_arguments()?;
        Ok(Expr::Call {
            name: format!("{}::{}", class_name, member_name),
            arguments,
            argument_names,
            span: combine_spans(class_span, right_span),
        })
    }

    fn parse_new_object_expr(&mut self, start_span: SourceSpan) -> Result<Expr> {
        let (class_name, class_span) = self.parse_new_object_class_name()?;
        let mut span = combine_spans(start_span, class_span);
        let (arguments, argument_names) = if matches!(self.peek().kind, TokenKind::LeftParen) {
            let (arguments, argument_names, right_span) = self.parse_call_arguments()?;
            span = combine_spans(start_span, right_span);
            (arguments, argument_names)
        } else {
            (Vec::new(), Vec::new())
        };
        Ok(Expr::NewObject {
            class_name,
            arguments,
            argument_names,
            span,
        })
    }

    fn parse_new_object_class_name(&mut self) -> Result<(String, SourceSpan)> {
        self.parse_resolved_class_name("expected class name")
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
        let (targets, argument_names, right_span) = self.parse_call_arguments()?;
        reject_named_language_construct_arguments(&argument_names, start_span)?;
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
        let (mut arguments, argument_names, right_span) = self.parse_call_arguments()?;
        reject_named_language_construct_arguments(&argument_names, start_span)?;
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

    fn parse_call_arguments(&mut self) -> Result<(Vec<Expr>, Vec<Option<String>>, SourceSpan)> {
        self.expect_left_paren()?;
        let mut arguments = Vec::new();
        let mut argument_names = Vec::new();
        let mut named_arguments = HashSet::new();
        let mut seen_named_argument = false;
        if !matches!(self.peek().kind, TokenKind::RightParen) {
            let (name, argument, span) = self.parse_call_argument()?;
            if let Some(name) = &name {
                seen_named_argument = true;
                if !named_arguments.insert(name.clone()) {
                    return Err(Diagnostic::new(
                        format!("Named parameter ${name} overwrites previous argument"),
                        Some(span),
                    ));
                }
            }
            arguments.push(argument);
            argument_names.push(name);
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                let (name, argument, span) = self.parse_call_argument()?;
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
                }
                arguments.push(argument);
                argument_names.push(name);
            }
        }
        let right_span = self.expect_right_paren()?;
        Ok((arguments, argument_names, right_span))
    }

    fn parse_call_argument(&mut self) -> Result<(Option<String>, Expr, SourceSpan)> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if matches!(
                self.tokens.get(self.index + 1).map(|token| &token.kind),
                Some(TokenKind::Colon)
            ) {
                let name = name.clone();
                let name_span = self.advance().span;
                self.expect_colon()?;
                let value = self.parse_expr()?;
                return Ok((Some(name), value, name_span));
            }
        }

        let value = self.parse_expr()?;
        let span = value.span();
        Ok((None, value, span))
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
        TokenKind::Include => "include",
        TokenKind::IncludeOnce => "include_once",
        TokenKind::Require => "require",
        TokenKind::RequireOnce => "require_once",
        TokenKind::Try => "try",
        TokenKind::Catch => "catch",
        TokenKind::Goto => "goto",
        TokenKind::Const => "const",
        TokenKind::Function => "function",
        TokenKind::Global => "global",
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
        AssignmentTarget::DynamicVariable { .. } => return Ok(()),
        AssignmentTarget::DynamicArrayDim { .. } => return Ok(()),
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
        Expr::Assign { value, .. }
        | Expr::AssignRef { source: value, .. }
        | Expr::Print {
            expression: value, ..
        }
        | Expr::Include { path: value, .. }
        | Expr::Grouped { expr: value, .. } => {
            expr_array_literal_reference_to_variable(value, variable)
        }
        Expr::DynamicVariable { name, .. } => {
            expr_array_literal_reference_to_variable(name, variable)
        }
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
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
        | Expr::NewObject { .. }
        | Expr::PropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. }
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

fn validate_void_returns(functions: &[FunctionDecl]) -> Result<()> {
    for function in functions {
        if function.return_type == Some(TypeHint::Void) {
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
            Statement::Try { body, catches, .. } => {
                validate_void_returns_in_statements(body)?;
                for catch in catches {
                    validate_void_returns_in_statements(&catch.body)?;
                }
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
            | Statement::Empty { .. }
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
            if function.return_type == Some(TypeHint::Void) {
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
        Expr::StaticPropertyFetch { .. } | Expr::ClassConstantFetch { .. } => {}
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
        Expr::Print {
            expression: expr, ..
        }
        | Expr::DynamicVariable { name: expr, .. }
        | Expr::Include { path: expr, .. }
        | Expr::Unary { expr, .. }
        | Expr::Cast { expr, .. }
        | Expr::Grouped { expr, .. } => validate_anonymous_functions_in_expr(expr, functions)?,
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
            | "strcasecmp"
            | "strncasecmp"
            | "strncmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "strpos"
            | "stripos"
            | "strrpos"
            | "strripos"
            | "strstr"
            | "stristr"
            | "substr_count"
            | "str_pad"
            | "str_repeat"
            | "str_split"
            | "strrev"
            | "stripcslashes"
            | "stripslashes"
            | "strtolower"
            | "strtoupper"
            | "ucfirst"
            | "lcfirst"
            | "trim"
            | "ltrim"
            | "rtrim"
            | "quotemeta"
            | "chunk_split"
            | "strip_tags"
            | "crc32"
            | "md5"
            | "sha1"
            | "sha1_file"
            | "substr"
            | "dirname"
            | "bin2hex"
            | "hex2bin"
            | "quoted_printable_decode"
            | "soundex"
            | "sprintf"
            | "printf"
            | "json_encode"
            | "ceil"
            | "floor"
            | "abs"
            | "sqrt"
            | "pow"
            | "fdiv"
            | "fclose"
            | "file_exists"
            | "file_get_contents"
            | "file_put_contents"
            | "fopen"
            | "pathinfo"
            | "stream_get_meta_data"
            | "get_cfg_var"
            | "get_loaded_extensions"
            | "highlight_file"
            | "highlight_string"
            | "ini_get"
            | "intdiv"
            | "assert"
            | "basename"
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
            | "zend_version"
            | "var_export"
            | "bindec"
            | "hexdec"
            | "implode"
            | "in_array"
            | "ob_get_contents"
            | "octdec"
            | "boolval"
            | "doubleval"
            | "floatval"
            | "intval"
            | "chr"
            | "ord"
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
            | "is_file"
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
            | "localeconv"
            | "define"
            | "constant"
            | "defined"
            | "extension_loaded"
            | "function_exists"
            | "get_class"
            | "isset"
            | "empty"
            | "count"
            | "sizeof"
            | "array_change_key_case"
            | "array_chunk"
            | "array_column"
            | "array_combine"
            | "array_count_values"
            | "array_diff"
            | "array_diff_assoc"
            | "array_fill"
            | "array_fill_keys"
            | "array_filter"
            | "array_flip"
            | "array_intersect"
            | "array_intersect_assoc"
            | "array_is_list"
            | "array_key_exists"
            | "array_key_first"
            | "array_key_last"
            | "array_keys"
            | "array_map"
            | "array_merge"
            | "array_merge_recursive"
            | "array_pad"
            | "array_pop"
            | "array_product"
            | "array_push"
            | "array_reduce"
            | "array_replace_recursive"
            | "array_reverse"
            | "array_search"
            | "array_shift"
            | "array_slice"
            | "array_sum"
            | "array_udiff"
            | "array_udiff_assoc"
            | "array_udiff_uassoc"
            | "array_unshift"
            | "array_values"
            | "array_walk"
            | "shuffle"
            | "sort"
            | "rsort"
            | "current"
            | "end"
            | "key"
            | "next"
            | "prev"
            | "reset"
            | "mkdir"
            | "rmdir"
            | "scandir"
            | "setlocale"
            | "str_shuffle"
            | "str_replace"
            | "strrchr"
            | "strtr"
            | "unlink"
            | "call_user_func"
            | "call_user_func_array"
            | "class_exists"
            | "debug_zval_dump"
            | "is_callable"
            | "method_exists"
            | "property_exists"
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
            | "CASE_LOWER"
            | "CASE_UPPER"
            | "ARRAY_FILTER_USE_BOTH"
            | "ARRAY_FILTER_USE_KEY"
            | "PATHINFO_DIRNAME"
            | "PATHINFO_BASENAME"
            | "PATHINFO_EXTENSION"
            | "PATHINFO_FILENAME"
            | "PATHINFO_ALL"
            | "LC_ALL"
            | "LC_COLLATE"
            | "LC_CTYPE"
            | "LC_MESSAGES"
            | "LC_MONETARY"
            | "LC_NUMERIC"
            | "LC_TIME"
            | "M_E"
            | "M_LOG2E"
            | "M_LOG10E"
            | "M_LN2"
            | "M_LN10"
            | "PHP_INT_MIN"
            | "PHP_INT_MAX"
            | "PHP_INT_SIZE"
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
    )
}

fn is_single_array_path_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "array_pop" | "array_shift"
    )
}

fn is_unsupported_sort_family_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "usort" | "uasort" | "uksort" | "array_multisort"
    )
}

fn is_regular_sort_flag_mutation_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "arsort" | "asort" | "krsort" | "ksort" | "rsort" | "sort"
    )
}

fn is_sort_regular_flag_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Int(0, _) => true,
        Expr::Constant(name, _) => name == "SORT_REGULAR",
        Expr::Grouped { expr, .. } => is_sort_regular_flag_expr(expr),
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

fn is_array_cursor_mutation_argument(expr: &Expr) -> bool {
    is_direct_variable_argument(expr) || is_variable_array_access_argument(expr)
}

fn validate_mutating_array_internal_call(
    name: &str,
    arguments: &[Expr],
    call_span: SourceSpan,
) -> Result<()> {
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "arsort" | "asort" | "krsort" | "ksort" | "rsort" | "sort"
    ) && arguments.len() > 1
    {
        if !(arguments.len() == 2
            && is_regular_sort_flag_mutation_name(name)
            && is_sort_regular_flag_expr(&arguments[1]))
        {
            let normalized = name.to_ascii_lowercase();
            return Err(Diagnostic::new(
                format!(
                    "{normalized}() currently supports default SORT_REGULAR semantics only; sort flags are unsupported"
                ),
                Some(arguments.get(1).map_or(call_span, Expr::span)),
            ));
        }
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
        if is_array_cursor_mutation_argument(&arguments[0]) {
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
    if is_single_array_path_mutation_name(name)
        && arguments.len() == 1
        && is_variable_array_access_argument(&arguments[0])
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
        IncDecTarget::Property { span, .. } => *span,
        IncDecTarget::StaticProperty { span, .. } => *span,
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
        Expr::Array { elements, span } => Ok(AssignmentTarget::List(
            list_assignment_target_from_array_elements(elements, span)?,
        )),
        Expr::Call {
            name,
            arguments,
            argument_names,
            span,
        } if name.eq_ignore_ascii_case("list") => {
            reject_named_language_construct_arguments(&argument_names, span)?;
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
        | AssignmentTarget::Property { span, .. }
        | AssignmentTarget::StaticProperty { span, .. } => *span,
        AssignmentTarget::ArrayDim(target) => target.span,
        AssignmentTarget::List(target) => target.span,
    }
}

fn validate_foreach_by_reference_target(target: &AssignmentTarget, span: SourceSpan) -> Result<()> {
    match target {
        AssignmentTarget::Variable { .. } | AssignmentTarget::ArrayDim(_) => Ok(()),
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
        AssignmentTarget::Property { .. } => Ok(()),
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
        AssignmentTarget::ArrayDim(_) | AssignmentTarget::DynamicArrayDim { .. } => Ok(()),
        AssignmentTarget::DynamicVariable { .. }
        | AssignmentTarget::Property { .. }
        | AssignmentTarget::StaticProperty { .. }
        | AssignmentTarget::List(_) => Err(Diagnostic::new(
            "compound assignment expressions currently support variables and array/string offsets",
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
        Expr::StaticPropertyFetch { .. } | Expr::ClassConstantFetch { .. } => {}
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
        | Expr::Print {
            expression: target, ..
        }
        | Expr::DynamicVariable { name: target, .. }
        | Expr::Include { path: target, .. }
        | Expr::Unary { expr: target, .. }
        | Expr::Cast { expr: target, .. }
        | Expr::Grouped { expr: target, .. } => reject_append_array_read(target)?,
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
        Expr::Ternary { .. } => false,
        Expr::InterpolatedString(_, _)
        | Expr::Variable(_, _)
        | Expr::DynamicVariable { .. }
        | Expr::IncDec { .. }
        | Expr::Assign { .. }
        | Expr::AssignRef { .. }
        | Expr::Print { .. }
        | Expr::Include { .. }
        | Expr::AnonymousFunction(_)
        | Expr::Call { .. }
        | Expr::DynamicCall { .. }
        | Expr::MethodCall { .. }
        | Expr::NewObject { .. }
        | Expr::PropertyFetch { .. }
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. }
        | Expr::ArrayAccess { .. }
        | Expr::Isset { .. }
        | Expr::Empty { .. } => false,
    }
}

fn is_supported_parameter_default_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_, _)
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_) => true,
        Expr::Array { elements, .. } => elements.iter().all(|element| {
            element
                .key
                .as_ref()
                .is_none_or(is_supported_parameter_default_expr)
                && match &element.value {
                    ArrayElementValue::Value(value) => is_supported_parameter_default_expr(value),
                    ArrayElementValue::Reference(_) => false,
                }
        }),
        Expr::Unary { expr, .. } | Expr::Grouped { expr, .. } => {
            is_supported_parameter_default_expr(expr)
        }
        _ => false,
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
