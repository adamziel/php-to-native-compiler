<?php
class wpdb {
    public $dbh;
    public $last_statement_sql;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
        $this->last_statement_sql = "";
    }

    public function bind_option_lookup($option_name) {
        $this->last_statement_sql = "SELECT option_value FROM wp_options WHERE option_name = ?";
        $stmt = $this->dbh;
        return mysqli_stmt_bind_param($stmt, "s", $option_name);
    }
}

$wpdb = new wpdb();
$wpdb->bind_option_lookup("home");
