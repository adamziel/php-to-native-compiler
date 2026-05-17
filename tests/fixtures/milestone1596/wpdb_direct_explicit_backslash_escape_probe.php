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

    public function get_col($query) {
        $result = mysqli_query($this->dbh, $query);
        $values = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $values[] = $row['option_name'];
        }
        return implode(',', $values);
    }

    public function delete($query) {
        mysqli_query($this->dbh, $query);
        return mysqli_affected_rows($this->dbh);
    }
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_mode_target', '200', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-mode-target', '200', 'no')");
$wpdb->query("SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");

echo 'implicit=' . $wpdb->get_col(
    "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ORDER BY option_name"
);
echo '|explicit=' . $wpdb->get_col(
    "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ESCAPE '\\\\' ORDER BY option_name"
);
echo '|expired=' . $wpdb->get_col(
    "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_timeout\\_%' ESCAPE '\\\\' AND option_value < 300 ORDER BY option_name"
);
echo '|delete=' . $wpdb->delete(
    "DELETE FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ESCAPE '\\\\'"
);
echo '|left=' . $wpdb->get_col('SELECT option_name FROM wp_options');
