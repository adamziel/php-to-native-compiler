<?php
interface WP_Hook_Base {
    public function dispatch(string $hook): string;
    public function summarize($context);
    public function optional($value);
}

interface WP_Hook_Contract extends WP_Hook_Base {
    public function dispatch($hook, $priority = 10): string;
    public function summarize($context): string;
    public function optional($value, $fallback = null);
}

trait WP_Hook_Methods {
    public function dispatch($hook, $priority = 10): string {
        return $hook . ":" . $priority;
    }

    public function summarize($context): string {
        return "summary:" . $context;
    }

    public function optional($value, $fallback = null) {
        return $value . ":" . $fallback;
    }
}

class WP_Hook_Plugin implements WP_Hook_Contract {
    use WP_Hook_Methods;
}

$plugin = new WP_Hook_Plugin();
echo $plugin instanceof WP_Hook_Base ? "base\n" : "missing\n";
echo $plugin instanceof WP_Hook_Contract ? "contract\n" : "missing\n";
echo method_exists($plugin, "dispatch") ? "dispatch-method\n" : "missing\n";
echo method_exists($plugin, "summarize") ? "summary-method\n" : "missing\n";
echo $plugin->optional("value", "fallback");
