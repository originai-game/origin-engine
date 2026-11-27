import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "japanese",
  "languages": [
    "ja"
  ],
  "scripts": [
    "Jpan"
  ],
  "faces": [ref("base/fonts/japanese/NotoSansJP-Regular")],
  "hyphenation": []
});
