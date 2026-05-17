<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function get_col_prepared($query, $pattern) {
        $result = mysqli_execute_query($this->dbh, $query, array($pattern));
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'];
        }
        return implode(',', $values);
    }

    public function delete_prepared($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern));
        return mysqli_stmt_affected_rows($stmt);
    }

    public function delete_execute($query, $pattern) {
        mysqli_execute_query($this->dbh, $query, array($pattern));
        return mysqli_affected_rows($this->dbh);
    }

    public function option_parts() {
        $result = mysqli_query($this->dbh, 'SELECT option_name, option_value FROM wp_options');
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'] . '=' . $row['option_value'];
        }
        return implode(',', $values);
    }
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'default=' . $wpdb->get_col_prepared(
    'SELECT option_name FROM wp_options WHERE option_name LIKE ?',
    "\\_transient\\_%"
);

$wpdb->query("SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");

echo '|mode=' . $wpdb->get_col_prepared(
    'SELECT option_name FROM wp_options WHERE option_name LIKE ?',
    "\\_transient\\_%"
);
echo '|explicit=' . $wpdb->get_col_prepared(
    "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '!'",
    '!_transient!_%'
);
echo '|delete-mode=' . $wpdb->delete_prepared(
    'DELETE FROM wp_options WHERE option_name LIKE ?',
    "\\_transient\\_%"
);
echo '|delete-explicit=' . $wpdb->delete_execute(
    "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!'",
    '!_transient!_%'
);
echo '|left=' . $wpdb->option_parts();
