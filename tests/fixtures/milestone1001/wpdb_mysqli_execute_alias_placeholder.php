<?php
class wpdb {
    public $dbh;
    public $title;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function fetch_post($id) {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
        mysqli_execute($stmt, array($id));
        $result = mysqli_stmt_get_result($stmt);
        $row = mysqli_fetch_assoc($result);
        $this->title = $row["post_title"];
    }
}

$wpdb = new wpdb();
$wpdb->fetch_post(1);
echo $wpdb->title;
