use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn wordpress_inventory_normalized_synthetic_snapshot_matches_fixture() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let wp_root = std::env::temp_dir().join(format!(
        "phpc-wordpress-inventory-{}-{unique}",
        std::process::id()
    ));
    let wp_includes = wp_root.join("wp-includes");

    fs::create_dir_all(&wp_includes).expect("create synthetic wp-includes");
    fs::write(
        wp_root.join("wp-settings.php"),
        "<?php\ndefine('WPINC', 'wp-includes');\nrequire ABSPATH . WPINC . '/load.php';\n",
    )
    .expect("write synthetic wp-settings.php");
    fs::write(
        wp_includes.join("version.php"),
        "<?php\n$wp_version = '6.9.4';\n",
    )
    .expect("write synthetic version.php");
    fs::write(
        wp_includes.join("load.php"),
        "<?php\nnamespace Synthetic\\WordPress;\nuse Synthetic\\Dependency;\ninterface Hookable {}\ntrait RegistersHooks {}\nenum Mode { case Front; }\nclass Loader extends BaseLoader {}\ntry { $callback = function () { return 1; }; } catch (Exception $e) {}\n$arrow = fn ($value) => $value;\n",
    )
    .expect("write synthetic load.php");

    let output = Command::new("sh")
        .arg(repo_root.join("tools/wordpress-inventory.sh"))
        .arg("--normalize")
        .arg(&wp_root)
        .env("PHPC_BIN", env!("CARGO_BIN_EXE_phpc"))
        .current_dir(repo_root)
        .output()
        .expect("run wordpress inventory");

    let _ = fs::remove_dir_all(&wp_root);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");

    let expected =
        include_str!("../../tests/fixtures/compat/wordpress/synthetic_inventory.expected");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}
