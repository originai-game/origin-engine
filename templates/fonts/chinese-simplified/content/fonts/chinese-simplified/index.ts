import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "chinese-simplified",
  "languages": [
    "zh",
    "zh-Hans",
    "zh-CN",
    "zh-SG"
  ],
  "scripts": [
    "Hans"
  ],
  "faces": [ref("base/fonts/chinese-simplified/NotoSansSC-Regular")],
  "hyphenation": []
});
