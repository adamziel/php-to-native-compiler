<?php
echo (bool) null ? "true" : "false", "|";
echo (boolean) false ? "true" : "false", "|";
echo (bool) true ? "true" : "false", "\n";
echo (bool) 0 ? "true" : "false", "|";
echo (bool) 1 ? "true" : "false", "|";
echo (bool) 0.0 ? "true" : "false", "|";
echo (bool) -0.5 ? "true" : "false", "\n";
echo (bool) "" ? "true" : "false", "|";
echo (bool) "0" ? "true" : "false", "|";
echo (bool) "false" ? "true" : "false", "\n";
echo (bool) [] ? "true" : "false", "|";
echo (bool) [0] ? "true" : "false", "\n";
class Flag {}
echo (bool) new Flag() ? "true" : "false";
