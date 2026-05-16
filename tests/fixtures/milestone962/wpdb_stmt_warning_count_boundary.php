<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function prepared_option_warning_count() {
        $stmt = $this->dbh;
        return mysqli_stmt_warning_count($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->prepared_option_warning_count();
