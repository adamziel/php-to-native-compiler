<?php
class WP_Hook {
    public $callbacks = array();

    public function add_filter($hook_name, $callback, $priority, $accepted_args) {
        $priority_existed = isset($this->callbacks[$priority]);
        $this->callbacks[$priority][$hook_name] = array(
            'function' => $callback,
            'accepted_args' => (int) $accepted_args,
        );

        echo $priority_existed ? 'old' : 'new';
        echo '|';
        echo isset($this->callbacks[$priority][$hook_name]) ? 'registered' : 'missing';
    }
}

$hook = new WP_Hook();
$hook->add_filter('the_title', 'trim', 10, 1);
