<?php
class wpdb {
    public $dbh;
    public $last_id;
    public $last_title;
    public $done;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function fetch_post_with_multi_query() {
        mysqli_multi_query($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        $result = mysqli_store_result($this->dbh);
        $row = mysqli_fetch_assoc($result);
        $this->last_id = $row["ID"];
        $this->last_title = $row["post_title"];
        $this->done = !mysqli_more_results($this->dbh);
    }
}

$wpdb = new wpdb();
$wpdb->fetch_post_with_multi_query();
echo $wpdb->last_id, ":", $wpdb->last_title, "|", $wpdb->done ? "done" : "more";
