import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "chinese-traditional",
  "languages": [
    "zh-Hant",
    "zh-TW",
    "zh-HK",
    "zh-MO"
  ],
  "scripts": [
    "Hant"
  ],
  "faces": [ref("base/fonts/chinese-traditional/NotoSansTC-Regular")],
  "hyphenation": []
});
