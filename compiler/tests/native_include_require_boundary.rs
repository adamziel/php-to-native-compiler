use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use php_compiler::codegen::{
    emit_native_executable_c_source, emit_native_executable_c_source_for_include_units,
};
use php_compiler::error::Phase;
use php_compiler::{
    compilation_unit_with_literal_include_metadata, emit_asm_source, emit_ir_source,
    executable_compilation_unit_with_literal_include_units,
};

const LLVM_REQUIRE_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles the current narrow include/require behavior";
const LLVM_REQUIRE_EXPRESSION_REJECTION: &str = "LLVM include/require lowering rejects multi-file execution for expression forms with include return values, _once de-duplication results, and caller-scope side effects until native source loading, path resolution, declaration registration, stack/source mapping, and exact native error behavior exist; phpc run handles current include/require expression behavior";

#[test]
fn emit_ir_rejects_statement_include_require_with_multifile_boundary() {
    for source in [
        "<?php\nrequire 'bootstrap.php';\n",
        "<?php\nrequire_once 'bootstrap.php';\n",
        "<?php\ninclude 'bootstrap.php';\n",
        "<?php\ninclude_once 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_expression_include_require_with_return_value_boundary() {
    for source in [
        "<?php\n$ok = require 'bootstrap.php';\n",
        "<?php\n$ok = require_once 'bootstrap.php';\n",
        "<?php\n$ok = include 'bootstrap.php';\n",
        "<?php\n$ok = include_once 'bootstrap.php';\n",
        "<?php\necho include 'bootstrap.php';\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_expression_include_require_before_backend_execution() {
    let error = emit_asm_source("<?php\n$ok = include 'bootstrap.php';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_REQUIRE_EXPRESSION_REJECTION);
}

#[test]
fn literal_include_graph_metadata_expands_declaration_only_class_files() {
    let dir = include_discovery_fixture_dir("metadata");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        "<?php\nclass IncludedMeta { public const TOKEN = 'T'; public static function mark() { return 'M'; } }\n",
    )
    .expect("write included metadata fixture");
    fs::write(
        &root,
        "<?php\nrequire __DIR__ . '/included.php';\necho class_exists('IncludedMeta') ? 'Y' : 'N';\n",
    )
    .expect("write root metadata fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root metadata fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_metadata.included_files.len(), 1);
    assert_eq!(
        unit.include_metadata.included_files[0].class_names,
        vec!["IncludedMeta".to_string()]
    );
    assert!(
        unit.program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, php_compiler::ast::Stmt::Class(class) if class.name == "IncludedMeta")),
        "literal include should expand to reusable class metadata statements"
    );
}

#[test]
fn literal_include_graph_metadata_expands_declaration_only_trait_and_interface_files() {
    let dir = include_discovery_fixture_dir("class-like-metadata");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "interface IncludedContract { public function label(); }\n",
            "trait IncludedTrait { public function traitLabel() { return 'trait'; } }\n",
        ),
    )
    .expect("write included class-like metadata fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "class UsesIncludedDeclarations { use IncludedTrait; }\n",
        ),
    )
    .expect("write root class-like metadata fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root class-like metadata fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_metadata.included_files.len(), 1);
    assert_eq!(
        unit.include_metadata.included_files[0].interface_names,
        vec!["IncludedContract".to_string()]
    );
    assert_eq!(
        unit.include_metadata.included_files[0].trait_names,
        vec!["IncludedTrait".to_string()]
    );
    assert!(
        unit.program.statements.iter().any(
            |stmt| matches!(stmt, php_compiler::ast::Stmt::Interface(interface) if interface.name == "IncludedContract")
        ),
        "literal include should expand to reusable interface metadata statements"
    );
    assert!(
        unit.program
            .statements
            .iter()
            .any(|stmt| matches!(stmt, php_compiler::ast::Stmt::Trait(trait_decl) if trait_decl.name == "IncludedTrait")),
        "literal include should expand to reusable trait metadata statements"
    );
}

#[test]
fn native_executable_c_source_uses_included_class_metadata_boundary() {
    let dir = include_discovery_fixture_dir("c-source");
    let root = write_supported_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_declare_user_class_parent_bytes")
            && source.contains("phpc_native_declare_user_class_method_bytes")
            && source.contains("phpc_native_static_property_storage_declare_properties_and_defaults_bytes")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_registry_and_diagnostic")
            && source.contains("phpc_native_class_constant_declare_constant_bytes_and_free"),
        "generated C should declare included class/method/property/constant metadata through shared runtime boundaries:\n{source}"
    );
}

#[test]
fn native_executable_c_source_uses_included_trait_metadata_boundary_with_trait_execution() {
    let dir = include_discovery_fixture_dir("trait-c-source");
    let root = write_supported_trait_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read trait c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_declare_user_class_bytes")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_registry_and_diagnostic")
            && source.contains(
                "phpc_native_callable_table_register_visibility_staticness_magic_signature_frame_callback_and_free"
            )
            && source.contains(
                "phpc_native_method_invoke_value_with_access_context_diagnostic_and_free_receiver_method_arguments"
            ),
        "generated C should accept classes using included trait metadata and route trait methods through declared frames:\n{source}"
    );
}

#[test]
fn native_executable_c_source_uses_included_trait_constructor_metadata_boundary() {
    let dir = include_discovery_fixture_dir("trait-constructor-c-source");
    let root = write_supported_trait_constructor_include_fixture(&dir);
    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read trait constructor c-source include fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("PHPC_NATIVE_CALLABLE_KIND_CONSTRUCTOR")
            && source.contains(
                "phpc_native_callable_table_register_visibility_staticness_magic_signature_frame_callback_and_free"
            )
            && source.contains(
                "phpc_native_constructor_allocation_invoke_value_with_access_context_diagnostic_and_free_scope_receiver_arguments"
            )
            && !source.contains("object-instantiation lowering rejects"),
        "included trait constructors should compose into declared constructor metadata and use shared constructor invocation:\n{source}"
    );
}

#[test]
fn native_executable_c_source_accepts_included_interface_metadata_boundary() {
    let dir = include_discovery_fixture_dir("interface-c-boundary");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        "<?php\ninterface IncludedContract { public function label(); }\n",
    )
    .expect("write included interface fixture");
    fs::write(
        &root,
        "<?php\nrequire __DIR__ . '/included.php';\necho 'after';\n",
    )
    .expect("write root interface fixture");

    let unit = compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(&root).expect("read root interface fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source(&unit.program).unwrap();

    assert!(
        source.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "included interface declarations should compose with generated-C metadata without blocking unrelated root execution:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_included_class_metadata_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("exe");
    let root = write_supported_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included class metadata executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included class metadata executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "exists|child|static|42|child-const|base-const\n"
    );
}

#[test]
fn emit_exe_links_and_runs_included_trait_method_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("trait-exe");
    let root = write_supported_trait_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included trait metadata executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included trait metadata executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "trait-meta|trait\n");
}

#[test]
fn emit_exe_links_and_runs_included_trait_constructor_consumers() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("trait-constructor-exe");
    let root = write_supported_trait_constructor_include_fixture(&dir);
    let output = dir.join("program");
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile included trait constructor executable");
    assert!(
        compile.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&output)
        .output()
        .expect("run included trait constructor executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "trait-ctor-meta|included-ctor\n"
    );
}

#[test]
fn emit_exe_links_and_runs_side_effecting_literal_include_unit() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-side-effects-exe");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    let output = dir.join("program");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "$value = 'included';\n",
            "echo 'inc:', $value;\n",
        ),
    )
    .expect("write side-effecting include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "echo '|', $value, \"\\n\";\n",
        ),
    )
    .expect("write root side-effecting include fixture");

    let output = compile_exe(&root, &output, "side-effecting include executable");
    let run = Command::new(&output)
        .output()
        .expect("run side-effecting include executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "inc:included|included\n"
    );
}

#[test]
fn emit_exe_links_and_runs_include_once_return_value_and_duplicate_true() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-once-return-exe");
    let root = dir.join("root.php");
    let included = dir.join("value.php");
    let output = dir.join("program");
    fs::write(&included, "<?php\nreturn 'loaded';\n").expect("write include return fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "$first = include_once __DIR__ . '/value.php';\n",
            "$second = include_once __DIR__ . '/value.php';\n",
            "echo $first, '|', $second, \"\\n\";\n",
        ),
    )
    .expect("write include_once return fixture");

    let output = compile_exe(&root, &output, "include_once return executable");
    let run = Command::new(&output)
        .output()
        .expect("run include_once return executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "loaded|1\n");
}

#[test]
fn emit_exe_links_and_runs_include_unit_return_through_finally() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-return-finally-exe");
    let root = dir.join("root.php");
    let included = dir.join("value.php");
    let output = dir.join("program");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "try {\n",
            "    echo 'try|';\n",
            "    return 'loaded';\n",
            "} finally {\n",
            "    echo 'finally|';\n",
            "}\n",
        ),
    )
    .expect("write include return-through-finally fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "$value = include __DIR__ . '/value.php';\n",
            "echo $value, '|after', \"\\n\";\n",
        ),
    )
    .expect("write include return-through-finally root fixture");

    let output = compile_exe(&root, &output, "include return-through-finally executable");
    let run = Command::new(&output)
        .output()
        .expect("run include return-through-finally executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "try|finally|loaded|after\n"
    );
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");
}

#[test]
fn native_executable_c_source_uses_include_execution_state_for_function_scope() {
    let dir = include_discovery_fixture_dir("execution-function-scope-c-source");
    let root = dir.join("root.php");
    fs::write(
        dir.join("scope.php"),
        "<?php\n$local = $prefix . 'included';\nreturn $local . ':return';\n",
    )
    .expect("write function-scope include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function load_scope($seed) {\n",
            "    $prefix = $seed . ':';\n",
            "    $value = include __DIR__ . '/scope.php';\n",
            "    echo $local, '|', $value, \"\\n\";\n",
            "}\n",
            "load_scope('caller');\n",
        ),
    )
    .expect("write function-scope root fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read function-scope root fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains(
            "typedef struct { phpc_NativeSymbolTableHandle scope_symbols; } phpc_NativeIncludeExecutionState;"
        ) && source.contains(
            "static phpc_NativeIncludeResult phpc_include_unit_0(phpc_NativeIncludeExecutionState *phpc_include_state)"
        ) && source.contains("phpc_include_state->scope_symbols")
            && source.contains("phpc_NativeIncludeExecutionState include_execution_state_")
            && !source.contains("phpc_include_unit_0(phpc_root_symbols)"),
        "generated C should pass include execution state instead of a hard-coded root symbol table:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_function_scope_literal_include_unit_symbols_and_return() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-function-scope-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        dir.join("scope.php"),
        "<?php\n$local = $prefix . 'included';\nreturn $local . ':return';\n",
    )
    .expect("write function-scope executable include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function load_scope($seed) {\n",
            "    $prefix = $seed . ':';\n",
            "    $value = include __DIR__ . '/scope.php';\n",
            "    echo $local, '|', $value, \"\\n\";\n",
            "}\n",
            "load_scope('caller');\n",
        ),
    )
    .expect("write function-scope executable root fixture");

    let output = compile_exe(&root, &output, "function-scope include executable");
    let run = Command::new(&output)
        .output()
        .expect("run function-scope include executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "caller:included|caller:included:return\n"
    );
}

#[test]
fn native_executable_c_source_tags_include_unit_exit_as_termination() {
    let dir = include_discovery_fixture_dir("execution-exit-tag-c-source");
    let root = dir.join("root.php");
    fs::write(dir.join("stop.php"), "<?php\necho 'included|';\nexit();\n")
        .expect("write include exit c-source fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/stop.php';\n",
            "echo 'after';\n",
        ),
    )
    .expect("write include exit root c-source fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read include exit root fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains("#define PHPC_NATIVE_INCLUDE_RESULT_TERMINATE 3")
            && source.contains(
                "return (phpc_NativeIncludeResult){PHPC_NATIVE_INCLUDE_RESULT_TERMINATE"
            )
            && source.contains(".tag == PHPC_NATIVE_INCLUDE_RESULT_TERMINATE")
            && !source.contains(".exit_code != 0"),
        "include-unit exit should propagate through an explicit control-transfer tag instead of an ordinary zero exit-code value:\n{source}"
    );
}

#[test]
fn emit_exe_include_unit_exit_zero_terminates_caller_without_after() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-exit-zero-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(dir.join("stop.php"), "<?php\necho 'included|';\nexit();\n")
        .expect("write include exit executable fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "echo 'before|';\n",
            "require __DIR__ . '/stop.php';\n",
            "echo 'after';\n",
        ),
    )
    .expect("write include exit executable root fixture");

    let output = compile_exe(&root, &output, "include unit exit zero executable");
    let run = Command::new(&output)
        .output()
        .expect("run include unit exit zero executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "before|included|");
    assert_eq!(String::from_utf8_lossy(&run.stderr), "");
}

#[test]
fn emit_exe_links_and_runs_literal_include_path_search_before_source_fallback() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-include-path-precedence-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create include_path lib fixture dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        lib.join("shared.php"),
        "<?php\n$origin = 'include-path';\necho 'lib|';\nreturn 'lib-return';\n",
    )
    .expect("write include_path fixture");
    fs::write(
        dir.join("shared.php"),
        "<?php\n$origin = 'source';\necho 'source|';\nreturn 'source-return';\n",
    )
    .expect("write source fallback fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$value = include 'shared.php';\n",
            "echo $origin, '|', $value, \"\\n\";\n",
        ),
    )
    .expect("write include_path root fixture");

    let output = compile_exe(&root, &output, "include_path search executable");
    let run = Command::new(&output)
        .output()
        .expect("run include_path search executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "lib|include-path|lib-return\n"
    );
}

#[test]
fn emit_exe_links_and_runs_literal_include_path_source_relative_fallback() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-include-path-fallback-exe");
    let missing = dir.join("missing");
    fs::create_dir_all(&missing).expect("create missing include_path fixture dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        dir.join("fallback.php"),
        "<?php\necho 'fallback|';\nreturn 'fallback-return';\n",
    )
    .expect("write source fallback fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/missing');\n",
            "$value = require 'fallback.php';\n",
            "echo $value, \"\\n\";\n",
        ),
    )
    .expect("write include_path fallback root fixture");

    let output = compile_exe(&root, &output, "include_path fallback executable");
    let run = Command::new(&output)
        .output()
        .expect("run include_path fallback executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "fallback|fallback-return\n"
    );
}

#[test]
fn emit_exe_links_and_runs_include_path_once_canonical_duplicate_true() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-include-path-once-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create include_path once fixture dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        lib.join("once.php"),
        "<?php\necho 'once|';\nreturn 'once-return';\n",
    )
    .expect("write include_path once fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$first = include_once 'once.php';\n",
            "$second = include_once __DIR__ . '/lib/once.php';\n",
            "echo $first, '|', $second, \"\\n\";\n",
        ),
    )
    .expect("write include_path once root fixture");

    let output = compile_exe(&root, &output, "include_path once executable");
    let run = Command::new(&output)
        .output()
        .expect("run include_path once executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "once|once-return|1\n");
}

#[test]
fn executable_include_discovery_tags_variable_string_found_and_missing_results() {
    let dir = include_discovery_fixture_dir("execution-variable-discovery");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create variable discovery include_path dir");
    let root = dir.join("root.php");
    fs::write(lib.join("present.php"), "<?php\nreturn 'present';\n")
        .expect("write variable discovery include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$present = 'present.php';\n",
            "$missing = 'missing.php';\n",
            "$first = include $present;\n",
            "$second = include $missing;\n",
        ),
    )
    .expect("write variable discovery root fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read variable discovery root fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_units.len(), 1);
    assert_eq!(unit.include_resolutions.len(), 2);
    assert!(unit.include_resolutions[0].found);
    assert_eq!(unit.include_resolutions[0].requested_path, "present.php");
    assert!(!unit.include_resolutions[1].found);
    assert_eq!(unit.include_resolutions[1].requested_path, "missing.php");
}

#[test]
fn native_executable_c_source_uses_runtime_string_dispatch_for_variable_include_path() {
    let dir = include_discovery_fixture_dir("execution-variable-c-source");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create variable c-source include_path dir");
    let root = dir.join("root.php");
    fs::write(lib.join("present.php"), "<?php\nreturn 'present';\n")
        .expect("write variable c-source include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$present = 'present.php';\n",
            "$value = include $present;\n",
            "echo $value;\n",
        ),
    )
    .expect("write variable c-source root fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read variable c-source root fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains("phpc_native_value_to_string_bytes")
            && source.contains("memcmp")
            && source.contains("phpc_include_unit_0"),
        "variable include operands should convert the runtime path string and dispatch to include units through shared helpers:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_variable_string_include_path_found_and_missing_results() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-variable-include-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create variable include_path fixture dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        lib.join("value.php"),
        "<?php\necho 'loaded|';\nreturn 'value-return';\n",
    )
    .expect("write variable include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$present = 'value.php';\n",
            "$missing = 'missing.php';\n",
            "$first = include $present;\n",
            "$second = include $missing;\n",
            "echo $first, '|', ($second === false ? 'false' : 'bad'), \"\\n\";\n",
        ),
    )
    .expect("write variable include root fixture");

    let output = compile_exe(&root, &output, "variable include executable");
    let run = Command::new(&output)
        .output()
        .expect("run variable include executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "loaded|value-return|false\n"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("include(missing.php)"), "{stderr}");
}

#[test]
fn emit_exe_links_and_runs_variable_string_require_result() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-variable-require-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        dir.join("required.php"),
        "<?php\necho 'required|';\nreturn 'require-return';\n",
    )
    .expect("write variable require fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "$path = 'required.php';\n",
            "$value = require $path;\n",
            "echo $value, \"\\n\";\n",
        ),
    )
    .expect("write variable require root fixture");

    let output = compile_exe(&root, &output, "variable require executable");
    let run = Command::new(&output)
        .output()
        .expect("run variable require executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "required|require-return\n"
    );
}

#[test]
fn native_executable_c_source_uses_runtime_include_unit_registry_for_nonfinite_path_lookup() {
    let dir = include_discovery_fixture_dir("runtime-registry-c-source");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime registry c-source lib dir");
    fs::write(lib.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime registry c-source include");
    let root = dir.join("root.php");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_path() { return 'declared.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "$value = include_once $path;\n",
            "echo $value;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write runtime registry c-source root");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read runtime registry c-source root"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains("phpc_native_include_unit_registry_lookup")
            && source.contains("phpc_native_include_unit_registry_lookup_include_path")
            && source.contains("phpc_NativeIncludeUnitLookupEntry")
            && source.contains("phpc_include_unit_0"),
        "non-finite include paths should dispatch through the generated include-unit registry:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_runtime_include_registry_include_path_search() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-include-path-search-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime registry include_path lib dir");
    fs::write(
        lib.join("declared.php"),
        "<?php\necho 'declared|';\nreturn 'include-return';\n",
    )
    .expect("write runtime registry include_path include");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_include_path() { return 'declared.php'; }\n",
            "require_once __DIR__ . '/lib/declared.php';\n",
            "$path = runtime_include_path();\n",
            "$value = include $path;\n",
            "echo $value;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write runtime registry include_path search root");

    let output = compile_exe(
        &root,
        &output,
        "runtime include registry include_path search executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime include registry include_path search executable");

    assert!(
        run.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "declared|declared|include-return\n"
    );
}

#[test]
fn emit_exe_links_and_runs_runtime_require_registry_include_path_search() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-require-path-search-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime registry require include_path lib dir");
    fs::write(
        lib.join("required.php"),
        "<?php\necho 'required|';\nreturn 'require-return';\n",
    )
    .expect("write runtime registry require include_path include");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_require_path() { return 'required.php'; }\n",
            "include_once __DIR__ . '/lib/required.php';\n",
            "$path = runtime_require_path();\n",
            "$value = require $path;\n",
            "echo $value;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write runtime registry require include_path search root");

    let output = compile_exe(
        &root,
        &output,
        "runtime require registry include_path search executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime require registry include_path search executable");

    assert!(
        run.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "required|required|require-return\n"
    );
}

#[test]
fn emit_exe_links_and_runs_runtime_once_registry_include_path_search() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-once-path-search-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime registry once include_path lib dir");
    fs::write(
        lib.join("once.php"),
        "<?php\necho 'once|';\nreturn 'once-return';\n",
    )
    .expect("write runtime registry once include_path include");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_once_path() { return 'once.php'; }\n",
            "$first = include_once __DIR__ . '/lib/once.php';\n",
            "$path = runtime_once_path();\n",
            "$second = include_once $path;\n",
            "$third = require_once $path;\n",
            "echo $first, '|', $second, '|', $third, \"\\n\";\n",
        ),
    )
    .expect("write runtime registry once include_path search root");

    let output = compile_exe(
        &root,
        &output,
        "runtime once registry include_path search executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime once registry include_path search executable");

    assert!(
        run.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "once|once-return|1|1\n"
    );
}

#[test]
fn emit_exe_links_and_runs_runtime_include_registry_request_path_lookup() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-request-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime registry request lib dir");
    fs::write(lib.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime registry request include");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_path() { return 'declared.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "$value = include_once $path;\n",
            "echo $value;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write runtime registry request root");

    let output = compile_exe(
        &root,
        &output,
        "runtime include registry request executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime include registry request executable");

    assert!(
        run.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "declared|1\n");
}

#[test]
fn emit_exe_links_and_runs_runtime_include_registry_canonical_path_lookup() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-canonical-exe");
    let included = dir.join("declared.php");
    fs::write(&included, "<?php\necho 'canonical|';\n")
        .expect("write runtime registry canonical include");
    let canonical = fs::canonicalize(&included)
        .expect("canonicalize runtime registry include")
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "\\'");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        format!(
            concat!(
                "<?php\n",
                "function runtime_path() {{ return '{canonical}'; }}\n",
                "require_once 'declared.php';\n",
                "$path = runtime_path();\n",
                "$value = require_once $path;\n",
                "echo $value;\n",
                "echo \"\\n\";\n",
            ),
            canonical = canonical
        ),
    )
    .expect("write runtime registry canonical root");

    let output = compile_exe(
        &root,
        &output,
        "runtime include registry canonical executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime include registry canonical executable");

    assert!(
        run.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "canonical|1\n");
}

#[test]
fn native_executable_c_source_uses_included_function_source_dir_for_runtime_include_registry() {
    let dir = include_discovery_fixture_dir("runtime-registry-included-function-c-source");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create included-function runtime registry lib dir");
    fs::write(
        lib.join("payload.php"),
        "<?php\necho 'payload|';\n$payloadLocal = 'local';\nreturn 'payload-return';\n",
    )
    .expect("write included-function runtime registry payload");
    fs::write(
        lib.join("loader.php"),
        concat!(
            "<?php\n",
            "function included_loader($name) {\n",
            "    $value = include $name;\n",
            "    echo $payloadLocal, '|', $value, \"\\n\";\n",
            "}\n",
        ),
    )
    .expect("write included-function runtime registry loader");
    let root = dir.join("root.php");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require_once __DIR__ . '/lib/payload.php';\n",
            "require_once __DIR__ . '/lib/loader.php';\n",
            "included_loader('payload.php');\n",
        ),
    )
    .expect("write included-function runtime registry root");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read included-function runtime registry root"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();
    let canonical_lib = fs::canonicalize(&lib).expect("canonicalize included-function lib dir");
    let source_dir_bytes = c_byte_array_for_test(canonical_lib.to_string_lossy().as_bytes());

    assert!(
        source.contains("phpc_native_include_unit_registry_lookup_include_path")
            && source.contains("phpc_NativeIncludeExecutionState include_execution_state_")
            && source.contains("phpc_include_state->scope_symbols")
            && source.contains(&format!("= {{{source_dir_bytes}}};")),
        "runtime include in an included-file function should use that function's declaring directory and still execute include units through scoped include state:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_source_loader")
            && !source.contains("phpc_runtime_source_loader")
            && !source.contains("class_prefix")
            && !source.contains("generated_source_table"),
        "included-function runtime registry lookup must not synthesize a parser/source-loader path:\n{source}"
    );
}

#[test]
fn emit_exe_links_and_runs_included_function_runtime_registry_source_relative_include() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-registry-included-function-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create included-function runtime registry exe lib dir");
    fs::write(
        lib.join("payload.php"),
        "<?php\necho 'payload|';\n$payloadLocal = 'local';\nreturn 'payload-return';\n",
    )
    .expect("write included-function runtime registry exe payload");
    fs::write(
        lib.join("loader.php"),
        concat!(
            "<?php\n",
            "function included_loader($name) {\n",
            "    $value = include $name;\n",
            "    echo $payloadLocal, '|', $value, \"\\n\";\n",
            "}\n",
        ),
    )
    .expect("write included-function runtime registry exe loader");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require_once __DIR__ . '/lib/payload.php';\n",
            "require_once __DIR__ . '/lib/loader.php';\n",
            "included_loader('payload.php');\n",
        ),
    )
    .expect("write included-function runtime registry exe root");

    let output = compile_exe(
        &root,
        &output,
        "included-function runtime registry source-relative executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run included-function runtime registry executable");

    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "payload|payload|local|payload-return\n"
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).is_empty(),
        "runtime registry should resolve the compiled payload from the included function's source directory without source-loader diagnostics:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn native_executable_c_source_uses_runtime_include_no_match_diagnostic_boundary() {
    let dir = include_discovery_fixture_dir("runtime-no-match-c-source");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime no-match declared include");
    let root = dir.join("root.php");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function runtime_path() { return 'missing.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "$value = include $path;\n",
            "echo $value === false ? 'false' : 'bad';\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write runtime no-match c-source root");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read runtime no-match c-source root"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains("phpc_native_include_unit_registry_lookup")
            && source.contains("phpc_native_include_runtime_no_match_diagnostic")
            && source.contains("phpc_NativeByteBuffer resolved_path")
            && source.contains("phpc_native_byte_buffer_free")
            && source.contains("PHPC_NATIVE_INCLUDE_RUNTIME_NO_MATCH_MISSING"),
        "registry no-match should route through the shared runtime diagnostic/source-loader boundary:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_source_loader")
            && !source.contains("phpc_runtime_source_loader")
            && !source.contains("class_prefix")
            && !source.contains("class_prefix_map")
            && !source.contains("generated_source_table")
            && !source.contains("spl_autoload_register_callback_")
            && !source.contains("dynamic_user_function_matched_"),
        "registry no-match must not synthesize runtime source loaders, class-prefix maps, source tables, or callback-name branches:\n{source}"
    );
}

#[test]
fn native_executable_c_source_routes_autoload_callback_include_to_runtime_boundary() {
    let dir = include_discovery_fixture_dir("autoload-callback-include-runtime-c-source");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write autoload callback runtime declared include");
    let root = dir.join("root.php");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require_once __DIR__ . '/declared.php';\n",
            "function loader($name) {\n",
            "    include $name . '.php';\n",
            "}\n",
            "spl_autoload_register('loader');\n",
            "echo class_exists('RuntimeLoaded') ? 'loaded' : 'missing';\n",
        ),
    )
    .expect("write autoload callback include runtime c-source root");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read autoload callback include runtime c-source root"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();

    assert!(
        source.contains("phpc_native_include_unit_registry_lookup_include_path")
            && source.contains("phpc_native_include_runtime_no_match_diagnostic")
            && source.contains("phpc_native_value_class_metadata_exists_with_autoload_registry_and_diagnostic")
            && !source.contains("spl_autoload_register_callback_")
            && !source.contains("dynamic_user_function_matched_"),
        "autoload callbacks with include should use the generated callback frame, SPL registry, and shared runtime include boundary:\n{source}"
    );
}

#[test]
fn emit_exe_runtime_registry_no_match_include_warns_returns_false_and_continues() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-no-match-include-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime no-match include declared fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function runtime_path() { return 'missing.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "$value = include $path;\n",
            "echo $value === false ? 'false' : 'bad';\n",
            "echo '|continued', \"\\n\";\n",
        ),
    )
    .expect("write runtime no-match include root");

    let output = compile_exe(&root, &output, "runtime no-match include executable");
    let run = Command::new(&output)
        .output()
        .expect("run runtime no-match include executable");

    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "declared|false|continued\n"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("include(missing.php)"), "{stderr}");
}

#[test]
fn emit_exe_runtime_registry_no_match_require_warns_fatals() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-no-match-require-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime no-match require declared fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function runtime_path() { return 'missing-required.php'; }\n",
            "echo 'before|';\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "require $path;\n",
            "echo 'after';\n",
        ),
    )
    .expect("write runtime no-match require root");

    let output = compile_exe(&root, &output, "runtime no-match require executable");
    let run = Command::new(&output)
        .output()
        .expect("run runtime no-match require executable");

    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "before|declared|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("require(missing-required.php)"), "{stderr}");
    assert!(stderr.contains("PHP Fatal error"), "{stderr}");
    assert!(
        stderr.contains("Failed opening required 'missing-required.php'"),
        "{stderr}"
    );
}

#[test]
fn emit_exe_runtime_registry_no_match_existing_source_blocks_on_loader_abi() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-no-match-existing-source-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime existing-source declared fixture");
    fs::write(dir.join("extra.php"), "<?php\necho 'extra|';\n")
        .expect("write undeclared runtime source fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "function runtime_path() { return 'extra.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "include $path;\n",
            "echo 'after';\n",
        ),
    )
    .expect("write runtime existing-source root");

    let output = compile_exe(&root, &output, "runtime existing-source blocker executable");
    let run = Command::new(&output)
        .output()
        .expect("run runtime existing-source blocker executable");

    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "declared|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("existing filesystem source"), "{stderr}");
    assert!(
        stderr.contains("native source loading/parsing ABI"),
        "{stderr}"
    );
}

#[test]
fn emit_exe_runtime_registry_no_match_existing_include_path_source_blocks_on_loader_abi() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("runtime-no-match-existing-include-path-source-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create runtime existing include_path lib dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(lib.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write runtime existing include_path declared fixture");
    fs::write(lib.join("extra.php"), "<?php\necho 'extra|';\n")
        .expect("write undeclared runtime include_path source fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "function runtime_path() { return 'extra.php'; }\n",
            "require_once 'declared.php';\n",
            "$path = runtime_path();\n",
            "include $path;\n",
            "echo 'after';\n",
        ),
    )
    .expect("write runtime existing include_path source root");

    let output = compile_exe(
        &root,
        &output,
        "runtime existing include_path source blocker executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run runtime existing include_path source blocker executable");

    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "declared|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("resolved 'extra.php'"), "{stderr}");
    assert!(stderr.contains("existing filesystem source"), "{stderr}");
    assert!(
        stderr.contains("native source loading/parsing ABI"),
        "{stderr}"
    );
}

#[test]
fn emit_exe_autoload_callback_existing_source_blocks_on_loader_abi() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("autoload-callback-existing-source-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    let runtime_source = dir.join("RuntimeLoaded.php");
    fs::write(dir.join("declared.php"), "<?php\necho 'declared|';\n")
        .expect("write autoload runtime declared fixture");
    fs::write(&runtime_source, "<?php\nclass RuntimeLoaded {}\n")
        .expect("write undeclared autoload runtime source fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require_once __DIR__ . '/declared.php';\n",
            "function loader($name) {\n",
            "    include $name . '.php';\n",
            "    echo 'after-loader';\n",
            "}\n",
            "spl_autoload_register('loader');\n",
            "echo class_exists('RuntimeLoaded') ? 'loaded' : 'missing';\n",
            "echo 'after';\n",
        ),
    )
    .expect("write autoload existing source root");

    let output = compile_exe(
        &root,
        &output,
        "autoload callback existing source blocker executable",
    );
    let run = Command::new(&output)
        .output()
        .expect("run autoload callback existing source blocker executable");

    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "declared|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("RuntimeLoaded.php"), "{stderr}");
    assert!(
        stderr.contains(runtime_source.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(stderr.contains("existing filesystem source"), "{stderr}");
    assert!(
        stderr.contains("native source loading/parsing ABI"),
        "{stderr}"
    );
    assert!(
        stderr.contains("native callable-value invocation failed"),
        "{stderr}"
    );
}

#[test]
fn emit_exe_blocks_dynamic_include_path_search_before_native_link() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-dynamic-include-path-blocker");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        "<?php\n$dir = __DIR__ . '/lib';\nset_include_path($dir);\ninclude 'value.php';\n",
    )
    .expect("write dynamic include_path blocker fixture");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile dynamic include_path blocker executable");

    assert!(!compile.status.success());
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains("literal compile-time path string"),
        "stderr should report dynamic include_path blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn executable_include_discovery_tags_missing_and_found_literal_results() {
    let dir = include_discovery_fixture_dir("execution-missing-discovery");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create missing discovery include_path dir");
    let root = dir.join("root.php");
    fs::write(
        lib.join("present.php"),
        "<?php\n$loaded = 'present';\nreturn 'loaded';\n",
    )
    .expect("write present include fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$missing = include 'absent.php';\n",
            "$present = require 'present.php';\n",
            "echo $loaded;\n",
        ),
    )
    .expect("write missing discovery root fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read missing discovery root fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();

    assert_eq!(unit.include_units.len(), 1);
    assert_eq!(unit.include_resolutions.len(), 2);
    assert!(!unit.include_resolutions[0].found);
    assert_eq!(unit.include_resolutions[0].requested_path, "absent.php");
    assert!(unit.include_resolutions[0].include_path.ends_with("/lib"));
    assert!(unit.include_resolutions[1].found);
    assert!(unit.include_resolutions[1].path.ends_with("present.php"));
}

#[test]
fn emit_exe_links_and_runs_missing_include_returns_false_and_continues() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-missing-include-exe");
    let lib = dir.join("lib");
    fs::create_dir_all(&lib).expect("create missing include include_path dir");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "set_include_path(__DIR__ . '/lib');\n",
            "$first = include 'optional.php';\n",
            "$second = include_once __DIR__ . '/optional-once.php';\n",
            "echo 'first=', ($first === false ? 'false' : 'bad');\n",
            "echo '|second=', ($second === false ? 'false' : 'bad');\n",
            "echo '|continued', \"\\n\";\n",
        ),
    )
    .expect("write missing include root fixture");

    let output = compile_exe(&root, &output, "missing include executable");
    let run = Command::new(&output)
        .output()
        .expect("run missing include executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "first=false|second=false|continued\n"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("include(optional.php)"), "{stderr}");
    assert!(stderr.contains("include_once("), "{stderr}");
    assert!(stderr.contains("include_path='"), "{stderr}");
}

#[test]
fn emit_exe_missing_require_statement_exits_after_warning_and_fatal() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-missing-require-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        "<?php\necho 'before|';\nrequire 'required.php';\necho 'after';\n",
    )
    .expect("write missing require root fixture");

    let output = compile_exe(&root, &output, "missing require executable");
    let run = Command::new(&output)
        .output()
        .expect("run missing require executable");
    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "before|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("require(required.php)"), "{stderr}");
    assert!(stderr.contains("PHP Fatal error"), "{stderr}");
    assert!(
        stderr.contains("Failed opening required 'required.php'"),
        "{stderr}"
    );
}

#[test]
fn native_executable_c_source_runs_active_finally_before_missing_required_include_fatal() {
    let dir = include_discovery_fixture_dir("missing-require-finally-c-source");
    let root = dir.join("root.php");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "try {\n",
            "    echo 'before|';\n",
            "    require 'required.php';\n",
            "} finally {\n",
            "    echo 'finally|';\n",
            "}\n",
            "echo 'after';\n",
        ),
    )
    .expect("write missing require finally c-source fixture");

    let unit = executable_compilation_unit_with_literal_include_units(
        &fs::read_to_string(&root).expect("read missing require finally c-source fixture"),
        &root,
        workspace_root(),
    )
    .unwrap();
    let source = emit_native_executable_c_source_for_include_units(&unit).unwrap();
    let body = main_body(&source);

    let warning = body
        .find("PHPC_NATIVE_DIAGNOSTIC_SEVERITY_WARNING")
        .unwrap_or_else(|| panic!("missing include warning diagnostic:\n{source}"));
    let finally_output = warning
        + body[warning..]
            .find("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            .unwrap_or_else(|| panic!("missing active finally output after warning:\n{source}"));
    let fatal = body
        .find("PHPC_NATIVE_DIAGNOSTIC_SEVERITY_ERROR")
        .unwrap_or_else(|| panic!("missing required include fatal diagnostic:\n{source}"));
    let exit = fatal
        + body[fatal..]
            .find("return 255;")
            .unwrap_or_else(|| panic!("missing required include fatal exit:\n{source}"));

    assert!(
        warning < finally_output && finally_output < fatal && fatal < exit,
        "active finally output must be emitted after the warning and before the fatal/255 exit:\n{source}"
    );
}

#[test]
fn emit_exe_missing_require_statement_runs_active_finally_before_fatal() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-missing-require-finally-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "try {\n",
            "    echo 'before|';\n",
            "    require 'required.php';\n",
            "} finally {\n",
            "    echo 'finally|';\n",
            "}\n",
            "echo 'after';\n",
        ),
    )
    .expect("write missing require finally root fixture");

    let output = compile_exe(&root, &output, "missing require finally executable");
    let run = Command::new(&output)
        .output()
        .expect("run missing require finally executable");
    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "before|finally|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("require(required.php)"), "{stderr}");
    assert!(stderr.contains("PHP Fatal error"), "{stderr}");
    assert!(
        stderr.contains("Failed opening required 'required.php'"),
        "{stderr}"
    );
}

#[test]
fn emit_exe_missing_require_once_expression_exits_after_warning_and_fatal() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-missing-require-once-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "echo 'start|';\n",
            "$value = require_once __DIR__ . '/required-once.php';\n",
            "echo 'after';\n",
        ),
    )
    .expect("write missing require_once root fixture");

    let output = compile_exe(&root, &output, "missing require_once executable");
    let run = Command::new(&output)
        .output()
        .expect("run missing require_once executable");
    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "start|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("require_once("), "{stderr}");
    assert!(stderr.contains("PHP Fatal error"), "{stderr}");
    assert!(
        stderr.contains("Failed opening required"),
        "stderr should report require_once fatal opening failure:\n{stderr}"
    );
}

#[test]
fn emit_exe_missing_require_once_expression_runs_active_finally_before_fatal() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-missing-require-once-finally-exe");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "try {\n",
            "    echo 'start|';\n",
            "    $value = require_once __DIR__ . '/required-once.php';\n",
            "    echo 'unreachable';\n",
            "} finally {\n",
            "    echo 'finally|';\n",
            "}\n",
            "echo 'after';\n",
        ),
    )
    .expect("write missing require_once finally root fixture");

    let output = compile_exe(&root, &output, "missing require_once finally executable");
    let run = Command::new(&output)
        .output()
        .expect("run missing require_once finally executable");
    assert!(!run.status.success());
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "start|finally|");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("PHP Warning"), "{stderr}");
    assert!(stderr.contains("require_once("), "{stderr}");
    assert!(stderr.contains("PHP Fatal error"), "{stderr}");
    assert!(
        stderr.contains("Failed opening required"),
        "stderr should report require_once fatal opening failure:\n{stderr}"
    );
}

#[test]
fn emit_exe_blocks_dynamic_include_paths_before_native_link() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-dynamic-blocker");
    let root = dir.join("root.php");
    let output = dir.join("program");
    fs::write(&root, "<?php\n$value = include $name;\necho $value;\n")
        .expect("write dynamic include blocker fixture");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile dynamic include blocker executable");

    assert!(!compile.status.success());
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains("generated include-unit registry"),
        "stderr should report dynamic include path blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn emit_exe_links_and_runs_root_include_once_cycle_with_duplicate_true() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-root-once-cycle-exe");
    let root = dir.join("root.php");
    let other = dir.join("other.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "echo 'root:start|';\n",
            "require_once __DIR__ . '/other.php';\n",
            "echo '|root:end', \"\\n\";\n",
        ),
    )
    .expect("write root cyclic once fixture");
    fs::write(
        &other,
        concat!(
            "<?php\n",
            "echo 'other:start|';\n",
            "$again = require_once __DIR__ . '/root.php';\n",
            "echo 'again=', $again, '|other:end';\n",
        ),
    )
    .expect("write nested cyclic once fixture");

    let output = compile_exe(&root, &output, "root include_once cycle executable");
    let run = Command::new(&output)
        .output()
        .expect("run root include_once cycle executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "root:start|other:start|again=1|other:end|root:end\n"
    );
}

#[test]
fn emit_exe_links_and_runs_nested_include_once_cycle_with_duplicate_true() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-nested-once-cycle-exe");
    let root = dir.join("root.php");
    let first = dir.join("first.php");
    let second = dir.join("second.php");
    let output = dir.join("program");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require_once __DIR__ . '/first.php';\n",
            "echo '|root:end', \"\\n\";\n",
        ),
    )
    .expect("write root nested cyclic once fixture");
    fs::write(
        &first,
        concat!(
            "<?php\n",
            "echo 'first:start|';\n",
            "require_once __DIR__ . '/second.php';\n",
            "echo '|first:end';\n",
        ),
    )
    .expect("write first cyclic once fixture");
    fs::write(
        &second,
        concat!(
            "<?php\n",
            "echo 'second:start|';\n",
            "$again = require_once __DIR__ . '/first.php';\n",
            "echo 'again=', $again, '|second:end';\n",
        ),
    )
    .expect("write second cyclic once fixture");

    let output = compile_exe(&root, &output, "nested include_once cycle executable");
    let run = Command::new(&output)
        .output()
        .expect("run nested include_once cycle executable");
    assert!(
        run.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "first:start|second:start|again=1|second:end|first:end|root:end\n"
    );
}

#[test]
fn emit_exe_blocks_non_once_cyclic_include_paths_before_native_link() {
    if !has_cc() {
        return;
    }

    let dir = include_discovery_fixture_dir("execution-cycle-blocker");
    let root = dir.join("root.php");
    let other = dir.join("other.php");
    let output = dir.join("program");
    fs::write(&root, "<?php\nrequire __DIR__ . '/other.php';\n")
        .expect("write root cyclic include fixture");
    fs::write(&other, "<?php\nrequire __DIR__ . '/root.php';\n")
        .expect("write nested cyclic include fixture");

    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .expect("compile cyclic include blocker executable");

    assert!(!compile.status.success());
    assert!(
        String::from_utf8_lossy(&compile.stderr).contains("cyclic include graph"),
        "stderr should report cyclic include graph blocker:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert!(!output.exists());
}

#[test]
fn literal_include_graph_blocks_dynamic_missing_cyclic_and_side_effecting_files() {
    let dir = include_discovery_fixture_dir("blockers");
    fs::write(dir.join("class.php"), "<?php\nclass BlockedMeta {}\n").unwrap();

    let dynamic_root = dir.join("dynamic.php");
    fs::write(&dynamic_root, "<?php\nrequire $file;\n").unwrap();
    let dynamic = include_discovery_error(&dynamic_root);
    assert_eq!(dynamic.phase, Phase::Parse);
    assert!(dynamic.message.contains("literal same-repository path"));

    let missing_root = dir.join("missing.php");
    fs::write(&missing_root, "<?php\nrequire __DIR__ . '/absent.php';\n").unwrap();
    let missing = include_discovery_error(&missing_root);
    assert_eq!(missing.phase, Phase::Io);

    let cycle_a = dir.join("cycle-a.php");
    let cycle_b = dir.join("cycle-b.php");
    fs::write(&cycle_a, "<?php\nrequire __DIR__ . '/cycle-b.php';\n").unwrap();
    fs::write(&cycle_b, "<?php\nrequire __DIR__ . '/cycle-a.php';\n").unwrap();
    let cycle = include_discovery_error(&cycle_a);
    assert_eq!(cycle.phase, Phase::Parse);
    assert!(cycle.message.contains("cyclic include graph"));

    let side_effect_inc = dir.join("side-effect.inc");
    let side_effect_root = dir.join("side-effect.php");
    fs::write(
        &side_effect_inc,
        "<?php\necho 'side effect';\nclass SideEffect {}\n",
    )
    .unwrap();
    fs::write(
        &side_effect_root,
        "<?php\nrequire __DIR__ . '/side-effect.inc';\n",
    )
    .unwrap();
    let side_effect = include_discovery_error(&side_effect_root);
    assert_eq!(side_effect.phase, Phase::Parse);
    assert!(side_effect
        .message
        .contains("top-level executable side effects"));

    let autoload_inc = dir.join("autoload.inc");
    let autoload_root = dir.join("autoload.php");
    fs::write(
        &autoload_inc,
        "<?php\nspl_autoload_register(function ($name) { require __DIR__ . '/' . $name . '.php'; });\nclass AutoloadSideEffect {}\n",
    )
    .unwrap();
    fs::write(
        &autoload_root,
        "<?php\nrequire __DIR__ . '/autoload.inc';\n",
    )
    .unwrap();
    let autoload = include_discovery_error(&autoload_root);
    assert_eq!(autoload.phase, Phase::Parse);
    assert!(autoload.message.contains("autoload registration"));

    let late_root = dir.join("late.php");
    fs::write(
        &late_root,
        "<?php\necho 'before';\nrequire __DIR__ . '/class.php';\n",
    )
    .unwrap();
    let late = include_discovery_error(&late_root);
    assert_eq!(late.phase, Phase::Parse);
    assert!(late
        .message
        .contains("before executable top-level statements"));
}

#[test]
fn native_include_require_expression_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1108/native_include_expression_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone1108/native_include_expression_boundary_emit_ir.cli"),
    )
    .expect("native include expression CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn include_discovery_error(path: &Path) -> php_compiler::error::Diagnostic {
    compilation_unit_with_literal_include_metadata(
        &fs::read_to_string(path).expect("read include discovery error fixture"),
        path,
        workspace_root(),
    )
    .unwrap_err()
}

fn compile_exe(root: &Path, output: &Path, label: &str) -> PathBuf {
    let compile = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root())
        .args([
            "compile",
            root.to_str().expect("root fixture path is UTF-8"),
            "--emit-exe",
            output.to_str().expect("output path is UTF-8"),
        ])
        .output()
        .unwrap_or_else(|error| panic!("compile {label}: {error}"));
    assert!(
        compile.status.success(),
        "{label} compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
    output.to_path_buf()
}

fn write_supported_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "class IncludedBase { public const BASE_TOKEN = 'base-const'; public static $count = 1; }\n",
            "class IncludedBox extends IncludedBase {\n",
            "    public const TOKEN = 'child-const';\n",
            "    public function label() { return 'child'; }\n",
            "    public static function mark() { return 'static'; }\n",
            "}\n",
        ),
    )
    .expect("write included class fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included.php';\n",
            "echo class_exists('IncludedBox') ? 'exists' : 'missing';\n",
            "echo '|';\n",
            "$box = new IncludedBox();\n",
            "echo $box->label();\n",
            "echo '|';\n",
            "echo IncludedBox::mark();\n",
            "echo '|';\n",
            "IncludedBox::$count = 42;\n",
            "echo IncludedBox::$count;\n",
            "echo '|';\n",
            "echo IncludedBox::TOKEN;\n",
            "echo '|';\n",
            "echo IncludedBox::BASE_TOKEN;\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root class fixture");
    root
}

fn write_supported_trait_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include trait discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included-trait.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "trait IncludedMetaTrait {\n",
            "    public function traitLabel() { return 'trait'; }\n",
            "}\n",
        ),
    )
    .expect("write included trait fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included-trait.php';\n",
            "class UsesIncludedMetaTrait { use IncludedMetaTrait; }\n",
            "echo class_exists('UsesIncludedMetaTrait') ? 'trait-meta' : 'missing';\n",
            "echo '|';\n",
            "$box = new UsesIncludedMetaTrait();\n",
            "echo $box->traitLabel();\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root trait fixture");
    root
}

fn write_supported_trait_constructor_include_fixture(dir: &Path) -> PathBuf {
    fs::create_dir_all(dir).expect("create include trait constructor discovery fixture dir");
    let root = dir.join("root.php");
    let included = dir.join("included-trait-constructor.php");
    fs::write(
        &included,
        concat!(
            "<?php\n",
            "trait IncludedConstructorTrait {\n",
            "    public function __construct($value) { $this->label = 'included-' . $value; }\n",
            "    public function traitLabel() { return $this->label; }\n",
            "}\n",
        ),
    )
    .expect("write included trait constructor fixture");
    fs::write(
        &root,
        concat!(
            "<?php\n",
            "require __DIR__ . '/included-trait-constructor.php';\n",
            "class UsesIncludedConstructorTrait {\n",
            "    public $label;\n",
            "    use IncludedConstructorTrait;\n",
            "}\n",
            "echo class_exists('UsesIncludedConstructorTrait') ? 'trait-ctor-meta' : 'missing';\n",
            "echo '|';\n",
            "$box = new UsesIncludedConstructorTrait('ctor');\n",
            "echo $box->traitLabel();\n",
            "echo \"\\n\";\n",
        ),
    )
    .expect("write root trait constructor fixture");
    root
}

fn include_discovery_fixture_dir(name: &str) -> PathBuf {
    let mut dir = workspace_root().join("target/include-class-discovery");
    dir.push(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create include discovery fixture dir");
    dir
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler has a workspace root")
        .to_path_buf()
}

fn has_cc() -> bool {
    Command::new("cc").arg("--version").output().is_ok()
}

fn main_body(source: &str) -> &str {
    source
        .split_once("int main(void)")
        .map(|(_, body)| body)
        .unwrap_or(source)
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}

fn c_byte_array_for_test(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| byte.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
