<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function count_prepared_option_fields() {
        $stmt = $this->dbh;
        return mysqli_stmt_field_count($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->count_prepared_option_fields();
