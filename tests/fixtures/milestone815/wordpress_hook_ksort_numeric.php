<?php
class WP_Hook {
    public $callbacks = array();
    public $priorities = array();

    public function add_filter($hook_name, $callback, $priority, $accepted_args) {
        $priority_existed = isset($this->callbacks[$priority]);

        $this->callbacks[$priority][$hook_name] = array(
            'function' => $callback,
            'accepted_args' => (int) $accepted_args,
        );

        if (!$priority_existed && count($this->callbacks) > 1) {
            ksort($this->callbacks, SORT_NUMERIC);
        }

        $this->priorities = array_keys($this->callbacks);
    }
}

$hook = new WP_Hook();
$hook->add_filter('late', 'trim', 10, 1);
$hook->add_filter('early', 'trim', 2, 1);
echo $hook->priorities[0], '|', $hook->priorities[1];
