<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function has_prepared_option_more_results() {
        $stmt = $this->dbh;
        return mysqli_stmt_more_results($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->has_prepared_option_more_results();
