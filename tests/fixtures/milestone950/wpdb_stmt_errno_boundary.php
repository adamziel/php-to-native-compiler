<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function record_statement_errno() {
        $stmt = $this->dbh;
        return mysqli_stmt_errno($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->record_statement_errno();
