<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function reset_prepared_option_query() {
        $stmt = $this->dbh;
        return mysqli_stmt_reset($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->reset_prepared_option_query();
