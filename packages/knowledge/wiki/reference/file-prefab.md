<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-prefab"></a>
### `file:prefab`

Checked `prefab` input. Record roles and project-specific declarations are bound by `og check`.

```json
{
  "$defs": {
    "ContentRef": {
      "additionalProperties": false,
      "description": "A content identity, never a runtime handle or an instruction to evaluate\nanother entry. The project graph validates names, existence, subresources,\nmodule access and target kinds. A field declares its allowed kinds with\n`x-og-ref-kinds` schema metadata, not a second kind inside this token.\n`x-og-ref-factories` instead requires an original declaration (no suffix),\nfor source consumers such as parameterized model programs or theme recipes.",
      "properties": {
        "$ogRef": {
          "minLength": 1,
          "type": "string"
        }
      },
      "required": [
        "$ogRef"
      ],
      "type": "object",
      "x-og-content-ref": true
    },
    "EntityInput": {
      "additionalProperties": false,
      "properties": {
        "children": {
          "default": [],
          "items": {
            "$ref": "#/$defs/EntityInput"
          },
          "type": "array"
        },
        "components": {
          "default": {},
          "not": {},
          "x-og-record-role": "component"
        },
        "edits": {
          "default": [],
          "description": "Applied in order to an instance's existing tree. An override with path\n\"\" can patch the root again after an earlier edit. Added children follow\nthe inherited tree, as in the existing prefab flattener.",
          "items": {
            "$ref": "#/$defs/PrefabEdit"
          },
          "type": "array"
        },
        "model": {
          "anyOf": [
            {
              "$ref": "#/$defs/ContentRef"
            },
            {
              "type": "null"
            }
          ],
          "x-og-ref-kinds": [
            "model"
          ]
        },
        "name": {
          "type": "string"
        },
        "prefab": {
          "anyOf": [
            {
              "$ref": "#/$defs/ContentRef"
            },
            {
              "type": "null"
            }
          ],
          "x-og-ref-kinds": [
            "prefab"
          ]
        },
        "scene": {
          "anyOf": [
            {
              "$ref": "#/$defs/ContentRef"
            },
            {
              "type": "null"
            }
          ],
          "x-og-ref-kinds": [
            "scene"
          ]
        },
        "static": {
          "type": [
            "boolean",
            "null"
          ]
        }
      },
      "required": [
        "name"
      ],
      "type": "object"
    },
    "PrefabEdit": {
      "oneOf": [
        {
          "additionalProperties": false,
          "properties": {
            "components": {
              "default": {},
              "not": {},
              "x-og-record-role": "component"
            },
            "kind": {
              "const": "override",
              "type": "string"
            },
            "model": {
              "anyOf": [
                {
                  "$ref": "#/$defs/ContentRef"
                },
                {
                  "type": "null"
                }
              ],
              "x-og-ref-kinds": [
                "model"
              ]
            },
            "path": {
              "description": "Relative descendant path, or \"\" for the instance root.",
              "type": "string"
            },
            "static": {
              "type": [
                "boolean",
                "null"
              ]
            }
          },
          "required": [
            "kind",
            "path"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "properties": {
            "component": {
              "description": "Omitted removes the subtree; present removes that component only.",
              "not": {},
              "x-og-record-role": "component-name"
            },
            "kind": {
              "const": "remove",
              "type": "string"
            },
            "path": {
              "type": "string"
            }
          },
          "required": [
            "kind",
            "path",
            "component"
          ],
          "type": "object"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "base": {
      "anyOf": [
        {
          "$ref": "#/$defs/ContentRef"
        },
        {
          "type": "null"
        }
      ],
      "x-og-ref-kinds": [
        "prefab"
      ]
    },
    "entity": {
      "$ref": "#/$defs/EntityInput",
      "description": "A prefab has exactly one root, including a variant of a base prefab."
    }
  },
  "required": [
    "entity"
  ],
  "title": "PrefabInput",
  "type": "object"
}
```
