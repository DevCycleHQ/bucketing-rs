using System;
using System.Collections.Generic;
using DevCycleFFI;
using System.Diagnostics;

namespace DevCycleFFI;

class Program
{
    // High-accuracy step results (wall-clock). Optional allocation + GC stats in detailed mode.
    private readonly record struct StepResult(string Label, double ElapsedMs, long AllocBytes, int[]? GcCollections);

    private static readonly List<StepResult> Steps = new(capacity: 16);
    private static bool DetailedProfile = false;
    private static bool UnifiedInitFallbackUsed = false;
    private static int BenchmarkRuns = 100; // default runs
    private static bool DebugMode = false;  // disable detailed DevCycleClient debug by default for cleaner timing
    private static bool NativeAvailable = true;

    // Added defaults for variable benchmarking
    private static string VariableKey = "test"; // override with --variable-key=KEY or env DVC_VARIABLE_KEY
    private static string VariableType = "String"; // override with --variable-type=TYPE or env DVC_VARIABLE_TYPE
    private static bool ListVariables = false; // new flag
    private static Dictionary<string, string> VariableCatalog = new(); // key -> type

    static void Main(string[] args)
    {
        ParseArgs(args);

        DevCycleClient.SetDebugMode(DebugMode);
        long totalStart = Stopwatch.GetTimestamp();
        Info("DevCycle FFI Example (C#)");
        if (DetailedProfile) Info("Detailed profiling enabled (allocations + GC collections)");

        NativeAvailable = CheckNativeLibrary();
        Steps.Add(new StepResult("Check native library", 0, 0, null));
        if (!NativeAvailable)
        {
            Warn("Native library not found – initialization & benchmarking will be skipped.");
            PrintTimingSummary(TicksToMs(Stopwatch.GetTimestamp() - totalStart));
            return;
        }

        string sdkKey = "dvc_server_token_hash";
        using var client = new DevCycleClient(sdkKey);

        try
        {
            string config = Measure("Load configuration", CreateSampleConfig);
            // Extract variable catalog early (best-effort) so user can choose valid key
            TryExtractVariableCatalog(config);
            if (ListVariables && VariableCatalog.Count > 0)
            {
                Console.WriteLine();
                Console.WriteLine("Available Variables (from config)");
                Console.WriteLine(new string('=', 36));
                foreach (var kvp in VariableCatalog)
                {
                    Console.WriteLine($"{kvp.Key} : {kvp.Value}");
                }
                Console.WriteLine(new string('-', 36));
                Console.WriteLine("Use --variable-key=<key> --variable-type=<type> to benchmark a specific variable.");
            }

            var customData = new Dictionary<string, object>
            {
                { "appVersion", "1.2.3" },
                { "platform", "C#" },
                { "feature", "FFI Integration" }
            };
            var platformData = Measure("Generate platform data", PlatformData.Generate);

            Measure("Init SDK key", () =>
            {
                try
                {
                    client.InitSdkKey(config, platformData: platformData, customData: customData, initializeEventQueue: true);
                }
                catch (DevCycleException ex) when (
                    ex.InnerException is EntryPointNotFoundException ||
                    ex.InnerException is DllNotFoundException ||
                    ex.Message.Contains("Unable to find an entry point named") ||
                    ex.Message.Contains("EntryPointNotFoundException"))
                {
                    UnifiedInitFallbackUsed = true;
                }
                catch (EntryPointNotFoundException)
                {
                    UnifiedInitFallbackUsed = true;
                }
                catch (DllNotFoundException)
                {
                    UnifiedInitFallbackUsed = true;
                    NativeAvailable = false;
                }
            });

            if (!NativeAvailable)
            {
                Warn("Native library calls unavailable – aborting benchmark.");
            }
            else if (UnifiedInitFallbackUsed)
            {
                Warn("Unified init_sdk_key missing – using fallback sequence.");
                Measure("Set configuration (fallback)", () => client.SetConfig(config));
                Measure("Set platform data (fallback)", () => client.SetPlatformData(platformData));
                Measure("Set client custom data (fallback)", () => client.SetClientCustomData(customData));
                Measure("Initialize event queue (fallback)", () =>
                {
                    try { client.InitializeEventQueue(); } catch (DevCycleException ex) { VerboseException(ex); }
                });
            }

            DevCycleUser? user = null;
            if (NativeAvailable)
            {
                user = Measure("Create user", () => new DevCycleUser
                {
                    UserId = "user-123",
                    Email = "user@example.com",
                    Name = "Test User",
                    Country = "US",
                    CustomData = new Dictionary<string, object> { { "plan", "premium" }, { "age", 25 } }
                });

                // Warm-up (raw generation)
                client.GenerateBucketedConfigRaw(user);

                Measure("Benchmark bucketed config generation", () =>
                {
                    var timings = BenchmarkBucketedConfig(client, user!, BenchmarkRuns);
                    PrintBenchmarkSummary(timings);
                });

                // Variable evaluation warm-up & benchmark
                try
                {
                    client.GetVariableForUser(user!, VariableKey, VariableType); // warm-up (ignore result)
                }
                catch (Exception ex)
                {
                    Warn($"Variable warm-up failed: {ex.GetType().Name}: {ex.Message}");
                }

                Measure("Benchmark variable evaluation", () =>
                {
                    var (varTimes, errorCount) = BenchmarkVariableForUser(client, user!, VariableKey, VariableType, BenchmarkRuns);
                    PrintVariableBenchmarkSummary(varTimes, errorCount, VariableKey, VariableType);
                });
            }

            Info("Execution complete.");
        }
        catch (DllNotFoundException ex)
        {
            Error($"DLL not found: {ex.Message}");
            NativeAvailable = false;
        }
        catch (Exception ex)
        {
            Error($"Unexpected Error: {ex.Message}");
        }
        finally
        {
            long totalEnd = Stopwatch.GetTimestamp();
            PrintTimingSummary(TicksToMs(totalEnd - totalStart));
        }
    }

    // Check for native library presence
    static bool CheckNativeLibrary()
    {
        var currentDir = Directory.GetCurrentDirectory();
        var candidateDirs = new List<string>
        {
            currentDir,
            Path.Combine(currentDir, "bin", "Debug", "net8.0"),
            Path.Combine(currentDir, "bin", "Release", "net8.0")
        };
        var envPath = Environment.GetEnvironmentVariable("DVC_LIB_PATH");
        if (!string.IsNullOrEmpty(envPath)) candidateDirs.Insert(0, envPath);

        var possibleNames = new[] { "libdevcycle_bucketing_rs.dylib", "libdevcycle_bucketing_rs.so", "devcycle_bucketing_rs.dll" };
        foreach (var dir in candidateDirs)
        {
            foreach (var name in possibleNames)
            {
                var full = Path.Combine(dir, name);
                if (File.Exists(full))
                {
                    Info($"Found native library: {full}");
                    return true;
                }
            }
        }
        return false;
    }

    /// <summary>
    /// Creates a sample configuration JSON for testing
    /// </summary>
    static string CreateSampleConfig()
    {
        var testConfigPath = Path.Combine(
            Directory.GetCurrentDirectory(),
            "..", "..", "..", "..", "..", "..",
            "tests", "resources", "production_config.json");

        var fallbackPaths = new[]
        {
            "../../../tests/resources/production_config.json",
            "../../../../tests/resources/production_config.json",
            "../../../../../tests/resources/production_config.json",
            "../../../../../../tests/resources/production_config.json"
        };

        string? configJson = null;

        if (File.Exists(testConfigPath))
        {
            configJson = File.ReadAllText(testConfigPath);
        }
        else
        {
            foreach (var fallbackPath in fallbackPaths)
            {
                var fullPath = Path.GetFullPath(fallbackPath);
                if (File.Exists(fullPath)) { configJson = File.ReadAllText(fullPath); break; }
            }
        }

        if (configJson == null)
            throw new FileNotFoundException("Test configuration file not found in expected locations.");

        return configJson;
    }

    // Attempt to parse production config JSON to build variable catalog
    private static void TryExtractVariableCatalog(string configJson)
    {
        VariableCatalog.Clear();
        if (string.IsNullOrWhiteSpace(configJson)) return;
        try
        {
            using var doc = System.Text.Json.JsonDocument.Parse(configJson);
            var root = doc.RootElement;
            // Heuristics: look for top-level "features" or "variables" arrays/objects
            if (root.TryGetProperty("variables", out var varsElem))
            {
                // Could be object of key -> { type, ... }
                if (varsElem.ValueKind == System.Text.Json.JsonValueKind.Object)
                {
                    foreach (var prop in varsElem.EnumerateObject())
                    {
                        string type = ExtractType(prop.Value);
                        VariableCatalog[prop.Name] = type;
                    }
                }
                else if (varsElem.ValueKind == System.Text.Json.JsonValueKind.Array)
                {
                    foreach (var item in varsElem.EnumerateArray())
                    {
                        if (item.ValueKind == System.Text.Json.JsonValueKind.Object && item.TryGetProperty("key", out var keyProp))
                        {
                            string key = keyProp.GetString() ?? string.Empty;
                            if (string.IsNullOrEmpty(key)) continue;
                            string type = ExtractType(item);
                            VariableCatalog[key] = type;
                        }
                    }
                }
            }
            // Some configs might nest under features -> variables
            if (VariableCatalog.Count == 0 && root.TryGetProperty("features", out var featuresElem))
            {
                if (featuresElem.ValueKind == System.Text.Json.JsonValueKind.Object)
                {
                    foreach (var feat in featuresElem.EnumerateObject())
                    {
                        var fVal = feat.Value;
                        if (fVal.ValueKind == System.Text.Json.JsonValueKind.Object && fVal.TryGetProperty("variables", out var fv))
                        {
                            if (fv.ValueKind == System.Text.Json.JsonValueKind.Object)
                            {
                                foreach (var vProp in fv.EnumerateObject())
                                {
                                    string type = ExtractType(vProp.Value);
                                    VariableCatalog[vProp.Name] = type;
                                }
                            }
                            else if (fv.ValueKind == System.Text.Json.JsonValueKind.Array)
                            {
                                foreach (var item in fv.EnumerateArray())
                                {
                                    if (item.ValueKind == System.Text.Json.JsonValueKind.Object && item.TryGetProperty("key", out var keyProp))
                                    {
                                        string key = keyProp.GetString() ?? string.Empty;
                                        if (string.IsNullOrEmpty(key)) continue;
                                        string type = ExtractType(item);
                                        VariableCatalog[key] = type;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        catch
        {
            // Ignore parse errors silently for catalog
        }
    }

    private static string ExtractType(System.Text.Json.JsonElement elem)
    {
        if (elem.ValueKind == System.Text.Json.JsonValueKind.Object)
        {
            if (elem.TryGetProperty("type", out var t))
            {
                return t.GetString() ?? "unknown";
            }
            if (elem.TryGetProperty("valueType", out var vt))
            {
                return vt.GetString() ?? "unknown";
            }
        }
        return "unknown";
    }

    // High-accuracy measurement (excludes logging overhead inside measured delegate)
    private static void Measure(string label, Action action)
    {
        var result = MeasureInternal(label, () => { action(); return (object?)null; });
    }

    private static T Measure<T>(string label, Func<T> func)
    {
        var boxed = MeasureInternal(label, () => (object?)func());
        return (T)boxed!;
    }

    private static object? MeasureInternal(string label, Func<object?> func)
    {
        long start = Stopwatch.GetTimestamp();
        long allocStart = 0;
        int[]? gcStart = null;
        if (DetailedProfile)
        {
            allocStart = GC.GetTotalAllocatedBytes(precise: false);
            gcStart = new int[3];
            for (int i = 0; i < 3; i++) gcStart[i] = GC.CollectionCount(i);
        }
        object? result = func();
        long end = Stopwatch.GetTimestamp();
        long allocBytes = 0;
        int[]? gcDelta = null;
        if (DetailedProfile)
        {
            allocBytes = GC.GetTotalAllocatedBytes(precise: false) - allocStart;
            gcDelta = new int[3];
            for (int i = 0; i < 3; i++) gcDelta[i] = GC.CollectionCount(i) - gcStart![i];
        }
        var elapsedMs = TicksToMs(end - start);
        Steps.Add(new StepResult(label, elapsedMs, allocBytes, gcDelta));
        Info($"[STEP] {label} {elapsedMs,8:F2} ms" + (DetailedProfile ? $" | alloc {FormatBytes(allocBytes),9} | GC [{FormatGc(gcDelta)}]" : string.Empty));
        return result;
    }

    private static double TicksToMs(long ticks) => ticks * 1000.0 / Stopwatch.Frequency;

    private static string FormatBytes(long bytes)
    {
        if (bytes < 1024) return bytes + " B";
        double val = bytes;
        string[] units = ["B", "KB", "MB", "GB", "TB"];
        int u = 0;
        while (val >= 1024 && u < units.Length - 1) { val /= 1024; u++; }
        return $"{val:F2} {units[u]}";
    }

    private static string FormatGc(int[]? gc)
    {
        if (gc == null) return "-";
        return string.Join(',', new[] { gc[0], gc[1], gc[2] });
    }

    private static void PrintTimingSummary(double totalMs)
    {
        Console.WriteLine();
        Console.WriteLine("Timing Summary (wall-clock)" + (DetailedProfile ? " + allocations & GC" : string.Empty));
        Console.WriteLine(new string('=', 60));
        Console.WriteLine($"{"Step",-32} {"Time (ms)",10} {(DetailedProfile ? "Alloc".PadLeft(14) + "  GC(0,1,2)" : string.Empty)}");
        Console.WriteLine(new string('-', 60));
        foreach (var s in Steps)
        {
            if (DetailedProfile)
            {
                Console.WriteLine($"{s.Label,-32} {s.ElapsedMs,10:F2} {FormatBytes(s.AllocBytes),14}  {FormatGc(s.GcCollections)}");
            }
            else
            {
                Console.WriteLine($"{s.Label,-32} {s.ElapsedMs,10:F2}");
            }
        }
        Console.WriteLine(new string('-', 60));
        Console.WriteLine($"Total{"",-28} {totalMs,10:F2}");
        Console.WriteLine();

        if (UnifiedInitFallbackUsed)
        {
            Warn("Fallback path used: native devcycle_init_sdk_key symbol missing.");
        }
        Info("Note: Times are wall-clock and include any blocking inside steps; detailed mode adds allocation & GC deltas which can introduce slight overhead.");
    }

    // Benchmark logic (returns list of per-run milliseconds)
    private static List<double> BenchmarkBucketedConfig(DevCycleClient client, DevCycleUser user, int runs)
    {
        var times = new List<double>(runs);
        for (int i = 0; i < runs; i++)
        {
            long start = Stopwatch.GetTimestamp();
            client.GenerateBucketedConfigRaw(user);
            long end = Stopwatch.GetTimestamp();
            double ms = TicksToMs(end - start);
            times.Add(ms);
        }
        return times;
    }

    // New variable benchmark logic (returns list of per-run ms + error count)
    private static (List<double> Times, int ErrorCount) BenchmarkVariableForUser(DevCycleClient client, DevCycleUser user, string variableKey, string variableType, int runs)
    {
        var times = new List<double>(runs);
        int errorCount = 0;
        for (int i = 0; i < runs; i++)
        {
            long start = Stopwatch.GetTimestamp();
            try
            {
                var result = client.GetVariableForUser(user, variableKey, variableType);
                if (result.IsError) errorCount++;
            }
            catch (Exception ex)
            {
                // Count as error, continue
                errorCount++;
                if (DebugMode)
                {
                    Warn($"Variable evaluation exception (run {i + 1}): {ex.GetType().Name}: {ex.Message}");
                }
            }
            long end = Stopwatch.GetTimestamp();
            double ms = TicksToMs(end - start);
            times.Add(ms);
        }
        return (times, errorCount);
    }

    private static void PrintBenchmarkSummary(List<double> times)
    {
        if (times.Count == 0)
        {
            Warn("No benchmark timings collected.");
            return;
        }
        double min = double.MaxValue, max = double.MinValue, sum = 0.0;
        foreach (var t in times)
        {
            if (t < min) min = t;
            if (t > max) max = t;
            sum += t;
        }
        double avg = sum / times.Count;
        // Standard deviation (population)
        double varianceSum = 0.0;
        foreach (var t in times) varianceSum += (t - avg) * (t - avg);
        double stdDev = Math.Sqrt(varianceSum / times.Count);

        // 50th, 90th, 95th, 99th percentiles
        var sorted = new List<double>(times);
        sorted.Sort();
        double P(double p)
        {
            if (sorted.Count == 1) return sorted[0];
            double idx = (p / 100.0) * (sorted.Count - 1);
            int lo = (int)Math.Floor(idx);
            int hi = (int)Math.Ceiling(idx);
            if (lo == hi) return sorted[lo];
            double frac = idx - lo;
            return sorted[lo] + (sorted[hi] - sorted[lo]) * frac;
        }

        Console.WriteLine();
        Console.WriteLine("Bucketed Config Benchmark Summary");
        Console.WriteLine(new string('=', 40));
        Console.WriteLine($"Runs            : {times.Count}");
        Console.WriteLine($"Min (ms)        : {min:F4}");
        Console.WriteLine($"Max (ms)        : {max:F4}");
        Console.WriteLine($"Avg (ms)        : {avg:F4}");
        Console.WriteLine($"Std Dev (ms)    : {stdDev:F4}");
        Console.WriteLine($"P50 (ms)        : {P(50):F4}");
        Console.WriteLine($"P90 (ms)        : {P(90):F4}");
        Console.WriteLine($"P95 (ms)        : {P(95):F4}");
        Console.WriteLine($"P99 (ms)        : {P(99):F4}");
        Console.WriteLine(new string('-', 40));
        Console.WriteLine("Note: First warm-up run excluded from statistics; debug logging disabled unless --debug or DVC_DEBUG=1.");
    }

    // New variable benchmark summary
    private static void PrintVariableBenchmarkSummary(List<double> times, int errorCount, string variableKey, string variableType)
    {
        if (times.Count == 0)
        {
            Warn("No variable benchmark timings collected.");
            return;
        }
        double min = double.MaxValue, max = double.MinValue, sum = 0.0;
        foreach (var t in times)
        {
            if (t < min) min = t;
            if (t > max) max = t;
            sum += t;
        }
        double avg = sum / times.Count;
        double varianceSum = 0.0;
        foreach (var t in times) varianceSum += (t - avg) * (t - avg);
        double stdDev = Math.Sqrt(varianceSum / times.Count);
        var sorted = new List<double>(times);
        sorted.Sort();
        double P(double p)
        {
            if (sorted.Count == 1) return sorted[0];
            double idx = (p / 100.0) * (sorted.Count - 1);
            int lo = (int)Math.Floor(idx);
            int hi = (int)Math.Ceiling(idx);
            if (lo == hi) return sorted[lo];
            double frac = idx - lo;
            return sorted[lo] + (sorted[hi] - sorted[lo]) * frac;
        }
        Console.WriteLine();
        Console.WriteLine("Variable Evaluation Benchmark Summary");
        Console.WriteLine(new string('=', 46));
        Console.WriteLine($"Variable Key     : {variableKey}");
        Console.WriteLine($"Variable Type    : {variableType}");
        Console.WriteLine($"Runs             : {times.Count}");
        Console.WriteLine($"Errors           : {errorCount}");
        Console.WriteLine($"Min (ms)         : {min:F4}");
        Console.WriteLine($"Max (ms)         : {max:F4}");
        Console.WriteLine($"Avg (ms)         : {avg:F4}");
        Console.WriteLine($"Std Dev (ms)     : {stdDev:F4}");
        Console.WriteLine($"P50 (ms)         : {P(50):F4}");
        Console.WriteLine($"P90 (ms)         : {P(90):F4}");
        Console.WriteLine($"P95 (ms)         : {P(95):F4}");
        Console.WriteLine($"P99 (ms)         : {P(99):F4}");
        Console.WriteLine(new string('-', 46));
        Console.WriteLine("Note: Debug logging disabled unless --debug or DVC_DEBUG=1.");
    }

    // Lightweight logging helpers
    private static void Info(string message) => Console.WriteLine(message);
    private static void Warn(string message) => Console.WriteLine($"[WARN] {message}");
    private static void Error(string message) => Console.WriteLine($"[ERROR] {message}");
    private static void VerboseException(Exception ex)
    {
        // Keep minimal to avoid perturbing timings significantly.
        Warn($"Optional step exception: {ex.GetType().Name}: {ex.Message}");
    }

    private static void ParseArgs(string[] args)
    {
        for (int i = 0; i < args.Length; i++)
        {
            var a = args[i];
            if (a == "--debug") DebugMode = true;
            else if (a == "--profile-detailed" || a == "-pd") DetailedProfile = true;
            else if (a.StartsWith("--runs="))
            {
                if (int.TryParse(a.Substring("--runs=".Length), out var r) && r > 0) BenchmarkRuns = r;
            }
            else if ((a == "--runs" || a == "-r") && i + 1 < args.Length)
            {
                if (int.TryParse(args[i + 1], out var r) && r > 0) BenchmarkRuns = r;
            }
            else if (a.StartsWith("--variable-key="))
            {
                var val = a.Substring("--variable-key=".Length);
                if (!string.IsNullOrWhiteSpace(val)) VariableKey = val;
            }
            else if (a.StartsWith("--variable-type="))
            {
                var val = a.Substring("--variable-type=".Length);
                if (!string.IsNullOrWhiteSpace(val)) VariableType = val;
            }
            else if (a == "--list-variables") ListVariables = true;
        }
        var envRuns = Environment.GetEnvironmentVariable("DVC_RUNS");
        if (!string.IsNullOrEmpty(envRuns) && int.TryParse(envRuns, out var er) && er > 0) BenchmarkRuns = er;
        if (Environment.GetEnvironmentVariable("DVC_DEBUG") == "1") DebugMode = true;
        if (Environment.GetEnvironmentVariable("DVC_PROFILE_DETAILED") == "1") DetailedProfile = true;

        var envVarKey = Environment.GetEnvironmentVariable("DVC_VARIABLE_KEY");
        if (!string.IsNullOrWhiteSpace(envVarKey)) VariableKey = envVarKey;
        var envVarType = Environment.GetEnvironmentVariable("DVC_VARIABLE_TYPE");
        if (!string.IsNullOrWhiteSpace(envVarType)) VariableType = envVarType;

        if (Environment.GetEnvironmentVariable("DVC_LIST_VARIABLES") == "1") ListVariables = true;
    }
}
