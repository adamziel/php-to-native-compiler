<?php
class WP_Interactivity_API_Probe {
    public function current_context($tag_stack) {
        list($opening_tag_name, $directives_prefixes) = ! empty($tag_stack)
            ? end($tag_stack)
            : array(null, null);

        return $opening_tag_name;
    }
}

echo "after";
