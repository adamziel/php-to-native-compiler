<?php
class wpdb {
    public $dbh;
    public $last_id;
    public $last_title;
    public $first_fetch;
    public $second_fetch;
    public $prebuffered_rows;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function fetch_seed_post_without_store_result() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
        mysqli_stmt_execute($stmt, array("1"));
        $this->prebuffered_rows = mysqli_stmt_num_rows($stmt);
        $id = null;
        $title = null;
        mysqli_stmt_bind_result($stmt, $id, $title);
        $this->first_fetch = mysqli_stmt_fetch($stmt);
        $this->last_id = $id;
        $this->last_title = $title;
        $this->second_fetch = mysqli_stmt_fetch($stmt);
    }
}

$wpdb = new wpdb();
$wpdb->fetch_seed_post_without_store_result();
echo $wpdb->prebuffered_rows;
echo "|";
echo $wpdb->first_fetch ? "fetched" : "not-fetched";
echo "|";
echo $wpdb->last_id, ":", $wpdb->last_title;
echo "|";
echo $wpdb->second_fetch === null ? "done" : "again";
