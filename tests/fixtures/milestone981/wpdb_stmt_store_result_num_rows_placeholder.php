<?php
class wpdb {
    public $dbh;
    public $num_rows;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function count_seed_posts() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_stmt_execute($stmt);
        mysqli_stmt_store_result($stmt);
        $this->num_rows = mysqli_stmt_num_rows($stmt);
        mysqli_stmt_free_result($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->count_seed_posts();
echo $wpdb->num_rows;
