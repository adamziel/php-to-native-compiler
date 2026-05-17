<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function delete_direct_prefix($pattern) {
        return mysqli_query(
            $this->dbh,
            "DELETE FROM " . $this->options . " WHERE option_name LIKE '" . $pattern . "'"
        );
    }

    public function delete_prepared_prefix($query, $pattern) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $pattern);
        $ok = mysqli_stmt_execute($stmt);
        return $ok ? mysqli_stmt_affected_rows($stmt) : -1;
    }

    public function get_results($query) {
        $result = mysqli_query($this->dbh, $query);
        $rows = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $rows[] = $row;
        }
        return $rows;
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function set_transient($name, $value, $timeout) {
    add_option_row('_transient_' . $name, $value, 'no');
    add_option_row('_transient_timeout_' . $name, $timeout, 'no');
}

function set_site_transient($name, $value, $timeout) {
    add_option_row('_site_transient_' . $name, $value, 'no');
    add_option_row('_site_transient_timeout_' . $name, $timeout, 'no');
}

function option_parts($query) {
    global $wpdb;
    $rows = $wpdb->get_results($query);
    $parts = array();
    foreach ($rows as $row) {
        $parts[] = $row['option_name'] . '=' . $row['option_value'];
    }
    return implode(',', $parts);
}

$wpdb = new wpdb('wp_');
add_option_row('siteurl', 'https://example.test', 'yes');
set_transient('feed_mod', 'cached-feed', '123456');
set_transient('theme_roots', 'cached-theme', '789000');
set_site_transient('update_core', 'core-payload', '456789');

echo 'before=' . option_parts("SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%'");
echo '|';
echo $wpdb->delete_direct_prefix('_transient_timeout_%') ? 'direct' : 'direct-failed';
echo ':' . mysqli_affected_rows($wpdb->dbh);
echo '|';
echo 'after-direct=' . option_parts("SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%'");
echo '|';
$site_deleted = $wpdb->delete_prepared_prefix(
    'DELETE FROM `' . $wpdb->options . '` WHERE `option_name` LIKE ?',
    '\_site_transient\_%'
);
echo 'prepared=' . $site_deleted . ':' . mysqli_affected_rows($wpdb->dbh);
echo '|';
mysqli_execute_query($wpdb->dbh, 'DELETE FROM wp_options WHERE `option_name` LIKE ?', array('_transient_%'));
echo 'execute=' . mysqli_affected_rows($wpdb->dbh);
echo '|';
echo 'left=' . option_parts('SELECT option_name, option_value FROM wp_options');
