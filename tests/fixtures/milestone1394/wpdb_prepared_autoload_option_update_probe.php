<?php
class wpdb {
    public $dbh;
    public $options;

    public function __construct($prefix) {
        $this->options = $prefix . 'options';
        $this->dbh = mysqli_init();
        mysqli_real_connect($this->dbh, 'localhost', 'user', 'pass', null, 3306, null, 0);
    }

    public function query($query) {
        return mysqli_query($this->dbh, $query);
    }

    public function get_var_prepared($query, $name) {
        $stmt = mysqli_prepare($this->dbh, $query);
        mysqli_stmt_bind_param($stmt, 's', $name);
        mysqli_stmt_execute($stmt);
        $result = mysqli_stmt_get_result($stmt);
        $row = mysqli_fetch_assoc($result);
        if ($row) {
            return $row['autoload'];
        }
        return false;
    }

    public function update_option_row($name, $value, $autoload) {
        $stmt = mysqli_prepare(
            $this->dbh,
            'UPDATE `wp_options` SET `option_value` = ?, `autoload` = ? WHERE `option_name` = ?'
        );
        mysqli_stmt_bind_param($stmt, 'sss', $value, $autoload, $name);
        mysqli_stmt_execute($stmt);
        return mysqli_stmt_affected_rows($stmt);
    }

    public function get_row($query) {
        $result = mysqli_query($this->dbh, $query);
        return mysqli_fetch_assoc($result);
    }
}

function add_option_row($name, $value, $autoload) {
    global $wpdb;
    return $wpdb->query(
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('" .
        $name . "', '" . $value . "', '" . $autoload . "')"
    );
}

function update_option_probe($option, $value, $autoload = null) {
    global $wpdb;
    if ($autoload === null) {
        $raw = $wpdb->get_var_prepared(
            'SELECT `autoload` FROM `' . $wpdb->options . '` WHERE `option_name` = ? LIMIT 1',
            $option
        );
        if ($raw === 'auto-on') {
            $autoload = 'auto-off';
        } else {
            $autoload = $raw;
        }
    }
    if ($autoload === false) {
        return false;
    }
    return $wpdb->update_option_row($option, $value, $autoload) === 1;
}

$wpdb = new wpdb('wp_');
add_option_row('blogdescription', 'old-db', 'auto-on');
add_option_row('siteurl', 'https://example.test', 'yes');

echo 'raw=' . $wpdb->get_var_prepared(
    'SELECT autoload FROM wp_options WHERE option_name = ? LIMIT 1',
    'blogdescription'
);

echo '|';
echo update_option_probe('blogdescription', 'fresh-db') ? 'updated' : 'failed';

echo '|';
$row = $wpdb->get_row("SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
echo $row['option_value'] . ':' . $row['autoload'];

echo '|';
$missing = $wpdb->get_var_prepared(
    'SELECT autoload FROM wp_options WHERE option_name = ? LIMIT 1',
    'missing'
);
echo $missing === false ? 'missing' : $missing;
