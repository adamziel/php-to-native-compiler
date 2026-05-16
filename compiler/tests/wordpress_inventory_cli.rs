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
        "<?php\ndefine('WPINC', 'wp-includes');\nrequire ABSPATH . WPINC . '/load.php';\necho $table_prefix;\n",
    )
    .expect("write synthetic wp-settings.php");
    fs::write(
        wp_root.join("wp-blog-header.php"),
        "<?php\nrequire_once __DIR__ . '/wp-load.php';\necho 'front';\n",
    )
    .expect("write synthetic wp-blog-header.php");
    fs::write(
        wp_root.join("wp-load.php"),
        "<?php\nif (!defined('ABSPATH')) { define('ABSPATH', __DIR__ . '/'); }\nrequire_once ABSPATH . 'wp-config.php';\n",
    )
    .expect("write synthetic wp-load.php");
    fs::write(
        wp_root.join("wp-config.php"),
        "<?php\n$table_prefix = 'wp_';\nrequire_once ABSPATH . 'wp-settings.php';\n",
    )
    .expect("write synthetic wp-config.php");
    fs::write(
        wp_includes.join("version.php"),
        "<?php\n$wp_version = '6.9.4';\n",
    )
    .expect("write synthetic version.php");
    fs::write(
        wp_includes.join("load.php"),
        "<?php\nnamespace Synthetic\\WordPress;\nuse Synthetic\\Dependency;\nfunction bootstrap_label($value = \"ok\") { return $value; }\nbootstrap_label();\ninterface Hookable {}\ntrait RegistersHooks {}\nenum Mode { case Front; }\nclass BaseLoader {}\nclass Loader extends BaseLoader {}\ntry { $callback = function () { return 1; }; } catch (Exception $e) {}\n$arrow = fn ($value) => $value;\n",
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

#[test]
fn wordpress_inventory_front_controller_smoke_matches_fixture() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let wp_root = std::env::temp_dir().join(format!(
        "phpc-wordpress-front-controller-{}-{unique}",
        std::process::id()
    ));
    let wp_includes = wp_root.join("wp-includes");

    fs::create_dir_all(&wp_includes).expect("create synthetic wp-includes");
    fs::write(
        wp_root.join("wp-blog-header.php"),
        "<?php\nrequire_once __DIR__ . '/wp-load.php';\n",
    )
    .expect("write front controller");
    fs::write(
        wp_root.join("wp-load.php"),
        "<?php\nif (!defined('ABSPATH')) { define('ABSPATH', __DIR__ . '/'); }\nrequire_once ABSPATH . 'wp-config.php';\n",
    )
    .expect("write wp-load.php");
    fs::write(
        wp_root.join("wp-config.php"),
        "<?php\n$table_prefix = 'wp_';\nrequire_once ABSPATH . 'wp-settings.php';\n",
    )
    .expect("write wp-config.php");
    fs::write(
        wp_root.join("wp-settings.php"),
        "<?php\ndefine('WPINC', 'wp-includes');\nrequire ABSPATH . WPINC . '/load.php';\nrequire ABSPATH . WPINC . '/class-wpdb.php';\n$wpdb = new wpdb();\n$wpdb->set_charset('utf8mb4', 'utf8mb4_unicode_520_ci');\n",
    )
    .expect("write wp-settings.php");
    fs::write(wp_includes.join("load.php"), "<?php\n").expect("write load.php");
    fs::write(
        wp_includes.join("class-wpdb.php"),
        "<?php\nclass wpdb {\n    public $dbh;\n\n    public function __construct() {\n        $this->dbh = mysqli_init();\n        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);\n    }\n\n    public function set_charset($charset, $collate) {\n        $query = \"SET NAMES '\" . $charset . \"' COLLATE '\" . $collate . \"'\";\n        return mysqli_query($this->dbh, $query);\n    }\n}\n",
    )
    .expect("write class-wpdb.php");
    fs::write(
        wp_includes.join("version.php"),
        "<?php\n$wp_version = '6.9.4';\n",
    )
    .expect("write version.php");

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
        include_str!("../../tests/fixtures/compat/wordpress/front_controller_smoke.expected");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn wordpress_inventory_wpdb_option_bootstrap_smoke_matches_fixture() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let wp_root = std::env::temp_dir().join(format!(
        "phpc-wordpress-wpdb-options-{}-{unique}",
        std::process::id()
    ));
    let wp_includes = wp_root.join("wp-includes");

    fs::create_dir_all(&wp_includes).expect("create synthetic wp-includes");
    fs::write(
        wp_root.join("wp-blog-header.php"),
        "<?php\nrequire_once __DIR__ . '/wp-load.php';\n",
    )
    .expect("write front controller");
    fs::write(
        wp_root.join("wp-load.php"),
        "<?php\nif (!defined('ABSPATH')) { define('ABSPATH', __DIR__ . '/'); }\nrequire_once ABSPATH . 'wp-config.php';\n",
    )
    .expect("write wp-load.php");
    fs::write(
        wp_root.join("wp-config.php"),
        "<?php\n$table_prefix = 'wp_';\nrequire_once ABSPATH . 'wp-settings.php';\n",
    )
    .expect("write wp-config.php");
    fs::write(
        wp_root.join("wp-settings.php"),
        "<?php\ndefine('WPINC', 'wp-includes');\nrequire ABSPATH . WPINC . '/load.php';\nrequire ABSPATH . WPINC . '/class-wpdb.php';\n$wpdb = new wpdb();\necho $wpdb->bootstrap_option();\n",
    )
    .expect("write wp-settings.php");
    fs::write(wp_includes.join("load.php"), "<?php\n").expect("write load.php");
    fs::write(
        wp_includes.join("class-wpdb.php"),
        "<?php\nclass wpdb {\n    public $dbh;\n\n    public function __construct() {\n        $this->dbh = mysqli_init();\n        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);\n    }\n\n    public function bootstrap_option() {\n        mysqli_query($this->dbh, \"INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'db-ok', 'yes')\");\n        $result = mysqli_query($this->dbh, \"SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1\");\n        $row = mysqli_fetch_assoc($result);\n        return $row['option_value'];\n    }\n}\n",
    )
    .expect("write class-wpdb.php");
    fs::write(
        wp_includes.join("version.php"),
        "<?php\n$wp_version = '6.9.4';\n",
    )
    .expect("write version.php");

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
        include_str!("../../tests/fixtures/compat/wordpress/wpdb_option_bootstrap.expected");
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

#[test]
fn wordpress_inventory_reports_probe_timeouts() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler crate has workspace parent");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let wp_root = std::env::temp_dir().join(format!(
        "phpc-wordpress-inventory-timeout-{}-{unique}",
        std::process::id()
    ));
    let wp_includes = wp_root.join("wp-includes");

    fs::create_dir_all(&wp_includes).expect("create synthetic wp-includes");
    fs::write(wp_root.join("wp-settings.php"), "<?php\nwhile (true) {}\n")
        .expect("write looping synthetic wp-settings.php");
    fs::write(
        wp_root.join("wp-blog-header.php"),
        "<?php\nrequire_once __DIR__ . '/wp-settings.php';\n",
    )
    .expect("write looping synthetic wp-blog-header.php");
    fs::write(
        wp_includes.join("version.php"),
        "<?php\n$wp_version = '6.9.4';\n",
    )
    .expect("write synthetic version.php");

    let output = Command::new("sh")
        .arg(repo_root.join("tools/wordpress-inventory.sh"))
        .arg("--normalize")
        .arg(&wp_root)
        .env("PHPC_BIN", env!("CARGO_BIN_EXE_phpc"))
        .env("WORDPRESS_PROBE_TIMEOUT", "1s")
        .current_dir(repo_root)
        .output()
        .expect("run wordpress inventory with timeout");

    let _ = fs::remove_dir_all(&wp_root);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("direct_settings_probe:\n"));
    assert!(stdout.contains("bootstrap_shim_probe:\n"));
    assert!(stdout.contains("front_controller_probe:\n"));
    assert_eq!(stdout.matches("  timeout: 1s\n").count(), 3);
    assert_eq!(stdout.matches("  timed_out: yes\n").count(), 3);
    assert_eq!(stdout.matches("  exit: 124\n").count(), 3);
}
