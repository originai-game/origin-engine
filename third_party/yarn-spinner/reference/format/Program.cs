using System;
using System.Globalization;
using System.Text.Json;
CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
foreach (var n in new float[] {1234.5f, -1234.5f, 0f, 0.125f, 0.00001f, 1e10f, 0.1f, -0f, -0.01f, 9.999f, 99.999f, 999.999f, 12345678f, 16777217f, float.Epsilon, float.MaxValue, 0.99999994f, 1234.56789f})
foreach (var spec in new [] {"F2", "N0", "E", "e2", "G", "G2", "G9", "R7", "C", "P1", "00000.00", "#.##", "#,#.00", "#,,.0", "0.0%", "0.0‰", "0.00E+00", "0.0;[0.0];zero", "'v='0.00", "00##", "##00", "##.00E-0", "0.00e0", "0.0;[0.0]", "0.", "0\\;0", "0.00‰%", "0.0000000000", "0,,", "0.0;;zero", "D2", "X", "B", "G0", "R2"}) {
  try { Console.WriteLine(JsonSerializer.Serialize(new {value=n, spec, expected=n.ToString(spec)})); }
  catch (Exception e) { Console.WriteLine(JsonSerializer.Serialize(new {value=n, spec, error=e.GetType().Name})); }
}
