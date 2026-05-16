<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function bind_prepared_option_result() {
        $stmt = $this->dbh;
        $option_name = null;
        return mysqli_stmt_bind_result($stmt, $option_name);
    }
}

$wpdb = new wpdb();
$wpdb->bind_prepared_option_result();
