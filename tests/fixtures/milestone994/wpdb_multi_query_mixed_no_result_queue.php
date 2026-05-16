<?php
class wpdb {
    public $dbh;
    public $title;
    public $done;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function set_charset_then_fetch_post() {
        mysqli_multi_query($this->dbh, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'; SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        mysqli_store_result($this->dbh);
        mysqli_next_result($this->dbh);
        $result = mysqli_store_result($this->dbh);
        $row = mysqli_fetch_assoc($result);
        $this->title = $row["post_title"];
        $this->done = !mysqli_more_results($this->dbh);
    }
}

$wpdb = new wpdb();
$wpdb->set_charset_then_fetch_post();
echo $wpdb->title, "|", $wpdb->done ? "done" : "more";
