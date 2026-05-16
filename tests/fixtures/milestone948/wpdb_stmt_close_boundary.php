<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function close_prepared_statement() {
        $stmt = $this->dbh;
        return mysqli_stmt_close($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->close_prepared_statement();
