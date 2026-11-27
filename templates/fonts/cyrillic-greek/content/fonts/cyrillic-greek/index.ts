import {file, ref} from '@origingame/engine/author';

export default file['font-extension']({
  "kind": "font",
  "name": "cyrillic-greek",
  "languages": [
    "ru",
    "uk",
    "be",
    "bg",
    "sr",
    "mk",
    "kk",
    "ky",
    "mn",
    "tg",
    "el"
  ],
  "scripts": [
    "Cyrl",
    "Grek"
  ],
  "faces": [ref("base/fonts/cyrillic-greek/NotoSans-CyrillicGreek")],
  "hyphenation": [
    "assets/fonts/cyrillic-greek/ru.hyph",
    "assets/fonts/cyrillic-greek/uk.hyph",
    "assets/fonts/cyrillic-greek/be.hyph",
    "assets/fonts/cyrillic-greek/bg.hyph",
    "assets/fonts/cyrillic-greek/sr.hyph",
    "assets/fonts/cyrillic-greek/mn.hyph",
    "assets/fonts/cyrillic-greek/el.hyph"
  ]
});
