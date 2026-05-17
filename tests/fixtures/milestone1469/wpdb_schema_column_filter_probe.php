<?php
class wpdb {
    public $dbh;

    public function __construct() {
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function get_row($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_fetch_assoc($result);
    }

    public function get_var($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_num_rows($result);
    }
}

$wpdb = new wpdb();

$wpdb->query("CREATE TABLE wp_probe_options (option_id bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, autoload varchar(20) NOT NULL default 'yes', PRIMARY KEY  (option_id), UNIQUE KEY option_name (option_name)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");

$like = $wpdb->get_row("SHOW FULL COLUMNS FROM `wp_probe_options` LIKE 'option_name'");
echo $like['Field'];
echo ':';
echo $like['Type'];
echo ':';
echo $like['Collation'];
echo ':';
echo $like['Key'];

echo '|';

$where = $wpdb->get_row("SHOW COLUMNS FROM wp_probe_options WHERE Field = 'option_value'");
echo $where['Field'];
echo ':';
echo $where['Null'];
echo ':';
echo $where['Collation'];

echo '|';

$describe = $wpdb->get_row('DESCRIBE wp_probe_options autoload');
echo $describe['Field'];
echo ':';
echo $describe['Default'];

echo '|missing=';
echo $wpdb->get_var("SHOW FULL COLUMNS FROM wp_probe_options LIKE 'missing_column'");
