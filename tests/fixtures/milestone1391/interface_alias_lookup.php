<?php
interface SourceContract {
    public function label();
}

echo interface_exists("SourceAlias", false) ? "pre-alias\n" : "pre-missing\n";
echo class_alias("SourceContract", "SourceAlias") ? "alias-ok\n" : "alias-fail\n";
echo interface_exists("sourcealias", false) ? "alias-interface\n" : "alias-missing\n";
echo class_exists("SourceAlias", false) ? "alias-class\n" : "alias-not-class\n";

require_once __DIR__ . "/interface_alias_plugin.inc";

$plugin = new AliasPlugin();
echo $plugin instanceof SourceContract ? "instanceof-source\n" : "missing-source\n";
echo $plugin instanceof SourceAlias ? "instanceof-alias\n" : "missing-alias\n";
echo is_a($plugin, "SourceAlias") ? "is-a-alias\n" : "not-alias\n";
echo is_subclass_of("AliasPlugin", "SourceAlias") ? "subclass-alias\n" : "not-subclass\n";
echo in_array("SourceAlias", get_declared_interfaces(), true) ? "alias-declared\n" : "alias-hidden\n";
echo class_alias("SourceAlias", "SecondAlias") ? "second-ok\n" : "second-fail\n";
echo interface_exists("SecondAlias", false) ? "second-interface" : "second-missing";
