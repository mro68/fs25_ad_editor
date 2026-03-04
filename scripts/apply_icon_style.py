import os
import glob
import re

DEFS = """<defs>
    <radialGradient id="ringGlow" cx="50%" cy="50%" r="50%" fx="50%" fy="50%">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="1" />
      <stop offset="50%" stop-color="#ffffff" stop-opacity="0" />
      <stop offset="100%" stop-color="#ffffff" stop-opacity="1" />
    </radialGradient>
  </defs>"""

def process_file(filepath):
    if "icon_add_node.svg" in filepath:
        return
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Inject defs if not present
    if "<defs>" not in content:
        content = content.replace(">", ">" + DEFS, 1) # put after first svg tag opening

    # Transform circles (nodes) to use the ringGlow gradient + remove old fills/strokes
    # Match basic circle
    def circle_replacer(match):
        attrs = match.group(1)
        # remove old fills and strokes
        attrs = re.sub(r'fill="[^"]+"', '', attrs)
        attrs = re.sub(r'stroke="[^"]+"', '', attrs)
        attrs = re.sub(r'stroke-width="[^"]+"', '', attrs)
        # Add the glow style
        return f'<circle {attrs} fill="url(#ringGlow)" stroke="none" />'
        
    content = re.sub(r'<circle([^>]+)>', circle_replacer, content)

    # Some lines / paths might need to emulate the "black stroke outline" 
    # The user manual path has `fill:#ffffff` and `stroke:#000000` with `stroke-width:0.4`.
    # Let's add that to basic lines/paths if they use white stroke? 
    # Actually, since lines are just strokes, adding a black outline to a line in SVG 
    # means duplicating it or using SVG filters. 
    # If the user did it by converting paths to fills: we can't easily auto-convert lines to stroked-fills.
    # What we CAN do easily is give paths a dark drop-shadow/glow or just leave lines.
    # But wait, looking at user's SVG, they just used standard lines for connections:
    # line: stroke="#ffffff", stroke-width="1"
    
    # We will adjust all lines: scale down stroke widths to make it more delicate like User's 1px stroke
    # User's edited line has stroke-width="1".
    content = re.sub(r'stroke-width="[45678]2?"', 'stroke-width="1.5"', content)
    
    # Paths with fill none and stroke white:
    
    # Write back
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for f in glob.glob('/home/mro/Share/repos/fs25_ad_editor/assets/newgemini/*.svg'):
    process_file(f)

print("Icons adapted!")
