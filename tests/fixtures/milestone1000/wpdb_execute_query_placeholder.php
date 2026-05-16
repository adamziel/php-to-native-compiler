<?php
class wpdb {
    public $dbh;
    public $title;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, "localhost", "user", "pass", null, 3306, null, 0);
    }

    public function fetch_post($id) {
        $result = mysqli_execute_query($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array($id));
        $row = mysqli_fetch_assoc($result);
        $this->title = $row["post_title"];
    }
}

$wpdb = new wpdb();
$wpdb->fetch_post(1);
echo $wpdb->title;

