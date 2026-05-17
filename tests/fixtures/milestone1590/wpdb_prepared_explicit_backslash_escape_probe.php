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

    public function get_col_prepared($query, $pattern, $threshold = null) {
        if ($threshold === null) {
            $result = mysqli_execute_query($this->dbh, $query, array($pattern));
        } else {
            $result = mysqli_execute_query($this->dbh, $query, array($pattern, $threshold));
        }
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

    public function option_names() {
        $result = mysqli_query($this->dbh, 'SELECT option_name FROM wp_options');
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'];
        }
        return implode(',', $values);
    }
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_mode_target', '200', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-mode-target', '200', 'no')");
$wpdb->query("SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");

echo 'implicit=' . $wpdb->get_col_prepared(
    'SELECT option_name FROM wp_options WHERE option_name LIKE ? ORDER BY option_name',
    "\\_transient\\_%"
);
echo '|explicit=' . $wpdb->get_col_prepared(
    "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\' ORDER BY option_name",
    "\\_transient\\_%"
);
echo '|expired=' . $wpdb->get_col_prepared(
    "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\' AND option_value < ? ORDER BY option_name",
    "\\_transient\\_timeout\\_%",
    '300'
);
echo '|delete=' . $wpdb->delete_prepared(
    "DELETE FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\'",
    "\\_transient\\_%"
);
echo '|left=' . $wpdb->option_names();
