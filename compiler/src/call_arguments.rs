use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentParameter {
    pub name: String,
    pub required: bool,
    pub by_reference: bool,
    pub variadic: bool,
}

impl CallArgumentParameter {
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: true,
            by_reference: false,
            variadic: false,
        }
    }

    pub fn optional(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: false,
            by_reference: false,
            variadic: false,
        }
    }

    pub fn with_by_reference(mut self) -> Self {
        self.by_reference = true;
        self
    }

    pub fn with_variadic(mut self) -> Self {
        self.variadic = true;
        self.required = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentSignature {
    params: Vec<CallArgumentParameter>,
    fixed_name_to_index: HashMap<String, usize>,
    variadic_index: Option<usize>,
}

impl CallArgumentSignature {
    pub fn new(params: Vec<CallArgumentParameter>) -> CallArgumentNormalizationResult<Self> {
        let mut fixed_name_to_index = HashMap::new();
        let mut variadic_index = None;

        for (index, param) in params.iter().enumerate() {
            if let Some(first_index) = fixed_name_to_index.get(&param.name).copied() {
                return Err(CallArgumentNormalizationError::DuplicateParameterName {
                    name: param.name.clone(),
                    first_parameter_index: first_index,
                    duplicate_parameter_index: index,
                });
            }

            if param.variadic {
                if let Some(first_index) = variadic_index {
                    return Err(CallArgumentNormalizationError::MultipleVariadicParameters {
                        first_parameter_index: first_index,
                        duplicate_parameter_index: index,
                    });
                }
                if index + 1 != params.len() {
                    return Err(CallArgumentNormalizationError::VariadicParameterNotFinal {
                        parameter_index: index,
                    });
                }
                variadic_index = Some(index);
            } else {
                fixed_name_to_index.insert(param.name.clone(), index);
            }
        }

        Ok(Self {
            params,
            fixed_name_to_index,
            variadic_index,
        })
    }

    pub fn params(&self) -> &[CallArgumentParameter] {
        &self.params
    }

    pub fn fixed_param_count(&self) -> usize {
        self.variadic_index.unwrap_or(self.params.len())
    }

    pub fn variadic_param(&self) -> Option<(usize, &CallArgumentParameter)> {
        self.variadic_index
            .map(|index| (index, &self.params[index]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgument {
    Positional,
    Named(String),
    Spread,
}

impl CallArgument {
    pub fn positional() -> Self {
        Self::Positional
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn spread() -> Self {
        Self::Spread
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallArgumentPassingMode {
    Value,
    Reference,
}

impl CallArgumentPassingMode {
    fn for_param(param: &CallArgumentParameter) -> Self {
        if param.by_reference {
            Self::Reference
        } else {
            Self::Value
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCallArguments {
    pub source_evaluations: Vec<CallArgumentSourceEvaluation>,
    pub fixed_slots: Vec<CallArgumentFixedSlot>,
    pub variadic_slot: Option<CallArgumentVariadicSlot>,
    pub cleanup: CallArgumentCleanupPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentSourceEvaluation {
    pub source_index: usize,
    pub kind: CallArgumentSourceEvaluationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgumentSourceEvaluationKind {
    Positional,
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentFixedSlot {
    pub parameter_index: usize,
    pub parameter_name: String,
    pub source: CallArgumentSlotSource,
    pub passing_mode: CallArgumentPassingMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgumentSlotSource {
    Supplied { source_index: usize },
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentVariadicSlot {
    pub parameter_index: usize,
    pub parameter_name: String,
    pub entries: Vec<CallArgumentVariadicEntry>,
    pub entry_passing_mode: CallArgumentPassingMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentVariadicEntry {
    pub source_index: usize,
    pub key: CallArgumentVariadicKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgumentVariadicKey {
    NextInteger,
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArgumentCleanupPlan {
    pub source_indices_reverse: Vec<usize>,
}

pub type CallArgumentNormalizationResult<T> = Result<T, CallArgumentNormalizationError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArgumentNormalizationError {
    DuplicateParameterName {
        name: String,
        first_parameter_index: usize,
        duplicate_parameter_index: usize,
    },
    MultipleVariadicParameters {
        first_parameter_index: usize,
        duplicate_parameter_index: usize,
    },
    VariadicParameterNotFinal {
        parameter_index: usize,
    },
    PositionalArgumentAfterNamedArgument {
        source_index: usize,
    },
    UnsupportedSpreadArgument {
        source_index: usize,
    },
    TooManyPositionalArguments {
        source_index: usize,
        max_positional_count: usize,
    },
    DuplicateArgument {
        parameter_name: String,
        first_source_index: usize,
        duplicate_source_index: usize,
    },
    UnknownNamedArgument {
        name: String,
        source_index: usize,
    },
    MissingRequiredArgument {
        parameter_name: String,
        parameter_index: usize,
    },
}

impl fmt::Display for CallArgumentNormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateParameterName {
                name,
                first_parameter_index,
                duplicate_parameter_index,
            } => write!(
                f,
                "duplicate call parameter ${name} at parameter {duplicate_parameter_index}; first declared at parameter {first_parameter_index}"
            ),
            Self::MultipleVariadicParameters {
                first_parameter_index,
                duplicate_parameter_index,
            } => write!(
                f,
                "multiple variadic parameters at parameter {duplicate_parameter_index}; first variadic parameter is {first_parameter_index}"
            ),
            Self::VariadicParameterNotFinal { parameter_index } => {
                write!(f, "variadic parameter at {parameter_index} is not final")
            }
            Self::PositionalArgumentAfterNamedArgument { source_index } => write!(
                f,
                "positional call argument at source slot {source_index} follows a named argument"
            ),
            Self::UnsupportedSpreadArgument { source_index } => write!(
                f,
                "spread call argument at source slot {source_index} requires runtime unpack normalization"
            ),
            Self::TooManyPositionalArguments {
                source_index,
                max_positional_count,
            } => write!(
                f,
                "positional call argument at source slot {source_index} exceeds maximum positional count {max_positional_count}"
            ),
            Self::DuplicateArgument {
                parameter_name,
                first_source_index,
                duplicate_source_index,
            } => write!(
                f,
                "call argument for ${parameter_name} at source slot {duplicate_source_index} duplicates source slot {first_source_index}"
            ),
            Self::UnknownNamedArgument { name, source_index } => write!(
                f,
                "named call argument ${name} at source slot {source_index} does not match a parameter"
            ),
            Self::MissingRequiredArgument {
                parameter_name,
                parameter_index,
            } => write!(
                f,
                "required call parameter ${parameter_name} at parameter {parameter_index} is missing"
            ),
        }
    }
}

impl std::error::Error for CallArgumentNormalizationError {}

pub fn normalize_call_arguments(
    signature: &CallArgumentSignature,
    arguments: &[CallArgument],
) -> CallArgumentNormalizationResult<NormalizedCallArguments> {
    let fixed_count = signature.fixed_param_count();
    let mut supplied_fixed_sources: Vec<Option<usize>> = vec![None; fixed_count];
    let mut variadic_entries = Vec::new();
    let mut source_evaluations = Vec::with_capacity(arguments.len());
    let mut seen_named_argument = false;
    let mut next_positional_index = 0;
    let mut named_sources: HashMap<String, usize> = HashMap::new();

    for (source_index, argument) in arguments.iter().enumerate() {
        match argument {
            CallArgument::Spread => {
                return Err(CallArgumentNormalizationError::UnsupportedSpreadArgument {
                    source_index,
                });
            }
            CallArgument::Positional => {
                if seen_named_argument {
                    return Err(
                        CallArgumentNormalizationError::PositionalArgumentAfterNamedArgument {
                            source_index,
                        },
                    );
                }
                source_evaluations.push(CallArgumentSourceEvaluation {
                    source_index,
                    kind: CallArgumentSourceEvaluationKind::Positional,
                });
                if next_positional_index < fixed_count {
                    supplied_fixed_sources[next_positional_index] = Some(source_index);
                    next_positional_index += 1;
                } else if signature.variadic_param().is_some() {
                    variadic_entries.push(CallArgumentVariadicEntry {
                        source_index,
                        key: CallArgumentVariadicKey::NextInteger,
                    });
                } else {
                    return Err(CallArgumentNormalizationError::TooManyPositionalArguments {
                        source_index,
                        max_positional_count: fixed_count,
                    });
                }
            }
            CallArgument::Named(name) => {
                seen_named_argument = true;
                source_evaluations.push(CallArgumentSourceEvaluation {
                    source_index,
                    kind: CallArgumentSourceEvaluationKind::Named(name.clone()),
                });
                if let Some(first_source_index) = named_sources.get(name).copied() {
                    return Err(CallArgumentNormalizationError::DuplicateArgument {
                        parameter_name: name.clone(),
                        first_source_index,
                        duplicate_source_index: source_index,
                    });
                }
                named_sources.insert(name.clone(), source_index);

                if let Some(parameter_index) = signature.fixed_name_to_index.get(name).copied() {
                    if let Some(first_source_index) = supplied_fixed_sources[parameter_index] {
                        return Err(CallArgumentNormalizationError::DuplicateArgument {
                            parameter_name: name.clone(),
                            first_source_index,
                            duplicate_source_index: source_index,
                        });
                    }
                    supplied_fixed_sources[parameter_index] = Some(source_index);
                } else if signature.variadic_param().is_some() {
                    variadic_entries.push(CallArgumentVariadicEntry {
                        source_index,
                        key: CallArgumentVariadicKey::Named(name.clone()),
                    });
                } else {
                    return Err(CallArgumentNormalizationError::UnknownNamedArgument {
                        name: name.clone(),
                        source_index,
                    });
                }
            }
        }
    }

    let mut fixed_slots = Vec::with_capacity(fixed_count);
    for (parameter_index, param) in signature.params.iter().take(fixed_count).enumerate() {
        let source = match supplied_fixed_sources[parameter_index] {
            Some(source_index) => CallArgumentSlotSource::Supplied { source_index },
            None if param.required => {
                return Err(CallArgumentNormalizationError::MissingRequiredArgument {
                    parameter_name: param.name.clone(),
                    parameter_index,
                });
            }
            None => CallArgumentSlotSource::Default,
        };

        fixed_slots.push(CallArgumentFixedSlot {
            parameter_index,
            parameter_name: param.name.clone(),
            source,
            passing_mode: CallArgumentPassingMode::for_param(param),
        });
    }

    let variadic_slot =
        signature
            .variadic_param()
            .map(|(parameter_index, param)| CallArgumentVariadicSlot {
                parameter_index,
                parameter_name: param.name.clone(),
                entries: variadic_entries,
                entry_passing_mode: CallArgumentPassingMode::for_param(param),
            });

    let cleanup = CallArgumentCleanupPlan {
        source_indices_reverse: source_evaluations
            .iter()
            .rev()
            .map(|evaluation| evaluation.source_index)
            .collect(),
    };

    Ok(NormalizedCallArguments {
        source_evaluations,
        fixed_slots,
        variadic_slot,
        cleanup,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(params: Vec<CallArgumentParameter>) -> CallArgumentSignature {
        CallArgumentSignature::new(params).expect("test signature should be valid")
    }

    #[test]
    fn call_argument_normalization_preserves_source_order_and_binds_named_slots_by_parameter_order()
    {
        let signature = signature(vec![
            CallArgumentParameter::required("first"),
            CallArgumentParameter::required("slot").with_by_reference(),
            CallArgumentParameter::optional("mode"),
        ]);
        let plan = normalize_call_arguments(
            &signature,
            &[CallArgument::named("slot"), CallArgument::named("first")],
        )
        .expect("named arguments should normalize through the shared contract");

        assert_eq!(
            plan.source_evaluations,
            vec![
                CallArgumentSourceEvaluation {
                    source_index: 0,
                    kind: CallArgumentSourceEvaluationKind::Named("slot".to_string()),
                },
                CallArgumentSourceEvaluation {
                    source_index: 1,
                    kind: CallArgumentSourceEvaluationKind::Named("first".to_string()),
                },
            ]
        );
        assert_eq!(
            plan.fixed_slots,
            vec![
                CallArgumentFixedSlot {
                    parameter_index: 0,
                    parameter_name: "first".to_string(),
                    source: CallArgumentSlotSource::Supplied { source_index: 1 },
                    passing_mode: CallArgumentPassingMode::Value,
                },
                CallArgumentFixedSlot {
                    parameter_index: 1,
                    parameter_name: "slot".to_string(),
                    source: CallArgumentSlotSource::Supplied { source_index: 0 },
                    passing_mode: CallArgumentPassingMode::Reference,
                },
                CallArgumentFixedSlot {
                    parameter_index: 2,
                    parameter_name: "mode".to_string(),
                    source: CallArgumentSlotSource::Default,
                    passing_mode: CallArgumentPassingMode::Value,
                },
            ]
        );
        assert_eq!(plan.variadic_slot, None);
        assert_eq!(plan.cleanup.source_indices_reverse, vec![1, 0]);
    }

    #[test]
    fn call_argument_normalization_reports_duplicate_missing_unknown_and_order_diagnostics() {
        let signature = signature(vec![
            CallArgumentParameter::required("first"),
            CallArgumentParameter::required("second"),
        ]);

        assert_eq!(
            normalize_call_arguments(
                &signature,
                &[CallArgument::positional(), CallArgument::named("first")],
            )
            .unwrap_err(),
            CallArgumentNormalizationError::DuplicateArgument {
                parameter_name: "first".to_string(),
                first_source_index: 0,
                duplicate_source_index: 1,
            }
        );
        assert_eq!(
            normalize_call_arguments(
                &signature,
                &[CallArgument::named("first"), CallArgument::named("first")],
            )
            .unwrap_err(),
            CallArgumentNormalizationError::DuplicateArgument {
                parameter_name: "first".to_string(),
                first_source_index: 0,
                duplicate_source_index: 1,
            }
        );
        assert_eq!(
            normalize_call_arguments(&signature, &[CallArgument::named("missing")]).unwrap_err(),
            CallArgumentNormalizationError::UnknownNamedArgument {
                name: "missing".to_string(),
                source_index: 0,
            }
        );
        assert_eq!(
            normalize_call_arguments(&signature, &[CallArgument::named("second")]).unwrap_err(),
            CallArgumentNormalizationError::MissingRequiredArgument {
                parameter_name: "first".to_string(),
                parameter_index: 0,
            }
        );
        assert_eq!(
            normalize_call_arguments(
                &signature,
                &[CallArgument::named("second"), CallArgument::positional()],
            )
            .unwrap_err(),
            CallArgumentNormalizationError::PositionalArgumentAfterNamedArgument {
                source_index: 1,
            }
        );
    }

    #[test]
    fn call_argument_normalization_collects_variadic_entries_with_source_keys_and_reference_modes()
    {
        let signature = signature(vec![
            CallArgumentParameter::required("head"),
            CallArgumentParameter::optional("tail")
                .with_by_reference()
                .with_variadic(),
        ]);
        let plan = normalize_call_arguments(
            &signature,
            &[
                CallArgument::positional(),
                CallArgument::positional(),
                CallArgument::named("hook"),
            ],
        )
        .expect("mixed fixed and variadic arguments should normalize");

        assert_eq!(
            plan.fixed_slots,
            vec![CallArgumentFixedSlot {
                parameter_index: 0,
                parameter_name: "head".to_string(),
                source: CallArgumentSlotSource::Supplied { source_index: 0 },
                passing_mode: CallArgumentPassingMode::Value,
            }]
        );
        assert_eq!(
            plan.variadic_slot,
            Some(CallArgumentVariadicSlot {
                parameter_index: 1,
                parameter_name: "tail".to_string(),
                entries: vec![
                    CallArgumentVariadicEntry {
                        source_index: 1,
                        key: CallArgumentVariadicKey::NextInteger,
                    },
                    CallArgumentVariadicEntry {
                        source_index: 2,
                        key: CallArgumentVariadicKey::Named("hook".to_string()),
                    },
                ],
                entry_passing_mode: CallArgumentPassingMode::Reference,
            })
        );
        assert_eq!(plan.cleanup.source_indices_reverse, vec![2, 1, 0]);
    }

    #[test]
    fn call_argument_normalization_blocks_spread_until_unpack_can_feed_handle_slots() {
        let signature = signature(vec![CallArgumentParameter::required("value")]);
        let error = normalize_call_arguments(
            &signature,
            &[CallArgument::positional(), CallArgument::spread()],
        )
        .unwrap_err();

        assert_eq!(
            error,
            CallArgumentNormalizationError::UnsupportedSpreadArgument { source_index: 1 }
        );
        assert!(
            error.to_string().contains("runtime unpack normalization"),
            "{error}"
        );
    }

    #[test]
    fn call_argument_signature_rejects_malformed_parameter_metadata() {
        assert_eq!(
            CallArgumentSignature::new(vec![
                CallArgumentParameter::required("value"),
                CallArgumentParameter::optional("value"),
            ])
            .unwrap_err(),
            CallArgumentNormalizationError::DuplicateParameterName {
                name: "value".to_string(),
                first_parameter_index: 0,
                duplicate_parameter_index: 1,
            }
        );
        assert_eq!(
            CallArgumentSignature::new(vec![
                CallArgumentParameter::optional("rest").with_variadic(),
                CallArgumentParameter::optional("after"),
            ])
            .unwrap_err(),
            CallArgumentNormalizationError::VariadicParameterNotFinal { parameter_index: 0 }
        );
        assert_eq!(
            CallArgumentSignature::new(vec![
                CallArgumentParameter::optional("left").with_variadic(),
                CallArgumentParameter::optional("right").with_variadic(),
            ])
            .unwrap_err(),
            CallArgumentNormalizationError::VariadicParameterNotFinal { parameter_index: 0 }
        );
    }
}
