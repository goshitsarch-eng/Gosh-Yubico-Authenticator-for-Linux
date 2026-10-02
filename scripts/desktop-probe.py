#!/usr/bin/env python3
"""A fresh AT-SPI client per probe avoids retained WebKit accessibility objects."""
import json
import subprocess
import sys
import time
import gi
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi


def walk(node):
    yield node
    for index in range(node.get_child_count()):
        child = node.get_child_at_index(index)
        if child is not None:
            yield from walk(child)


request = json.loads(sys.stdin.read())
deadline = time.monotonic() + 3
while time.monotonic() < deadline:
    nodes = sorted(walk(Atspi.get_desktop(0)), key=lambda n: n.get_role_name() != "button")
    for node in nodes:
        text = node.get_name()
        if not text and node.get_role_name() == "paragraph":
            text = Atspi.Text.get_text(node, 0, -1)
        if text != request["name"]:
            continue
        result = {"role": node.get_role_name(), "name": text,
                  "enabled": node.get_state_set().contains(Atspi.StateType.ENABLED),
                  "checked": node.get_state_set().contains(Atspi.StateType.CHECKED)}
        if node.get_role_name() in ("entry", "password text"):
            result["value"] = Atspi.Text.get_text(node, 0, -1)
        operation = request.get("operation", "query")
        if operation == "activate":
            action = node.get_action_iface()
            if action is None or action.get_n_actions() == 0:
                continue
            if node.get_role_name() in ("button", "check box"):
                assert Atspi.Component.grab_focus(node)
                subprocess.run(["xdotool", "key", "space" if node.get_role_name() == "check box" else "Return"], check=True)
            else:
                assert action.do_action(0)
        elif operation == "text":
            # WebKit deliberately omits EditableText on password entries. Paste
            # through the real keyboard/clipboard, as a user would paste a URI.
            assert Atspi.Component.grab_focus(node)
            subprocess.run(["xdotool", "key", "ctrl+a"], check=True)
            subprocess.run(["xclip", "-selection", "clipboard"], input=request["value"], text=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
            subprocess.run(["xdotool", "key", "ctrl+v"], check=True)
        elif operation == "select":
            assert Atspi.Component.grab_focus(node)
            subprocess.run(["xdotool", "key", "space"], check=True)
            subprocess.run(["xdotool", "key", "Home"], check=True)
            for _ in range(request["index"]):
                subprocess.run(["xdotool", "key", "Down"], check=True)
            subprocess.run(["xdotool", "key", "Return", "Tab"], check=True)
        print(json.dumps(result))
        sys.exit(0)
    time.sleep(0.2)
sys.exit(3)
