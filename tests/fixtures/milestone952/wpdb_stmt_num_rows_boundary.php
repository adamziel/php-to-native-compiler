<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function count_prepared_option_rows() {
        $stmt = $this->dbh;
        return mysqli_stmt_num_rows($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->count_prepared_option_rows();
