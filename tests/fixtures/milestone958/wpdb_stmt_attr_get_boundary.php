<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function read_prepared_statement_attr() {
        $stmt = $this->dbh;
        return mysqli_stmt_attr_get($stmt, 1);
    }
}

$wpdb = new wpdb();
$wpdb->read_prepared_statement_attr();
