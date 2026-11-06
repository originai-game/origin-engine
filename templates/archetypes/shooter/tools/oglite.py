"""A tiny, standard-library-only sprite kit for Origin Engine archetypes.

It draws antialiased vector shapes into RGBA images (4x4 supersampling),
writes PNGs, packs sprites into one glTF binary (.glb) of unit quads, one
mesh and one emissive, alpha-masked material per sprite, and writes 16-bit
WAV sounds. Everything is a pure function of its inputs, so the assets it
makes are byte-for-byte reproducible.

A sprite quad lies in the xy plane facing +z, 1 x 1 units, centred on the
origin, with uv (0,0) at the image's top left. Its material is black base
colour (alpha from the image, for the mask), roughness 1, and the image as
the emissive texture: with no light, no environment and tone mapping off,
the frame shows the image's colours exactly.
"""
import json, math, struct, wave, zlib

SS = 4  # supersamples per axis


def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


class Canvas:
    """An RGBA canvas; shapes are painted in order with coverage antialiasing.
    Coordinates are in pixels, origin top left, y down."""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.px = [[0.0, 0.0, 0.0, 0.0] for _ in range(w * h)]  # premultiplied

    def _paint(self, inside, bbox, colour, alpha=1.0):
        r, g, b = (c / 255.0 for c in hex_rgb(colour))
        x0, y0, x1, y1 = bbox
        x0 = max(0, int(math.floor(x0))); y0 = max(0, int(math.floor(y0)))
        x1 = min(self.w, int(math.ceil(x1))); y1 = min(self.h, int(math.ceil(y1)))
        n = SS * SS
        for y in range(y0, y1):
            for x in range(x0, x1):
                hits = 0
                for sy in range(SS):
                    py = y + (sy + 0.5) / SS
                    for sx in range(SS):
                        if inside(x + (sx + 0.5) / SS, py):
                            hits += 1
                if hits == 0:
                    continue
                a = alpha * hits / n
                p = self.px[y * self.w + x]
                p[0] = r * a + p[0] * (1 - a)
                p[1] = g * a + p[1] * (1 - a)
                p[2] = b * a + p[2] * (1 - a)
                p[3] = a + p[3] * (1 - a)

    def rect(self, x, y, w, h, colour, radius=0.0, alpha=1.0):
        rad = min(radius, w / 2, h / 2)

        def inside(px, py):
            if px < x or px > x + w or py < y or py > y + h:
                return False
            if rad <= 0:
                return True
            cx = min(max(px, x + rad), x + w - rad)
            cy = min(max(py, y + rad), y + h - rad)
            return (px - cx) ** 2 + (py - cy) ** 2 <= rad * rad

        self._paint(inside, (x, y, x + w, y + h), colour, alpha)

    def circle(self, cx, cy, r, colour, alpha=1.0):
        self.ellipse(cx, cy, r, r, colour, alpha)

    def ellipse(self, cx, cy, rx, ry, colour, alpha=1.0):
        def inside(px, py):
            return ((px - cx) / rx) ** 2 + ((py - cy) / ry) ** 2 <= 1.0

        self._paint(inside, (cx - rx, cy - ry, cx + rx, cy + ry), colour, alpha)

    def ring(self, cx, cy, r, width, colour, alpha=1.0):
        def inside(px, py):
            d = math.hypot(px - cx, py - cy)
            return r - width <= d <= r

        self._paint(inside, (cx - r, cy - r, cx + r, cy + r), colour, alpha)

    def poly(self, pts, colour, alpha=1.0):
        xs = [p[0] for p in pts]; ys = [p[1] for p in pts]

        def inside(px, py):
            c = False
            j = len(pts) - 1
            for i in range(len(pts)):
                xi, yi = pts[i]; xj, yj = pts[j]
                if (yi > py) != (yj > py) and px < (xj - xi) * (py - yi) / (yj - yi) + xi:
                    c = not c
                j = i
            return c

        self._paint(inside, (min(xs), min(ys), max(xs), max(ys)), colour, alpha)

    def line(self, x0, y0, x1, y1, width, colour, alpha=1.0):
        dx, dy = x1 - x0, y1 - y0
        ln = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / ln * width / 2, dx / ln * width / 2
        self.poly([(x0 + nx, y0 + ny), (x1 + nx, y1 + ny), (x1 - nx, y1 - ny), (x0 - nx, y0 - ny)], colour, alpha)
        self.circle(x0, y0, width / 2, colour, alpha)
        self.circle(x1, y1, width / 2, colour, alpha)

    def rgba(self):
        """Straight-alpha RGBA8 rows. Transparent pixels take the colour of
        the nearest painted pixel (a few dilation passes), so filtering and
        mips never pull a dark fringe into the edge."""
        w, h = self.w, self.h
        col = [None] * (w * h)
        for i, p in enumerate(self.px):
            if p[3] > 0:
                col[i] = (p[0] / p[3], p[1] / p[3], p[2] / p[3])
        for _ in range(8):
            nxt = list(col)
            changed = False
            for y in range(h):
                for x in range(w):
                    i = y * w + x
                    if col[i] is not None:
                        continue
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        xx, yy = x + dx, y + dy
                        if 0 <= xx < w and 0 <= yy < h and col[yy * w + xx] is not None:
                            nxt[i] = col[yy * w + xx]
                            changed = True
                            break
            col = nxt
            if not changed:
                break
        rows = []
        for y in range(h):
            row = bytearray()
            for x in range(w):
                i = y * w + x
                c = col[i] or (0.0, 0.0, 0.0)
                a = self.px[i][3]
                row += bytes([round(min(1, max(0, c[0])) * 255), round(min(1, max(0, c[1])) * 255),
                              round(min(1, max(0, c[2])) * 255), round(min(1, max(0, a)) * 255)])
            rows.append(bytes(row))
        return rows


def png_bytes(w, h, rows):
    raw = b"".join(b"\x00" + r for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def write_glb(path, sprites):
    """sprites: list of (name, png bytes, masked). One quad mesh per sprite,
    in list order: GltfInfo.geometries[i] and materials[i] are sprite i."""
    bin_ = bytearray()

    def add(data, align=4):
        while len(bin_) % align:
            bin_.append(0)
        off = len(bin_)
        bin_.extend(data)
        return off, len(data)

    pos = struct.pack("<12f", -0.5, -0.5, 0, 0.5, -0.5, 0, 0.5, 0.5, 0, -0.5, 0.5, 0)
    nrm = struct.pack("<12f", *([0, 0, 1] * 4))
    uv = struct.pack("<8f", 0, 1, 1, 1, 1, 0, 0, 0)
    idx = struct.pack("<6H", 0, 1, 2, 0, 2, 3)
    views, accessors = [], []
    for data, target in ((pos, 34962), (nrm, 34962), (uv, 34962), (idx, 34963)):
        o, n = add(data)
        views.append({"buffer": 0, "byteOffset": o, "byteLength": n, "target": target})
    accessors = [
        {"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [-0.5, -0.5, 0], "max": [0.5, 0.5, 0]},
        {"bufferView": 1, "componentType": 5126, "count": 4, "type": "VEC3"},
        {"bufferView": 2, "componentType": 5126, "count": 4, "type": "VEC2"},
        {"bufferView": 3, "componentType": 5123, "count": 6, "type": "SCALAR"},
    ]
    images, textures, materials, meshes, nodes = [], [], [], [], []
    for i, (name, data, masked) in enumerate(sprites):
        o, n = add(data)
        views.append({"buffer": 0, "byteOffset": o, "byteLength": n})
        images.append({"bufferView": len(views) - 1, "mimeType": "image/png", "name": name})
        textures.append({"source": i, "sampler": 0})
        m = {
            "name": name,
            "pbrMetallicRoughness": {"baseColorFactor": [0, 0, 0, 1], "baseColorTexture": {"index": i},
                                     "metallicFactor": 0, "roughnessFactor": 1},
            "emissiveTexture": {"index": i},
            "emissiveFactor": [1, 1, 1],
        }
        if masked:
            m["alphaMode"] = "MASK"
            m["alphaCutoff"] = 0.5
        materials.append(m)
        meshes.append({"name": name, "primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2},
                                                     "indices": 3, "material": i}]})
        nodes.append({"name": name, "mesh": i, "translation": [i * 1.2, 0, 0]})
    while len(bin_) % 4:
        bin_.append(0)
    doc = {
        "asset": {"version": "2.0", "generator": "oglite.py"},
        "scene": 0, "scenes": [{"nodes": list(range(len(nodes)))}], "nodes": nodes, "meshes": meshes,
        "materials": materials, "textures": textures, "images": images,
        "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 33071, "wrapT": 33071}],
        "accessors": accessors, "bufferViews": views, "buffers": [{"byteLength": len(bin_)}],
    }
    js = json.dumps(doc, separators=(",", ":")).encode()
    while len(js) % 4:
        js += b" "
    out = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(bin_))
    out += struct.pack("<II", len(js), 0x4E4F534A) + js + struct.pack("<II", len(bin_), 0x004E4942) + bytes(bin_)
    with open(path, "wb") as f:
        f.write(out)


def write_wav(path, samples, rate=22050):
    """samples: floats in [-1, 1], mono."""
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(b"".join(struct.pack("<h", int(max(-1, min(1, s)) * 32000)) for s in samples))


def tone(seconds, f0, f1=None, rate=22050, shape="sine", attack=0.005, decay=None, gain=0.6):
    """A sweep from f0 to f1 Hz with a short attack and an exponential decay."""
    f1 = f0 if f1 is None else f1
    n = int(seconds * rate)
    out, phase = [], 0.0
    decay = decay or seconds / 4
    for i in range(n):
        t = i / rate
        f = f0 + (f1 - f0) * (i / max(1, n - 1))
        phase += 2 * math.pi * f / rate
        if shape == "square":
            s = 1.0 if math.sin(phase) >= 0 else -1.0
            s *= 0.5
        elif shape == "tri":
            s = 2 / math.pi * math.asin(math.sin(phase))
        else:
            s = math.sin(phase)
        env = min(1.0, t / attack) * math.exp(-t / decay)
        out.append(s * env * gain)
    return out


def noise(seconds, rate=22050, decay=0.08, gain=0.5, seed=1):
    x = seed & 0xFFFFFFFF
    out = []
    for i in range(int(seconds * rate)):
        x ^= (x << 13) & 0xFFFFFFFF; x ^= x >> 17; x ^= (x << 5) & 0xFFFFFFFF
        out.append(((x / 0xFFFFFFFF) * 2 - 1) * math.exp(-(i / rate) / decay) * gain)
    return out


def mix(*parts):
    n = max(len(p) for p in parts)
    return [sum(p[i] for p in parts if i < len(p)) for i in range(n)]


def concat(*parts):
    out = []
    for p in parts:
        out.extend(p)
    return out


def subset_font(src, dst, text):
    """Subset a TTF to Latin-1 plus `text` (needs fontTools)."""
    from fontTools import subset
    opts = subset.Options()
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.layout_features = []
    font = subset.load_font(src, opts)
    s = subset.Subsetter(opts)
    s.populate(text="".join(chr(c) for c in range(0x20, 0x7F)) + "".join(chr(c) for c in range(0xA0, 0x100)) + text)
    s.subset(font)
    subset.save_font(font, dst, opts)
