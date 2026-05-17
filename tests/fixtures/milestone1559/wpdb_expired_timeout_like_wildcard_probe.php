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

    public function affected_rows() {
        return mysqli_affected_rows($this->dbh);
    }

    public function prepared_timeout_delete($pattern, $threshold) {
        $stmt = mysqli_prepare($this->dbh, 'DELETE FROM wp_options WHERE option_name LIKE ? AND option_value < ?');
        mysqli_stmt_execute($stmt, array($pattern, $threshold));
        return mysqli_stmt_affected_rows($stmt);
    }

    public function execute_timeout_delete($pattern, $threshold) {
        mysqli_execute_query(
            $this->dbh,
            'DELETE FROM `wp_options` WHERE `option_name` LIKE ? AND `option_value` < ?',
            array($pattern, $threshold)
        );
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
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_fresh', '900', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

echo 'wild=';
echo $wpdb->query("DELETE FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300") ? $wpdb->affected_rows() : 'failed';

$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");

echo '|escape=' . $wpdb->prepared_timeout_delete('\\_transient\\_timeout\\_%', 300);
echo '|execute=' . $wpdb->execute_timeout_delete('_transient_timeout_%', '300');
echo '|left=' . $wpdb->option_parts();
