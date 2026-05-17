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
        boot as protected protected_boot;
        secret as private private_secret;
    }

    public function callProtected() {
        return $this->protected_boot();
    }

    public function callPrivate() {
        return $this->private_secret();
    }
}

$plugin = new Plugin();
echo $plugin->boot(), "|";
echo $plugin->secret(), "|";
echo $plugin->callProtected(), "|";
echo $plugin->callPrivate(), "|";
echo method_exists($plugin, "protected_boot") ? "protected-exists" : "missing";
echo "|";
echo method_exists($plugin, "private_secret") ? "private-exists" : "missing";
echo "|";
$methods = get_class_methods($plugin);
echo count($methods), ":";
echo in_array("boot", $methods) ? "boot" : "missing";
echo ",";
echo in_array("secret", $methods) ? "secret" : "missing";
echo ",";
echo in_array("callProtected", $methods) ? "callProtected" : "missing";
echo ",";
echo in_array("callPrivate", $methods) ? "callPrivate" : "missing";
