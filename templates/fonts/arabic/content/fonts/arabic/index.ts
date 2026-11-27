import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "arabic",
  "languages": [
    "ar",
    "fa",
    "ur",
    "ps",
    "sd",
    "ug",
    "ckb"
  ],
  "scripts": [
    "Arab"
  ],
  "faces": [ref("base/fonts/arabic/NotoSansArabic-Regular")],
  "hyphenation": []
});
