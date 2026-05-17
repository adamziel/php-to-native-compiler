<?php
trait HookTools {
    public function boot() {
        return "boot:" . get_class($this);
    }

    public function secret() {
        return "secret:" . get_class($this);
    }
}

class Plugin {
    use HookTools {
        boot as protected;
        secret as private;
    }

    public function callBoot() {
        return $this->boot();
    }

    public function callSecret() {
        return $this->secret();
    }
}

$plugin = new Plugin();
echo $plugin->callBoot(), "|";
echo $plugin->callSecret(), "|";
echo method_exists($plugin, "boot") ? "boot-exists" : "missing";
echo "|";
echo method_exists($plugin, "secret") ? "secret-exists" : "missing";
echo "|";
$methods = get_class_methods($plugin);
echo count($methods), ":";
echo in_array("boot", $methods) ? "boot" : "hidden";
echo ",";
echo in_array("secret", $methods) ? "secret" : "hidden";
echo ",";
echo in_array("callBoot", $methods) ? "callBoot" : "missing";
echo ",";
echo in_array("callSecret", $methods) ? "callSecret" : "missing";
