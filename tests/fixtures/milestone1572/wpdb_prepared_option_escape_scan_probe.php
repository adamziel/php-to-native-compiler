<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_feed_mod', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-feed-mod', 'wildcard', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_feed_mod', 'site-payload', 'no')");

class wpdb_probe {
    public $dbh;

    public function get_col_parts($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $first = mysqli_fetch_assoc($result);
        $second = mysqli_fetch_assoc($result);
        return array(mysqli_num_rows($result), $first["option_name"], $second["option_name"]);
    }

    public function get_row_via_stmt($query, $params) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_execute($stmt, $params);
        $result = mysqli_stmt_get_result($stmt);
        $row = mysqli_fetch_assoc($result);
        return array(mysqli_num_fields($result), $row["option_id"], $row["option_name"], $row["option_value"], $row["autoload"]);
    }

    public function get_var($query, $params) {
        $result = mysqli_execute_query($this->dbh, $query, $params);
        $row = mysqli_fetch_assoc($result);
        return $row["option_value"];
    }
}

$wpdb = new wpdb_probe();
$wpdb->dbh = $handle;

$names = $wpdb->get_col_parts(
    "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '!' ORDER BY option_name",
    array("!_transient!_%")
);
echo "names=", $names[0], ":", $names[1], ",", $names[2];

$site_row = $wpdb->get_row_via_stmt(
    "SELECT * FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!' ORDER BY `option_name` ASC",
    array("!_site!_transient!_%")
);
echo "|full=", $site_row[0], ":", $site_row[1], ":", $site_row[2], ":", $site_row[3], ":", $site_row[4];

$value = $wpdb->get_var(
    "SELECT option_value FROM wp_options WHERE option_name LIKE ? ESCAPE '!'",
    array("!_transient!_update!_%")
);
echo "|value=", $value;
