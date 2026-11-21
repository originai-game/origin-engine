import sys
faces={"default":"InstrumentSans.ttf","cartoon":"Fredoka-Medium.ttf","fantasy":"Newsreader-Medium.ttf","scifi":"BarlowSemiCondensed-SemiBold.ttf"}
SIZES=[12,14,16,20,28,48]
LINES=["Settings · Quit to menu · Hamburgefonstiv","开始游戏 设置 第 3 关 · Level 3","0123456789 +1,240 XP 06:54 ×320"]
for p,f in faces.items():
    rows=""
    for l in LINES:
        for s in SIZES:
            rows+=f'<div class="r" style="height:{round(s*1.3)}px"><span class="l">{s}</span><span class="t" style="font-size:{s}px">{l}</span></div>\n'
    open(f"{p}.html","w").write(f'''<!doctype html><meta charset="utf-8"><style>
@font-face{{font-family:B;src:url(fonts/{f})}}
@font-face{{font-family:SC;src:url(fonts/NotoSansSC-Regular.ttf)}}
html,body{{margin:0;background:#141a22;width:1200px;height:620px;overflow:hidden}}
body{{padding:8px 12px;box-sizing:border-box;font-family:B,SC;color:#e8ecf2}}
.r{{display:flex;align-items:center}}.l{{font-size:10px;color:rgba(255,255,255,.5);width:28px;flex:none}}.t{{white-space:nowrap;line-height:normal}}
</style>{rows}''')
