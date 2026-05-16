<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function write_prepared_statement_attr() {
        $stmt = $this->dbh;
        return mysqli_stmt_attr_set($stmt, 1, 1);
    }
}

$wpdb = new wpdb();
$wpdb->write_prepared_statement_attr();
