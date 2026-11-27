import {file} from '@origingame/engine/author';

export default file.import({
  "kind": "rive",
  "source": "assets/two_artboards.riv",
  "artboards": [
    "One"
  ],
  "settings": {
    "One": {
      "role": "button",
      "name": "@chest.open",
      "fit": "cover",
      "align": "top-left"
    }
  }
});
