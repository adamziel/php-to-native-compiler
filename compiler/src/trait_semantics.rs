use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{
    ClassDecl, ClassMember, ClassMethodDecl, ClassVisibility, Span, TraitDecl, TraitUseDecl,
};
use crate::error::{Diagnostic, Phase};

pub type TraitSemanticResult<T> = Result<T, TraitSemanticError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitSemanticError {
    pub span: Span,
    pub message: String,
}

impl TraitSemanticError {
    fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    pub fn to_diagnostic(&self, phase: Phase) -> Diagnostic {
        Diagnostic::new(
            phase,
            self.span.line,
            self.span.column,
            self.message.clone(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveTraitMethod {
    pub declaring_trait_name: String,
    pub method: ClassMethodDecl,
}

pub fn trait_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

pub fn method_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

pub fn compose_class_effective_trait_methods(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    compose_effective_trait_methods_from_uses(
        &class.trait_uses,
        trait_lookup,
        &mut HashSet::new(),
        &declared_class_method_names(class),
    )
}

pub fn compose_effective_trait_methods_for_trait(
    trait_decl: &TraitDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    compose_trait_methods_for_trait(trait_decl, trait_lookup, &mut HashSet::new())
}

fn compose_trait_methods_for_trait(
    trait_decl: &TraitDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    let key = trait_key(&trait_decl.name);
    if !path.insert(key.clone()) {
        return Err(TraitSemanticError::new(
            trait_decl.span,
            format!(
                "unsupported trait use: recursive trait-body use involving {} is not implemented",
                trait_decl.name
            ),
        ));
    }

    let mut methods = compose_effective_trait_methods_from_uses(
        &trait_decl.trait_uses,
        trait_lookup,
        path,
        &declared_trait_method_names(trait_decl),
    )?;
    methods.extend(
        trait_decl
            .methods
            .iter()
            .cloned()
            .map(|method| EffectiveTraitMethod {
                declaring_trait_name: trait_decl.name.clone(),
                method,
            }),
    );

    path.remove(&key);
    Ok(methods)
}

fn compose_effective_trait_methods_from_uses(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
    direct_method_names: &HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    let mut methods = Vec::new();
    let mut composed_names: HashMap<String, String> = HashMap::new();
    let precedence_exclusions =
        trait_precedence_exclusions_for_uses(trait_uses, trait_lookup, path)?;

    for trait_use in trait_uses {
        let used_trait_key = trait_key(&trait_use.name);
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        let trait_methods = compose_trait_methods_for_trait(trait_decl, trait_lookup, path)?;
        let visibility_adaptations = trait_visibility_adaptations(trait_use, &trait_methods)?;

        for candidate in &trait_methods {
            let method_name_key = method_key(&candidate.method.function.name);
            if precedence_exclusions.contains(&(used_trait_key.clone(), method_name_key.clone())) {
                continue;
            }
            if direct_method_names.contains(&method_name_key) {
                continue;
            }
            if let Some(existing_trait) = composed_names.get(&method_name_key) {
                if existing_trait.eq_ignore_ascii_case(&candidate.declaring_trait_name) {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    candidate.method.span,
                    format!(
                        "unsupported trait use: trait method {}::{} conflicts with {}::{}; add an insteadof adaptation or class override",
                        candidate.declaring_trait_name,
                        candidate.method.function.name,
                        existing_trait,
                        candidate.method.function.name
                    ),
                ));
            }

            let mut composed = candidate.clone();
            if let Some(visibility) = visibility_adaptations.get(&method_name_key) {
                composed.method.visibility = *visibility;
            }
            composed_names.insert(method_name_key, composed.declaring_trait_name.clone());
            methods.push(composed);
        }

        for alias in &trait_use.aliases {
            let Some(candidate) = trait_methods.iter().find(|candidate| {
                candidate
                    .method
                    .function
                    .name
                    .eq_ignore_ascii_case(&alias.method_name)
            }) else {
                return Err(TraitSemanticError::new(
                    alias.span,
                    format!(
                        "unsupported trait use: trait alias {}::{} targets a missing method",
                        trait_decl.name, alias.method_name
                    ),
                ));
            };

            let alias_key = method_key(&alias.alias);
            if direct_method_names.contains(&alias_key) {
                continue;
            }
            if let Some(existing_trait) = composed_names.get(&alias_key) {
                return Err(TraitSemanticError::new(
                    alias.span,
                    format!(
                        "unsupported trait use: trait alias {}::{} as {} conflicts with {}::{}",
                        trait_decl.name,
                        alias.method_name,
                        alias.alias,
                        existing_trait,
                        alias.alias
                    ),
                ));
            }

            let mut aliased = candidate.clone();
            aliased.method.function.name = alias.alias.clone();
            aliased.method.visibility = alias.visibility;
            aliased.method.span = alias.span;
            composed_names.insert(alias_key, aliased.declaring_trait_name.clone());
            methods.push(aliased);
        }
    }

    Ok(methods)
}

fn trait_visibility_adaptations(
    trait_use: &TraitUseDecl,
    trait_methods: &[EffectiveTraitMethod],
) -> TraitSemanticResult<HashMap<String, ClassVisibility>> {
    let mut adaptations = HashMap::new();
    for adaptation in &trait_use.visibility_adaptations {
        let Some(method) = trait_methods.iter().find(|method| {
            method
                .method
                .function
                .name
                .eq_ignore_ascii_case(&adaptation.method_name)
        }) else {
            return Err(TraitSemanticError::new(
                adaptation.span,
                format!(
                    "unsupported trait use: trait visibility adaptation {}::{} targets a missing method",
                    trait_use.name, adaptation.method_name
                ),
            ));
        };
        adaptations.insert(
            method_key(&method.method.function.name),
            adaptation.visibility,
        );
    }
    Ok(adaptations)
}

fn resolve_trait_use_decl<'a>(
    trait_use: &TraitUseDecl,
    trait_lookup: &'a HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<&'a TraitDecl> {
    let key = trait_key(&trait_use.name);
    trait_lookup.get(&key).map(Rc::as_ref).ok_or_else(|| {
        TraitSemanticError::new(
            trait_use.span,
            format!("undefined class '{}'", trait_use.name),
        )
    })
}

fn declared_class_method_names(class: &ClassDecl) -> HashSet<String> {
    class
        .members
        .iter()
        .filter_map(|member| {
            let ClassMember::Method(method) = member else {
                return None;
            };
            Some(method_key(&method.function.name))
        })
        .collect()
}

fn declared_trait_method_names(trait_decl: &TraitDecl) -> HashSet<String> {
    trait_decl
        .methods
        .iter()
        .map(|method| method_key(&method.function.name))
        .collect()
}

fn trait_precedence_exclusions_for_uses(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<HashSet<(String, String)>> {
    let mut exclusions = HashSet::new();
    for trait_use in trait_uses {
        if trait_use.precedences.is_empty() {
            continue;
        }
        let winner_trait = resolve_trait_use_decl(trait_use, trait_lookup)?;
        let winner_methods = compose_trait_methods_for_trait(winner_trait, trait_lookup, path)?;
        for precedence in &trait_use.precedences {
            let Some(winner_method) = winner_methods.iter().find(|method| {
                method
                    .method
                    .function
                    .name
                    .eq_ignore_ascii_case(&precedence.method_name)
            }) else {
                return Err(TraitSemanticError::new(
                    precedence.span,
                    format!(
                        "unsupported trait use: trait precedence {}::{} targets a missing winning method",
                        winner_trait.name, precedence.method_name
                    ),
                ));
            };

            let loser_trait_use = TraitUseDecl {
                name: precedence.loser_trait_name.clone(),
                aliases: Vec::new(),
                visibility_adaptations: Vec::new(),
                precedences: Vec::new(),
                span: precedence.span,
            };
            let loser_trait = resolve_trait_use_decl(&loser_trait_use, trait_lookup)?;
            let loser_methods = compose_trait_methods_for_trait(loser_trait, trait_lookup, path)?;
            if !loser_methods.iter().any(|method| {
                method
                    .method
                    .function
                    .name
                    .eq_ignore_ascii_case(&winner_method.method.function.name)
            }) {
                return Err(TraitSemanticError::new(
                    precedence.span,
                    format!(
                        "unsupported trait use: trait precedence {}::{} excludes missing loser method {}::{}",
                        winner_trait.name,
                        precedence.method_name,
                        loser_trait.name,
                        winner_method.method.function.name
                    ),
                ));
            }
            exclusions.insert((
                trait_key(&precedence.loser_trait_name),
                method_key(&winner_method.method.function.name),
            ));
        }
    }
    Ok(exclusions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Stmt;
    use crate::parser::parse_source;

    fn parsed_traits_and_class(source: &str) -> (HashMap<String, Rc<TraitDecl>>, ClassDecl) {
        let program = parse_source(source).unwrap();
        let mut traits = HashMap::new();
        let mut class = None;
        for statement in program.statements {
            match statement {
                Stmt::Trait(trait_decl) => {
                    traits.insert(trait_key(&trait_decl.name), Rc::new(trait_decl));
                }
                Stmt::Class(class_decl) => {
                    class = Some(class_decl);
                }
                _ => {}
            }
        }
        (traits, class.expect("test source includes a class"))
    }

    fn effective_method_names(source: &str) -> Vec<(String, ClassVisibility, String)> {
        let (traits, class) = parsed_traits_and_class(source);
        compose_class_effective_trait_methods(&class, &traits)
            .unwrap()
            .into_iter()
            .map(|method| {
                (
                    method.method.function.name,
                    method.method.visibility,
                    method.declaring_trait_name,
                )
            })
            .collect()
    }

    #[test]
    fn trait_semantics_composes_nested_alias_visibility_and_precedence_methods() {
        let methods = effective_method_names(
            r#"<?php
trait Nested {
    public function keep() {}
    public function collide() {}
}
trait Left {
    use Nested;
    public function leftOnly() {}
}
trait Right {
    public function collide() {}
    public function rightOnly() {}
}
class UsesTraits {
    use Left, Right {
        Left::collide insteadof Right;
        Left::keep as protected kept;
        Right::rightOnly as private renamedRight;
    }
}
"#,
        );

        assert_eq!(
            methods,
            vec![
                (
                    "keep".to_string(),
                    ClassVisibility::Public,
                    "Nested".to_string()
                ),
                (
                    "collide".to_string(),
                    ClassVisibility::Public,
                    "Nested".to_string()
                ),
                (
                    "leftOnly".to_string(),
                    ClassVisibility::Public,
                    "Left".to_string()
                ),
                (
                    "kept".to_string(),
                    ClassVisibility::Protected,
                    "Nested".to_string()
                ),
                (
                    "rightOnly".to_string(),
                    ClassVisibility::Public,
                    "Right".to_string()
                ),
                (
                    "renamedRight".to_string(),
                    ClassVisibility::Private,
                    "Right".to_string()
                ),
            ]
        );
    }

    #[test]
    fn trait_semantics_uses_case_insensitive_keys_and_class_overrides() {
        let methods = effective_method_names(
            r#"<?php
trait DestructorTrait {
    public function __DESTRUCT() {}
    public function Visible() {}
}
class OverridesTrait {
    use DestructorTrait;
    public function __destruct() {}
}
"#,
        );

        assert_eq!(
            methods,
            vec![(
                "Visible".to_string(),
                ClassVisibility::Public,
                "DestructorTrait".to_string()
            )]
        );
    }

    #[test]
    fn trait_semantics_reports_conflicts_and_recursive_trait_uses() {
        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait A { public function same() {} }
trait B { public function SAME() {} }
class Conflict { use A, B; }
"#,
        );
        let conflict = compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(conflict.message.contains("conflicts"));

        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait LoopA { use LoopB; public function a() {} }
trait LoopB { use LoopA; public function b() {} }
class Recursive { use LoopA; }
"#,
        );
        let recursive = compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(recursive.message.contains("recursive trait-body use"));
    }
}
