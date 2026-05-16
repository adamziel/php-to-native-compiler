<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function fetch_post_with_sparse_params() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
        return mysqli_stmt_execute($stmt, array(1 => 1));
    }
}

$wpdb = new wpdb();
$wpdb->fetch_post_with_sparse_params();

