<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function describe_prepared_option_result() {
        $stmt = $this->dbh;
        return mysqli_stmt_result_metadata($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->describe_prepared_option_result();
