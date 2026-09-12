# 生成 NSIS 安装器品牌素材：sidebarImage(164x314) 与 headerImage(150x57)
# 复用应用内 Kiroku 书形图标（viewBox 0 0 30 36），PIL 绘制 4x 超采样后降采样抗锯齿
from PIL import Image, ImageDraw, ImageFont

BRAND = (0, 113, 227)      # #0071e3
BG = (245, 245, 247)       # #f5f5f7
INK = (29, 29, 31)         # #1d1d1f
SUB = (134, 134, 139)      # gray


def draw_mark(d: ImageDraw.ImageDraw, ox: float, oy: float, s: float, color=BRAND):
    """以 s 为缩放系数绘制 Kiroku 书形图标，原 viewBox 30x36。"""
    def p(x, y):
        return (ox + x * s, oy + y * s)
    w = max(1, round(1.4 * s))
    # 书体外框 M7 5h17v25H7z
    d.line([p(7, 5), p(24, 5), p(24, 30), p(7, 30), p(7, 5)], fill=color, width=w)
    # M3 9v25h17
    d.line([p(3, 9), p(3, 34), p(20, 34)], fill=color, width=w)
    # M11 5v25
    d.line([p(11, 5), p(11, 30)], fill=color, width=w)
    # 书页横线
    d.line([p(15, 11), p(20, 11)], fill=color, width=w)
    d.line([p(15, 15), p(20, 15)], fill=color, width=w)
    # 书签 M19 3v6l2-1.4L23 9V3z
    d.polygon([p(19, 3), p(19, 9), p(21, 7.6), p(23, 9), p(23, 3)], fill=color)


def sidebar():
    SS = 4
    W, H = 164 * SS, 314 * SS
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    # 图标居中偏上
    s = 2.4 * SS
    mw, mh = 30 * s, 36 * s
    draw_mark(d, (W - mw) / 2, H * 0.30 - mh / 2, s)
    # 名称
    try:
        font = ImageFont.truetype("segoeuib.ttf", 30 * SS)
    except OSError:
        font = ImageFont.load_default()
    try:
        sub = ImageFont.truetype("msyh.ttc", 13 * SS)
    except OSError:
        try:
            sub = ImageFont.truetype("segoeui.ttf", 13 * SS)
        except OSError:
            sub = ImageFont.load_default()
    name = "Kiroku"
    bbox = d.textbbox((0, 0), name, font=font)
    d.text(((W - (bbox[2] - bbox[0])) / 2, H * 0.30 + mh / 2 + 14 * SS), name, font=font, fill=INK)
    tag = "个人番剧收藏"
    b2 = d.textbbox((0, 0), tag, font=sub)
    d.text(((W - (b2[2] - b2[0])) / 2, H * 0.30 + mh / 2 + 56 * SS), tag, font=sub, fill=SUB)
    img = img.resize((164, 314), Image.LANCZOS)
    img.save("src-tauri/installer/sidebar.bmp")


def header():
    SS = 4
    W, H = 150 * SS, 57 * SS
    img = Image.new("RGB", (W, H), (255, 255, 255))
    d = ImageDraw.Draw(img)
    s = 1.0 * SS
    mw = 30 * s
    # 小图标靠右
    draw_mark(d, W - mw - 14 * SS, (H - 36 * s) / 2, s)
    try:
        font = ImageFont.truetype("segoeuib.ttf", 17 * SS)
    except OSError:
        font = ImageFont.load_default()
    name = "Kiroku"
    bbox = d.textbbox((0, 0), name, font=font)
    d.text((W - mw - 20 * SS - (bbox[2] - bbox[0]), (H - (bbox[3] - bbox[1])) / 2 - 2 * SS), name, font=font, fill=INK)
    img = img.resize((150, 57), Image.LANCZOS)
    img.save("src-tauri/installer/header.bmp")


if __name__ == "__main__":
    import os
    os.makedirs("src-tauri/installer", exist_ok=True)
    sidebar()
    header()
    print("done")
