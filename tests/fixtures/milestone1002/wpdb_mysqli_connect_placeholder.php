<?php
class wpdb {
    public $dbh;
    public $title;

    public function db_connect() {
        $this->dbh = mysqli_connect("localhost", "user", "password", "wordpress");
    }

    public function fetch_post() {
        $result = mysqli_query($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        $row = mysqli_fetch_assoc($result);
        $this->title = $row["post_title"];
    }
}

$wpdb = new wpdb();
$wpdb->db_connect();
$wpdb->fetch_post();
echo get_class($wpdb->dbh), "|", $wpdb->title;
