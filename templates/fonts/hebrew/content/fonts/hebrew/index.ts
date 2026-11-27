import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "hebrew",
  "languages": [
    "he",
    "iw",
    "yi"
  ],
  "scripts": [
    "Hebr"
  ],
  "faces": [ref("base/fonts/hebrew/NotoSansHebrew-Regular")],
  "hyphenation": []
});
