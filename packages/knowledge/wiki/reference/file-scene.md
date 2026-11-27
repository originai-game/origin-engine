<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-scene"></a>
### `file:scene`

Checked `scene` input. Record roles and project-specific declarations are bound by `og check`.

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
    "HlodInput": {
      "additionalProperties": false,
      "properties": {
        "error": {
          "description": "Maximum proxy displacement in metres. Omitted/zero: size / 256;\na supplied positive value must be at most size / 4 (domain check).",
          "format": "float",
          "maximum": 3.4028234663852886e+38,
          "minimum": 0,
          "type": [
            "number",
            "null"
          ],
          "x-og-unit": "m"
        },
        "size": {
          "description": "Leaf cell side in metres; omitted: 64.",
          "format": "float",
          "maximum": 4096,
          "minimum": 4,
          "type": [
            "number",
            "null"
          ],
          "x-og-unit": "m"
        },
        "texels": {
          "description": "Longest proxy texture side in pixels; omitted: 1024.",
          "format": "uint32",
          "maximum": 4096,
          "minimum": 64,
          "type": [
            "integer",
            "null"
          ],
          "x-og-unit": "px"
        }
      },
      "type": "object"
    },
    "Path": {
      "additionalProperties": false,
      "properties": {
        "closed": {
          "type": [
            "boolean",
            "null"
          ]
        },
        "kind": {
          "anyOf": [
            {
              "$ref": "#/$defs/PathKind"
            },
            {
              "type": "null"
            }
          ]
        },
        "points": {
          "items": {
            "items": {
              "format": "float",
              "maximum": 3.4028234663852886e+38,
              "minimum": -3.4028234663852886e+38,
              "type": "number"
            },
            "maxItems": 3,
            "minItems": 3,
            "type": "array"
          },
          "type": "array",
          "x-og-unit": "m"
        },
        "tension": {
          "format": "float",
          "maximum": 3.4028234663852886e+38,
          "minimum": -3.4028234663852886e+38,
          "type": [
            "number",
            "null"
          ]
        }
      },
      "required": [
        "points"
      ],
      "type": "object"
    },
    "PathKind": {
      "enum": [
        "catmullRom",
        "bezier",
        "linear"
      ],
      "type": "string"
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
    },
    "StreamInput": {
      "additionalProperties": false,
      "properties": {
        "assets": {
          "default": [],
          "description": "A cell group may hold only its own subscene; other groups need assets.",
          "items": {
            "$ref": "#/$defs/ContentRef"
          },
          "type": "array"
        },
        "priority": {
          "description": "Higher loads first and unloads last; omitted: 0.",
          "format": "int32",
          "type": [
            "integer",
            "null"
          ]
        },
        "trigger": {
          "anyOf": [
            {
              "$ref": "#/$defs/StreamTriggerInput"
            },
            {
              "type": "null"
            }
          ],
          "description": "Omitted means manual loading only."
        }
      },
      "type": "object"
    },
    "StreamTriggerInput": {
      "oneOf": [
        {
          "additionalProperties": false,
          "properties": {
            "kind": {
              "const": "manual",
              "type": "string"
            }
          },
          "required": [
            "kind"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "properties": {
            "kind": {
              "const": "cell",
              "type": "string"
            }
          },
          "required": [
            "kind"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "properties": {
            "entity": {
              "type": "string"
            },
            "kind": {
              "const": "volume",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "entity"
          ],
          "type": "object"
        },
        {
          "additionalProperties": false,
          "properties": {
            "anchor": {
              "items": {
                "format": "float",
                "maximum": 3.4028234663852886e+38,
                "minimum": -3.4028234663852886e+38,
                "type": "number"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array",
              "x-og-unit": "m"
            },
            "kind": {
              "const": "distance",
              "type": "string"
            },
            "radius": {
              "exclusiveMinimum": 0,
              "format": "float",
              "maximum": 3.4028234663852886e+38,
              "minimum": -3.4028234663852886e+38,
              "type": "number",
              "x-og-unit": "m"
            },
            "unload": {
              "description": "Omitted: radius * 1.25; cannot be less than radius (domain check).",
              "format": "float",
              "maximum": 3.4028234663852886e+38,
              "minimum": 0,
              "type": [
                "number",
                "null"
              ],
              "x-og-unit": "m"
            }
          },
          "required": [
            "kind",
            "anchor",
            "radius"
          ],
          "type": "object"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "entities": {
      "default": [],
      "items": {
        "$ref": "#/$defs/EntityInput"
      },
      "type": "array"
    },
    "hlod": {
      "anyOf": [
        {
          "$ref": "#/$defs/HlodInput"
        },
        {
          "type": "null"
        }
      ]
    },
    "name": {
      "type": [
        "string",
        "null"
      ]
    },
    "paths": {
      "additionalProperties": {
        "$ref": "#/$defs/Path"
      },
      "default": {},
      "description": "Named local-space paths use the same typed path as timeline rails.",
      "type": "object"
    },
    "resources": {
      "default": {},
      "description": "Resource patches, never complete replacement values.",
      "not": {},
      "x-og-record-role": "resource"
    },
    "streams": {
      "additionalProperties": {
        "$ref": "#/$defs/StreamInput"
      },
      "default": {},
      "propertyNames": {
        "pattern": "^[A-Za-z0-9_-]{1,64}$"
      },
      "type": "object"
    }
  },
  "title": "SceneInput",
  "type": "object"
}
```
