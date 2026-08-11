import json
import os
import re

from build_sample_dataset import sample_games_json

# Read raw python template script
with open('/Users/hdqb/workspaces/xiangqi-rim/scratch/build_ultimate_inspector_v5.py', 'r', encoding='utf-8') as f:
    code = f.read()

# Replace all inline regex literals in JS with safe RegExp constructors so Python string escaping never breaks regex across lines!
js_regex_replacements = [
    (r"thoughtText\.match\(/\\\[\\d\+\/32\\\]\[\^\\n\]\+\(\\n\(\?!\\\[\\d\+\/32\\\]\)\[\^\\n\]\+\)\*/g\)",
     "thoughtText.match(new RegExp('\\\\[\\\\d+/32\\\\][^\\\\n]+(?:\\\\n(?!\\\\[\\\\d+/32\\\\])[^\\\\n]+)*', 'g'))"),
    
    (r"thoughtText\.match\(/\\\[25\/32\\\]\[\^\\n\]\+\(\\n\(\?!\\\[26\/32\\\]\)\[\^\\n\]\+\)\*/\)",
     "thoughtText.match(new RegExp('\\\\[25/32\\\\][^\\\\n]+(?:\\\\n(?!\\\\[26/32\\\\])[^\\\\n]+)*'))"),
    
    (r"userMsg\.content\.match\(/FEN:\\s\*\\(\[\^\\n\]\+\\)\/\)",
     "userMsg.content.match(new RegExp('FEN:\\\\s*([^\\\\n]+)'))"),
    
    (r"headerLine\.match\(/\^\\\[\(\\d\+\)\/32\\\]\\s\*\\(\[\^\\n:\]\+\\):\\?\(\\.\*\)\/\)",
     "headerLine.match(new RegExp('^\\\\[(\\\\d+)/32\\\\]\\\\s*([^\\\\n:]+):?(.*)'))"),

    (r"line\.match\(/\\\+\?\\s\*Ứng viên\\s\*\\d\+:\\s\*\\\(\[a-i\]\[0-9\]\[a-i\]\[0-9\]\\\)\?\\s\*—\?\\s\*\\\(\[\^★\\n\(\]\+\\\)\?\\(\\\(\[\\^\\)\]\+\\\)\\)\?\/\)",
     "line.match(new RegExp('\\\\+?\\\\s*Ứng viên\\\\s*\\\\d+:\\\\s*([a-i][0-9][a-i][0-9])?\\\\s*—?\\\\s*([^★\\\\n(]+)?(?:\\\\(([^)]+)\\\\))?'))")
]

# Let's inspect raw JS script in v5 and replace regex literals carefully
fixed_code = code
for old_pat, new_str in js_regex_replacements:
    fixed_code = re.sub(old_pat, new_str, fixed_code)

with open('/Users/hdqb/workspaces/xiangqi-rim/scratch/build_ultimate_inspector_v5_fix.py', 'w', encoding='utf-8') as f:
    f.write(fixed_code)

print("✅ Fixer script updated!")
