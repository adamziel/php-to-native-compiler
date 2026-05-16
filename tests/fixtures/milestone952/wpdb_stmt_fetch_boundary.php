<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function fetch_prepared_option_row() {
        $stmt = $this->dbh;
        return mysqli_stmt_fetch($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->fetch_prepared_option_row();
