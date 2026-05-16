<?php
class wpdb {
    public $dbh;
    public $last_id;
    public $last_title;
    public $fetched;
    public $after_fetch;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function fetch_seed_post_with_bound_statement() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_stmt_execute($stmt);
        mysqli_stmt_store_result($stmt);
        $id = null;
        $title = null;
        mysqli_stmt_bind_result($stmt, $id, $title);
        $this->fetched = mysqli_stmt_fetch($stmt);
        $this->last_id = $id;
        $this->last_title = $title;
        $this->after_fetch = mysqli_stmt_fetch($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->fetch_seed_post_with_bound_statement();
echo $wpdb->fetched ? "fetched" : "not-fetched";
echo "|";
echo $wpdb->last_id, ":", $wpdb->last_title;
echo "|";
echo $wpdb->after_fetch ? "again" : "done";
