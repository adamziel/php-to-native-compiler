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

    public function prepared_delete($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, array($pattern));
        return mysqli_stmt_affected_rows($stmt);
    }

    public function execute_delete($query, $pattern) {
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
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'site-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'wild=' . $wpdb->prepared_delete(
    'DELETE FROM wp_options WHERE option_name LIKE ?',
    '_transient_%'
);

$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");

echo '|escape=' . $wpdb->execute_delete(
    "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!'",
    '!_transient!_%'
);
echo '|left=' . $wpdb->option_parts();
