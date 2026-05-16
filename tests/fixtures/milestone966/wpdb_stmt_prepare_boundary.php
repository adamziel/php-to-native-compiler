<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function prepared_option_prepare() {
        $stmt = $this->dbh;
        return mysqli_stmt_prepare($stmt, "SELECT option_value FROM wp_options WHERE option_name = ?");
    }
}

$wpdb = new wpdb();
$wpdb->prepared_option_prepare();
