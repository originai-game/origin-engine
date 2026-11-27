import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "devanagari",
  "languages": [
    "hi",
    "mr",
    "ne",
    "sa"
  ],
  "scripts": [
    "Deva"
  ],
  "faces": [ref("base/fonts/devanagari/NotoSansDevanagari-Regular")],
  "hyphenation": []
});
