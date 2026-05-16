<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function execute_option_lookup() {
        $stmt = $this->dbh;
        return mysqli_stmt_execute($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->execute_option_lookup();
