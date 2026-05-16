<?php
class wpdb {
    public $dbh;
    public $defaults;
    public $updated;

    public function __construct() {
        $this->dbh = mysqli_init();
    }

    public function record_statement_attributes() {
        $stmt = mysqli_prepare($this->dbh, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
        $this->defaults = array(
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH),
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE),
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS),
        );
        mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH, 1);
        mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE, MYSQLI_CURSOR_TYPE_READ_ONLY);
        mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS, 4);
        $this->updated = array(
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH),
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE),
            mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS),
        );
    }
}

$wpdb = new wpdb();
$wpdb->record_statement_attributes();
echo $wpdb->defaults[0], ":", $wpdb->defaults[1], ":", $wpdb->defaults[2];
echo "|";
echo $wpdb->updated[0], ":", $wpdb->updated[1], ":", $wpdb->updated[2];
