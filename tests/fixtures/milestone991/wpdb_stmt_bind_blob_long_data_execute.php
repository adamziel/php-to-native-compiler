<?php
class wpdb {
    public $dbh;
    public $last_id;
    public $last_title;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function fetch_post_blob_id($id) {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
        $blob = "unused";
        mysqli_stmt_bind_param($stmt, "b", $blob);
        mysqli_stmt_send_long_data($stmt, 0, $id);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $row = mysqli_fetch_assoc($result);
        $this->last_id = $row["ID"];
        $this->last_title = $row["post_title"];
    }
}

$wpdb = new wpdb();
$wpdb->fetch_post_blob_id("1");
echo $wpdb->last_id, ":", $wpdb->last_title;
