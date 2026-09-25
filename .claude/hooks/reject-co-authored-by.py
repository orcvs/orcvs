#!/usr/bin/env python3
"""PreToolUse hook: deny a Bash `git commit` whose message carries a Co-Authored-By trailer.

Checks the command text (covers -m and heredoc messages) and any file named by
-F/--file. Only the trailer form counts, so a message that merely names the
trailer is allowed. Uses only the standard library so it runs where mise shims
are untrusted.
"""
import json
import re
import shlex
import sys

# The trailer key opens a message line, a quoted -m argument, or follows an
# escaped newline inside a quoted argument.
TRAILER = re.compile(
    r"(?:^|['\"]|\\n)[ \t]*co-authored-by[ \t]*:", re.IGNORECASE | re.MULTILINE
)
GIT_COMMIT = re.compile(r"\bgit\b[^\n;&|]*\bcommit\b")


def message_files(command):
    try:
        words = shlex.split(command, posix=True)
    except ValueError:
        return []
    files = []
    for i, word in enumerate(words):
        if word in ("-F", "--file") and i + 1 < len(words):
            files.append(words[i + 1])
        elif word.startswith("--file="):
            files.append(word.split("=", 1)[1])
        elif word.startswith("-F") and len(word) > 2:
            files.append(word[2:])
    return files


def main():
    payload = json.load(sys.stdin)
    command = payload.get("tool_input", {}).get("command", "")
    if not GIT_COMMIT.search(command):
        return 0
    found = bool(TRAILER.search(command))
    for path in message_files(command):
        if path == "-":
            continue
        try:
            with open(path, encoding="utf-8", errors="replace") as f:
                found = found or bool(TRAILER.search(f.read()))
        except OSError:
            pass
    if found:
        json.dump(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": "git commit messages in this repository must not contain a Co-Authored-By trailer. Remove it and commit again.",
                }
            },
            sys.stdout,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
