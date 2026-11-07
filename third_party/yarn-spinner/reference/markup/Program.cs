using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Yarn.Markup;
using YarnSpinner.Tests;
using Xunit;
var inputs = new SortedSet<string>(StringComparer.Ordinal);
foreach (var method in typeof(MarkupTests).GetMethods(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
foreach (var data in method.GetCustomAttributes<InlineDataAttribute>())
foreach (var row in data.GetData(method))
  if (row.Length > 0 && row[0] is string input) inputs.Add(input);
foreach (var row in MarkupTests.RangeComparisons()) inputs.Add((string)row[0]);
inputs.Add("Mae: 😀 [wave speed=2 active=true]hello[/wave]");
foreach (var input in inputs) {
  var (markup, diagnostics) = new LineParser().ParseStringAndIncludeMarkupDiagnostics(input, "en");
  object scalar(MarkupValue v) => v.Type switch {
    MarkupValueType.Integer => v.IntegerValue,
    MarkupValueType.Float => (double)v.FloatValue,
    MarkupValueType.Bool => v.BoolValue,
    _ => v.StringValue,
  };
  Console.WriteLine(JsonSerializer.Serialize(new {
    input, locale="en", error=diagnostics.Count > 0, text=markup.Text,
    spans=markup.Attributes?.Select(a => new {name=a.Name, position=a.Position, length=a.Length,
      properties=a.Properties.ToDictionary(p => p.Key, p => scalar(p.Value))})
  }));
}
