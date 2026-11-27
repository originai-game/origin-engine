<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-table"></a>
### `file:table`

Checked `table` input. Record roles and project-specific declarations are bound by `og check`.

```json
{
  "$defs": {
    "RowInput": {
      "additionalProperties": false,
      "properties": {
        "id": {
          "minLength": 1,
          "type": "string"
        },
        "values": {
          "description": "All writable columns may be omitted, including str/list and columns\nwithout default lanes. Do not insert zero values here: packed layouts\nmay not carry every canonical author default. Asset/row/localization\nsemantics also need author metadata, not guesses from numeric columns.",
          "not": {},
          "x-og-record-role": "row"
        }
      },
      "required": [
        "id",
        "values"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "additionalProperties": false,
  "properties": {
    "locale": {
      "description": "The base language of localized fields; omitted means en.",
      "minLength": 1,
      "type": [
        "string",
        "null"
      ]
    },
    "record": {
      "description": "The canonical record name, fixed by schema_with_record.",
      "not": {},
      "x-og-record-role": "record-name"
    },
    "rows": {
      "description": "Authored order is retained; ids are stable identities, never indices.",
      "items": {
        "$ref": "#/$defs/RowInput"
      },
      "type": "array"
    }
  },
  "required": [
    "record",
    "rows"
  ],
  "title": "TableInput",
  "type": "object"
}
```
