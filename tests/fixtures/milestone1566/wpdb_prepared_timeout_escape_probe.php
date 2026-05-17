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

    public function escaped_timeout_names($pattern, $threshold) {
        $result = mysqli_execute_query(
            $this->dbh,
            "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '!' AND option_value < ? ORDER BY option_name",
            array($pattern, $threshold)
        );
        $names = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $names[] = $row['option_name'];
        }
        return mysqli_num_rows($result) . ':' . implode(',', $names);
    }

    public function delete_timeout_with_stmt($pattern, $threshold) {
        $stmt = mysqli_prepare(
            $this->dbh,
            "DELETE FROM wp_options WHERE option_name LIKE ? ESCAPE '!' AND option_value < ?"
        );
        mysqli_stmt_execute($stmt, array($pattern, $threshold));
        return mysqli_stmt_affected_rows($stmt);
    }

    public function delete_timeout_with_execute_query($pattern, $threshold) {
        mysqli_execute_query(
            $this->dbh,
            "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!' AND `option_value` < ?",
            array($pattern, $threshold)
        );
        return mysqli_affected_rows($this->dbh);
    }

    public function remaining_timeout_like_names() {
        $result = mysqli_query(
            $this->dbh,
            "SELECT option_name FROM wp_options WHERE option_name LIKE '%transient%timeout%' ORDER BY option_name"
        );
        $names = array();
        while ($row = mysqli_fetch_assoc($result)) {
            $names[] = $row['option_name'];
        }
        return implode(',', $names);
    }
}

$wpdb = new wpdb();
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_timeout_feed_mod', '120', 'no')");
$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_fresh', '900', 'no')");

echo 'select=' . $wpdb->escaped_timeout_names('!_transient!_timeout!_%', '300');
echo '|stmt=' . $wpdb->delete_timeout_with_stmt('!_transient!_timeout!_%', 300);

$wpdb->query("INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");

echo '|execute=' . $wpdb->delete_timeout_with_execute_query('!_transient!_timeout!_%', '300');
echo '|left=' . $wpdb->remaining_timeout_like_names();
