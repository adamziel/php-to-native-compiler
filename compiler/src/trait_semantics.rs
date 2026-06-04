use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ast::{
    ClassConstantDecl, ClassDecl, ClassMember, ClassMethodDecl, ClassPropertyDecl, ClassVisibility,
    Expr, Span, TraitDecl, TraitMethodAliasDecl, TraitMethodVisibilityDecl, TraitUseDecl,
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

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveTraitProperty {
    pub declaring_trait_name: String,
    pub property: ClassPropertyDecl,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveTraitConstant {
    pub declaring_trait_name: String,
    pub constant: ClassConstantDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComposedTraitMethodName {
    declaring_trait_name: String,
    source_method_key: String,
}

#[derive(Debug, Clone, Default)]
struct ResolvedTraitMethodAdaptations {
    aliases_by_trait_key: HashMap<String, Vec<TraitMethodAliasDecl>>,
    visibility_by_trait_key: HashMap<String, Vec<TraitMethodVisibilityDecl>>,
}

impl ResolvedTraitMethodAdaptations {
    fn aliases_for(&self, trait_key: &str) -> &[TraitMethodAliasDecl] {
        self.aliases_by_trait_key
            .get(trait_key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn visibility_adaptations_for(&self, trait_key: &str) -> &[TraitMethodVisibilityDecl] {
        self.visibility_by_trait_key
            .get(trait_key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
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

pub fn compose_class_abstract_trait_method_requirements(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    compose_abstract_trait_method_requirements_from_uses(
        &class.trait_uses,
        trait_lookup,
        &mut HashSet::new(),
    )
}

pub fn compose_effective_trait_methods_for_trait(
    trait_decl: &TraitDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    compose_trait_methods_for_trait(trait_decl, trait_lookup, &mut HashSet::new())
}

pub fn compose_class_effective_trait_properties(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitProperty>> {
    let direct_properties = declared_class_properties(class);
    let mut properties = Vec::new();
    let mut composed: HashMap<String, EffectiveTraitProperty> = HashMap::new();
    for trait_use in &class.trait_uses {
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        for property in
            compose_trait_properties_for_trait(trait_decl, trait_lookup, &mut HashSet::new())?
        {
            let key = property.property.name.clone();
            if let Some(class_property) = direct_properties.get(&key) {
                if trait_properties_are_compatible(&property.property, class_property) {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    property.property.span,
                    format!(
                        "unsupported trait use: class {} and trait define incompatible property ${}",
                        class.name, property.property.name
                    ),
                ));
            }
            if let Some(existing) = composed.get(&key) {
                if trait_properties_are_compatible(&existing.property, &property.property) {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    property.property.span,
                    format!(
                        "unsupported trait use: trait property {}::${} conflicts with {}::${}; incompatible trait property definitions are not implemented",
                        property.declaring_trait_name,
                        property.property.name,
                        existing.declaring_trait_name,
                        property.property.name
                    ),
                ));
            }
            composed.insert(key, property.clone());
            properties.push(property);
        }
    }
    Ok(properties)
}

pub fn compose_class_effective_trait_constants(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> TraitSemanticResult<Vec<EffectiveTraitConstant>> {
    let direct_constants = declared_class_constants(class);
    let mut constants = Vec::new();
    let mut composed: HashMap<String, EffectiveTraitConstant> = HashMap::new();
    for trait_use in &class.trait_uses {
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        for constant in
            compose_trait_constants_for_trait(trait_decl, trait_lookup, &mut HashSet::new())?
        {
            let key = constant.constant.name.clone();
            if let Some(class_constant) = direct_constants.get(&key) {
                if trait_constants_are_compatible(&constant.constant, class_constant) {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    constant.constant.span,
                    format!(
                        "unsupported trait use: class {} and trait define incompatible constant {}",
                        class.name, constant.constant.name
                    ),
                ));
            }
            if let Some(existing) = composed.get(&key) {
                if trait_constants_are_compatible(&existing.constant, &constant.constant) {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    constant.constant.span,
                    format!(
                        "unsupported trait use: trait constant {}::{} conflicts with {}::{}; incompatible trait constant definitions are not implemented",
                        constant.declaring_trait_name,
                        constant.constant.name,
                        existing.declaring_trait_name,
                        constant.constant.name
                    ),
                ));
            }
            composed.insert(key, constant.clone());
            constants.push(constant);
        }
    }
    Ok(constants)
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

    let mut methods = trait_decl
        .methods
        .iter()
        .cloned()
        .map(|method| EffectiveTraitMethod {
            declaring_trait_name: trait_decl.name.clone(),
            method,
        })
        .collect::<Vec<_>>();
    methods.extend(compose_effective_trait_methods_from_uses(
        &trait_decl.trait_uses,
        trait_lookup,
        path,
        &declared_trait_method_names(trait_decl),
    )?);
    methods.extend(compose_abstract_trait_method_requirements_from_uses(
        &trait_decl.trait_uses,
        trait_lookup,
        path,
    )?);

    path.remove(&key);
    Ok(methods)
}

fn compose_trait_properties_for_trait(
    trait_decl: &TraitDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitProperty>> {
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

    let mut properties = Vec::new();
    let mut composed: HashMap<String, EffectiveTraitProperty> = HashMap::new();
    let direct_properties = declared_trait_properties(trait_decl);
    for trait_use in &trait_decl.trait_uses {
        let nested = resolve_trait_use_decl(trait_use, trait_lookup)?;
        for property in compose_trait_properties_for_trait(nested, trait_lookup, path)? {
            if let Some(direct_property) = direct_properties.get(&property.property.name) {
                if !trait_properties_are_compatible(direct_property, &property.property) {
                    return Err(TraitSemanticError::new(
                        direct_property.span,
                        format!(
                            "unsupported trait use: trait property {}::${} conflicts with {}::${}; incompatible trait property definitions are not implemented",
                            trait_decl.name,
                            direct_property.name,
                            property.declaring_trait_name,
                            property.property.name
                        ),
                    ));
                }
                continue;
            }
            insert_effective_trait_property(&mut properties, &mut composed, property)?;
        }
    }
    for property in &trait_decl.properties {
        insert_effective_trait_property(
            &mut properties,
            &mut composed,
            EffectiveTraitProperty {
                declaring_trait_name: trait_decl.name.clone(),
                property: property.clone(),
            },
        )?;
    }

    path.remove(&key);
    Ok(properties)
}

fn insert_effective_trait_property(
    properties: &mut Vec<EffectiveTraitProperty>,
    composed: &mut HashMap<String, EffectiveTraitProperty>,
    property: EffectiveTraitProperty,
) -> TraitSemanticResult<()> {
    let key = property.property.name.clone();
    if let Some(existing) = composed.get(&key) {
        if trait_properties_are_compatible(&existing.property, &property.property) {
            return Ok(());
        }
        return Err(TraitSemanticError::new(
            property.property.span,
            format!(
                "unsupported trait use: trait property {}::${} conflicts with {}::${}; incompatible trait property definitions are not implemented",
                property.declaring_trait_name,
                property.property.name,
                existing.declaring_trait_name,
                property.property.name
            ),
        ));
    }
    composed.insert(key, property.clone());
    properties.push(property);
    Ok(())
}

fn compose_trait_constants_for_trait(
    trait_decl: &TraitDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitConstant>> {
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

    let mut constants = Vec::new();
    let mut composed: HashMap<String, EffectiveTraitConstant> = HashMap::new();
    for trait_use in &trait_decl.trait_uses {
        let nested = resolve_trait_use_decl(trait_use, trait_lookup)?;
        for constant in compose_trait_constants_for_trait(nested, trait_lookup, path)? {
            insert_effective_trait_constant(&mut constants, &mut composed, constant)?;
        }
    }
    for constant in &trait_decl.constants {
        insert_effective_trait_constant(
            &mut constants,
            &mut composed,
            EffectiveTraitConstant {
                declaring_trait_name: trait_decl.name.clone(),
                constant: constant.clone(),
            },
        )?;
    }

    path.remove(&key);
    Ok(constants)
}

fn insert_effective_trait_constant(
    constants: &mut Vec<EffectiveTraitConstant>,
    composed: &mut HashMap<String, EffectiveTraitConstant>,
    constant: EffectiveTraitConstant,
) -> TraitSemanticResult<()> {
    let key = constant.constant.name.clone();
    if let Some(existing) = composed.get(&key) {
        if trait_constants_are_compatible(&existing.constant, &constant.constant) {
            return Ok(());
        }
        return Err(TraitSemanticError::new(
            constant.constant.span,
            format!(
                "unsupported trait use: trait constant {}::{} conflicts with {}::{}; incompatible trait constant definitions are not implemented",
                constant.declaring_trait_name,
                constant.constant.name,
                existing.declaring_trait_name,
                constant.constant.name
            ),
        ));
    }
    composed.insert(key, constant.clone());
    constants.push(constant);
    Ok(())
}

fn compose_effective_trait_methods_from_uses(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
    direct_method_names: &HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    let mut methods = Vec::new();
    let mut composed_names: HashMap<String, ComposedTraitMethodName> = HashMap::new();
    let method_adaptations =
        resolve_trait_method_adaptations_for_uses(trait_uses, trait_lookup, path)?;
    let precedence_exclusions =
        trait_precedence_exclusions_for_uses(trait_uses, trait_lookup, path)?;

    for trait_use in trait_uses {
        let used_trait_key = trait_key(&trait_use.name);
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        let trait_methods = compose_trait_methods_for_trait(trait_decl, trait_lookup, path)?;
        let visibility_adaptations = trait_visibility_adaptations(
            &trait_decl.name,
            method_adaptations.visibility_adaptations_for(&used_trait_key),
            &trait_methods,
        )?;
        let mut aliases_by_method: HashMap<String, Vec<_>> = HashMap::new();
        for alias in method_adaptations.aliases_for(&used_trait_key) {
            let Some(candidate) = trait_methods.iter().find(|candidate| {
                candidate
                    .method
                    .function
                    .name
                    .eq_ignore_ascii_case(&alias.method_name)
            }) else {
                let alias_trait_name = alias.trait_name.as_deref().unwrap_or(&trait_decl.name);
                return Err(TraitSemanticError::new(
                    alias.span,
                    format!(
                        "unsupported trait use: trait alias {}::{} targets a missing method",
                        alias_trait_name, alias.method_name
                    ),
                ));
            };
            aliases_by_method
                .entry(method_key(&candidate.method.function.name))
                .or_default()
                .push(alias);
        }

        for candidate in &trait_methods {
            if candidate.method.is_abstract {
                continue;
            }
            let method_name_key = method_key(&candidate.method.function.name);
            if let Some(aliases) = aliases_by_method.remove(&method_name_key) {
                for alias in aliases {
                    let alias_key = method_key(&alias.alias);
                    if direct_method_names.contains(&alias_key) {
                        continue;
                    }
                    if alias_key == method_name_key
                        && alias.visibility != candidate.method.visibility
                    {
                        let alias_trait_name =
                            alias.trait_name.as_deref().unwrap_or(&trait_decl.name);
                        return Err(TraitSemanticError::new(
                            alias.span,
                            format!(
                                "unsupported trait use: trait alias {}::{} as {} conflicts with {}::{}",
                                alias_trait_name,
                                alias.method_name,
                                alias.alias,
                                candidate.declaring_trait_name,
                                alias.alias
                            ),
                        ));
                    }
                    if let Some(existing) = composed_names.get(&alias_key) {
                        if !composed_trait_method_name_matches_source(
                            existing,
                            candidate,
                            &method_name_key,
                        ) || alias.visibility != candidate.method.visibility
                        {
                            let alias_trait_name =
                                alias.trait_name.as_deref().unwrap_or(&trait_decl.name);
                            return Err(TraitSemanticError::new(
                                alias.span,
                                format!(
                                    "unsupported trait use: trait alias {}::{} as {} conflicts with {}::{}",
                                    alias_trait_name,
                                    alias.method_name,
                                    alias.alias,
                                    existing.declaring_trait_name,
                                    alias.alias
                                ),
                            ));
                        }
                        continue;
                    }
                    if let Some(existing) = trait_methods.iter().find(|method| {
                        method_key(&method.method.function.name) == alias_key
                            && !same_effective_trait_method_source(
                                method,
                                candidate,
                                &method_name_key,
                            )
                    }) {
                        let alias_trait_name =
                            alias.trait_name.as_deref().unwrap_or(&trait_decl.name);
                        return Err(TraitSemanticError::new(
                            alias.span,
                            format!(
                                "unsupported trait use: trait alias {}::{} as {} conflicts with {}::{}",
                                alias_trait_name,
                                alias.method_name,
                                alias.alias,
                                existing.declaring_trait_name,
                                alias.alias
                            ),
                        ));
                    }

                    let mut aliased = candidate.clone();
                    aliased.method.function.name = alias.alias.clone();
                    aliased.method.visibility = alias.visibility;
                    aliased.method.span = alias.span;
                    composed_names.insert(
                        alias_key,
                        ComposedTraitMethodName {
                            declaring_trait_name: aliased.declaring_trait_name.clone(),
                            source_method_key: method_name_key.clone(),
                        },
                    );
                    methods.push(aliased);
                }
            }
            if precedence_exclusions.contains(&(used_trait_key.clone(), method_name_key.clone())) {
                continue;
            }
            if direct_method_names.contains(&method_name_key) {
                continue;
            }
            if let Some(existing) = composed_names.get(&method_name_key) {
                if composed_trait_method_name_matches_source(existing, candidate, &method_name_key)
                {
                    continue;
                }
                return Err(TraitSemanticError::new(
                    candidate.method.span,
                    format!(
                        "unsupported trait use: trait method {}::{} conflicts with {}::{}; add an insteadof adaptation or class override",
                        candidate.declaring_trait_name,
                        candidate.method.function.name,
                        existing.declaring_trait_name,
                        candidate.method.function.name
                    ),
                ));
            }

            let mut composed = candidate.clone();
            if let Some(visibility) = visibility_adaptations.get(&method_name_key) {
                composed.method.visibility = *visibility;
            }
            composed_names.insert(
                method_name_key.clone(),
                ComposedTraitMethodName {
                    declaring_trait_name: composed.declaring_trait_name.clone(),
                    source_method_key: method_name_key,
                },
            );
            methods.push(composed);
        }
    }

    Ok(methods)
}

fn compose_abstract_trait_method_requirements_from_uses(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<Vec<EffectiveTraitMethod>> {
    let mut requirements = Vec::new();
    let method_adaptations =
        resolve_trait_method_adaptations_for_uses(trait_uses, trait_lookup, path)?;
    let precedence_exclusions =
        trait_precedence_exclusions_for_uses(trait_uses, trait_lookup, path)?;

    for trait_use in trait_uses {
        let used_trait_key = trait_key(&trait_use.name);
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        let trait_methods = compose_trait_methods_for_trait(trait_decl, trait_lookup, path)?;
        let visibility_adaptations = trait_visibility_adaptations(
            &trait_decl.name,
            method_adaptations.visibility_adaptations_for(&used_trait_key),
            &trait_methods,
        )?;
        let mut aliases_by_method: HashMap<String, Vec<_>> = HashMap::new();
        for alias in method_adaptations.aliases_for(&used_trait_key) {
            let Some(candidate) = trait_methods.iter().find(|candidate| {
                candidate
                    .method
                    .function
                    .name
                    .eq_ignore_ascii_case(&alias.method_name)
            }) else {
                let alias_trait_name = alias.trait_name.as_deref().unwrap_or(&trait_decl.name);
                return Err(TraitSemanticError::new(
                    alias.span,
                    format!(
                        "unsupported trait use: trait alias {}::{} targets a missing method",
                        alias_trait_name, alias.method_name
                    ),
                ));
            };
            aliases_by_method
                .entry(method_key(&candidate.method.function.name))
                .or_default()
                .push(alias);
        }

        for candidate in &trait_methods {
            if !candidate.method.is_abstract {
                continue;
            }
            let method_name_key = method_key(&candidate.method.function.name);
            if let Some(aliases) = aliases_by_method.remove(&method_name_key) {
                for alias in aliases {
                    let mut aliased = candidate.clone();
                    aliased.method.function.name = alias.alias.clone();
                    aliased.method.visibility = alias.visibility;
                    aliased.method.span = alias.span;
                    requirements.push(aliased);
                }
            }
            if precedence_exclusions.contains(&(used_trait_key.clone(), method_name_key.clone())) {
                continue;
            }
            let mut requirement = candidate.clone();
            if let Some(visibility) = visibility_adaptations.get(&method_name_key) {
                requirement.method.visibility = *visibility;
            }
            requirements.push(requirement);
        }
    }

    Ok(requirements)
}

fn resolve_trait_method_adaptations_for_uses(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
) -> TraitSemanticResult<ResolvedTraitMethodAdaptations> {
    let mut resolved = ResolvedTraitMethodAdaptations::default();

    for trait_use in trait_uses {
        for alias in &trait_use.aliases {
            let target_key = resolve_trait_method_adaptation_target_key(
                trait_uses,
                trait_lookup,
                path,
                alias.trait_name.as_deref(),
                &alias.method_name,
                alias.span,
                "alias",
            )?;
            resolved
                .aliases_by_trait_key
                .entry(target_key)
                .or_default()
                .push(alias.clone());
        }

        for adaptation in &trait_use.visibility_adaptations {
            let target_key = resolve_trait_method_adaptation_target_key(
                trait_uses,
                trait_lookup,
                path,
                adaptation.trait_name.as_deref(),
                &adaptation.method_name,
                adaptation.span,
                "visibility adaptation",
            )?;
            resolved
                .visibility_by_trait_key
                .entry(target_key)
                .or_default()
                .push(adaptation.clone());
        }
    }

    Ok(resolved)
}

fn resolve_trait_method_adaptation_target_key(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
    explicit_trait_name: Option<&str>,
    method_name: &str,
    span: Span,
    kind: &str,
) -> TraitSemanticResult<String> {
    if let Some(trait_name) = explicit_trait_name {
        return Ok(trait_key(trait_name));
    }

    if trait_uses.len() == 1 {
        return Ok(trait_key(&trait_uses[0].name));
    }

    resolve_unqualified_trait_method_adaptation_target_key(
        trait_uses,
        trait_lookup,
        path,
        method_name,
        span,
        kind,
    )
}

fn resolve_unqualified_trait_method_adaptation_target_key(
    trait_uses: &[TraitUseDecl],
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    path: &mut HashSet<String>,
    method_name: &str,
    span: Span,
    kind: &str,
) -> TraitSemanticResult<String> {
    let mut matches = Vec::new();
    for trait_use in trait_uses {
        let trait_decl = resolve_trait_use_decl(trait_use, trait_lookup)?;
        let trait_methods = compose_trait_methods_for_trait(trait_decl, trait_lookup, path)?;
        if trait_methods.iter().any(|method| {
            method
                .method
                .function
                .name
                .eq_ignore_ascii_case(method_name)
        }) {
            matches.push((trait_key(&trait_use.name), trait_decl.name.clone()));
        }
    }

    match matches.as_slice() {
        [] => Err(TraitSemanticError::new(
            span,
            format!(
                "unsupported trait use: unqualified trait {kind} {method_name} targets a missing method"
            ),
        )),
        [(target_key, _)] => Ok(target_key.clone()),
        _ => {
            let trait_names = matches
                .iter()
                .map(|(_, name)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(TraitSemanticError::new(
                span,
                format!(
                    "unsupported trait use: unqualified trait {kind} {method_name} is ambiguous across {trait_names}; qualify the trait method"
                ),
            ))
        }
    }
}

fn same_effective_trait_method_source(
    left: &EffectiveTraitMethod,
    right: &EffectiveTraitMethod,
    right_method_key: &str,
) -> bool {
    left.declaring_trait_name
        .eq_ignore_ascii_case(&right.declaring_trait_name)
        && method_key(&left.method.function.name) == right_method_key
}

fn composed_trait_method_name_matches_source(
    existing: &ComposedTraitMethodName,
    candidate: &EffectiveTraitMethod,
    candidate_method_key: &str,
) -> bool {
    existing
        .declaring_trait_name
        .eq_ignore_ascii_case(&candidate.declaring_trait_name)
        && existing.source_method_key == candidate_method_key
}

fn trait_visibility_adaptations(
    trait_name: &str,
    visibility_adaptations: &[TraitMethodVisibilityDecl],
    trait_methods: &[EffectiveTraitMethod],
) -> TraitSemanticResult<HashMap<String, ClassVisibility>> {
    let mut adaptations = HashMap::new();
    for adaptation in visibility_adaptations {
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
                    adaptation.trait_name.as_deref().unwrap_or(trait_name),
                    adaptation.method_name
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

fn declared_class_properties(class: &ClassDecl) -> HashMap<String, ClassPropertyDecl> {
    class
        .members
        .iter()
        .filter_map(|member| {
            let ClassMember::Property(property) = member else {
                return None;
            };
            Some((property.name.clone(), property.clone()))
        })
        .collect()
}

fn declared_class_constants(class: &ClassDecl) -> HashMap<String, ClassConstantDecl> {
    class
        .members
        .iter()
        .filter_map(|member| {
            let ClassMember::Constant(constant) = member else {
                return None;
            };
            Some((constant.name.clone(), constant.clone()))
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

fn declared_trait_properties(trait_decl: &TraitDecl) -> HashMap<String, ClassPropertyDecl> {
    trait_decl
        .properties
        .iter()
        .map(|property| (property.name.clone(), property.clone()))
        .collect()
}

pub fn trait_properties_are_compatible(
    left: &ClassPropertyDecl,
    right: &ClassPropertyDecl,
) -> bool {
    left.visibility == right.visibility
        && left.is_static == right.is_static
        && left.type_decl.as_ref().map(|decl| decl.text.as_str())
            == right.type_decl.as_ref().map(|decl| decl.text.as_str())
        && optional_default_exprs_are_compatible(left.default.as_ref(), right.default.as_ref())
}

pub fn trait_constants_are_compatible(left: &ClassConstantDecl, right: &ClassConstantDecl) -> bool {
    left.visibility == right.visibility && default_exprs_are_compatible(&left.value, &right.value)
}

fn optional_default_exprs_are_compatible(left: Option<&Expr>, right: Option<&Expr>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => default_exprs_are_compatible(left, right),
        _ => false,
    }
}

fn default_exprs_are_compatible(left: &Expr, right: &Expr) -> bool {
    match (left, right) {
        (Expr::Null(_), Expr::Null(_)) => true,
        (Expr::Bool(left, _), Expr::Bool(right, _)) => left == right,
        (Expr::Int(left, _), Expr::Int(right, _)) => left == right,
        (Expr::Float(left, _), Expr::Float(right, _)) => left == right,
        (Expr::String(left, _), Expr::String(right, _)) => left == right,
        (
            Expr::Array {
                items: left_items, ..
            },
            Expr::Array {
                items: right_items, ..
            },
        ) => {
            left_items.len() == right_items.len()
                && left_items.iter().zip(right_items).all(|(left, right)| {
                    left.by_reference == right.by_reference
                        && optional_default_exprs_are_compatible(
                            left.key.as_ref(),
                            right.key.as_ref(),
                        )
                        && default_exprs_are_compatible(&left.value, &right.value)
                })
        }
        (
            Expr::Unary {
                op: left_op,
                expr: left_expr,
                ..
            },
            Expr::Unary {
                op: right_op,
                expr: right_expr,
                ..
            },
        ) => left_op == right_op && default_exprs_are_compatible(left_expr, right_expr),
        _ => false,
    }
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

    fn effective_property_names(source: &str) -> Vec<(String, bool, String)> {
        let (traits, class) = parsed_traits_and_class(source);
        compose_class_effective_trait_properties(&class, &traits)
            .unwrap()
            .into_iter()
            .map(|property| {
                (
                    property.property.name,
                    property.property.is_static,
                    property.declaring_trait_name,
                )
            })
            .collect()
    }

    fn effective_constant_names(source: &str) -> Vec<(String, String)> {
        let (traits, class) = parsed_traits_and_class(source);
        compose_class_effective_trait_constants(&class, &traits)
            .unwrap()
            .into_iter()
            .map(|constant| (constant.constant.name, constant.declaring_trait_name))
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
                    "renamedRight".to_string(),
                    ClassVisibility::Private,
                    "Right".to_string()
                ),
                (
                    "rightOnly".to_string(),
                    ClassVisibility::Public,
                    "Right".to_string()
                ),
            ]
        );
    }

    #[test]
    fn trait_semantics_resolves_unqualified_multi_trait_adaptations_when_unique() {
        let methods = effective_method_names(
            r#"<?php
trait HookTools {
    public function boot() {}
}
trait LabelTools {
    public function label() {}
}
class Plugin {
    use HookTools, LabelTools {
        boot as protected;
        label as private hiddenLabel;
    }
}
"#,
        );

        assert_eq!(
            methods,
            vec![
                (
                    "boot".to_string(),
                    ClassVisibility::Protected,
                    "HookTools".to_string()
                ),
                (
                    "hiddenLabel".to_string(),
                    ClassVisibility::Private,
                    "LabelTools".to_string()
                ),
                (
                    "label".to_string(),
                    ClassVisibility::Public,
                    "LabelTools".to_string()
                ),
            ]
        );
    }

    #[test]
    fn trait_semantics_reports_ambiguous_unqualified_multi_trait_adaptations() {
        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait FirstLabel { public function label() {} }
trait SecondLabel { public function label() {} }
class Plugin {
    use FirstLabel, SecondLabel {
        label as labelAlias;
        FirstLabel::label insteadof SecondLabel;
    }
}
"#,
        );
        let alias_error = compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(alias_error
            .message
            .contains("unqualified trait alias label is ambiguous"));

        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait FirstLabel { public function label() {} }
trait SecondLabel { public function label() {} }
class Plugin {
    use FirstLabel, SecondLabel {
        label as protected;
        FirstLabel::label insteadof SecondLabel;
    }
}
"#,
        );
        let visibility_error = compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(visibility_error
            .message
            .contains("unqualified trait visibility adaptation label is ambiguous"));
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

    #[test]
    fn trait_alias_existing_method_collision_reports_later_trait_methods() {
        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait CollidingAlias {
    public function aliasSource() {}
    public function existing() {}
}
class UsesCollidingAlias {
    use CollidingAlias {
        CollidingAlias::aliasSource as existing;
    }
}
"#,
        );
        let conflict = compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(conflict.message.contains(
            "trait alias CollidingAlias::aliasSource as existing conflicts with CollidingAlias::existing"
        ));

        let methods = effective_method_names(
            r#"<?php
trait SameNameAlias {
    public function existing() {}
}
class UsesSameNameAlias {
    use SameNameAlias {
        SameNameAlias::existing as existing;
    }
}
"#,
        );
        assert_eq!(
            methods,
            vec![(
                "existing".to_string(),
                ClassVisibility::Public,
                "SameNameAlias".to_string()
            )]
        );

        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait SameNameVisibilityAlias {
    public function existing() {}
}
class UsesSameNameVisibilityAlias {
    use SameNameVisibilityAlias {
        SameNameVisibilityAlias::existing as private existing;
    }
}
"#,
        );
        let visibility_conflict =
            compose_class_effective_trait_methods(&class, &traits).unwrap_err();
        assert!(visibility_conflict.message.contains(
            "trait alias SameNameVisibilityAlias::existing as existing conflicts with SameNameVisibilityAlias::existing"
        ));
    }

    #[test]
    fn trait_semantics_composes_properties_and_constants() {
        let properties = effective_property_names(
            r#"<?php
trait Nested {
    public $nested = "n";
    public static $shared = 1;
}
trait Direct {
    use Nested;
    public $own = "d";
    public static $shared = 1;
}
trait Same {
    public $own = "d";
}
class UsesMembers {
    use Direct, Same;
    public $nested = "n";
}
"#,
        );
        assert_eq!(
            properties,
            vec![
                ("own".to_string(), false, "Direct".to_string()),
                ("shared".to_string(), true, "Direct".to_string()),
            ]
        );

        let constants = effective_constant_names(
            r#"<?php
trait NestedConst { public const NESTED = "n"; }
trait DirectConst {
    use NestedConst;
    public const OWN = "d";
}
trait SameConst { public const OWN = "d"; }
class UsesConstants {
    use DirectConst, SameConst;
    public const NESTED = "n";
}
"#,
        );
        assert_eq!(
            constants,
            vec![("OWN".to_string(), "DirectConst".to_string())]
        );
    }

    #[test]
    fn trait_semantics_reports_property_and_constant_conflicts() {
        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait FirstProperty { public $same = "a"; }
trait SecondProperty { public $same = "b"; }
class UsesProperties { use FirstProperty, SecondProperty; }
"#,
        );
        let property_error = compose_class_effective_trait_properties(&class, &traits).unwrap_err();
        assert!(property_error.message.contains("trait property"));

        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait NestedProperty { public $same = "a"; }
trait DirectProperty { use NestedProperty; public $same = "b"; }
class UsesNestedProperty { use DirectProperty; }
"#,
        );
        let nested_property_error =
            compose_class_effective_trait_properties(&class, &traits).unwrap_err();
        assert!(nested_property_error.message.contains("trait property"));

        let (traits, class) = parsed_traits_and_class(
            r#"<?php
trait FirstConstant { public const SAME = "a"; }
trait SecondConstant { public const SAME = "b"; }
class UsesConstants { use FirstConstant, SecondConstant; }
"#,
        );
        let constant_error = compose_class_effective_trait_constants(&class, &traits).unwrap_err();
        assert!(constant_error.message.contains("trait constant"));
    }
}
