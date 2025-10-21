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

    static void Main(string[] args)
    {
        ParseArgs(args);

        long totalStart = Stopwatch.GetTimestamp();
        Info("DevCycle FFI Example (C#)");
        if (DetailedProfile) Info("Detailed profiling enabled (allocations + GC collections)");

        Measure("Check native library", CheckNativeLibrary);

        string sdkKey = "dvc_server_token_hash";
        using var client = new DevCycleClient(sdkKey);

        try
        {
            string config = Measure("Load configuration", CreateSampleConfig);
            Measure("Set configuration", () => client.SetConfig(config));

            var platformData = Measure("Generate platform data", PlatformData.Generate);
            Measure("Set platform data", () => client.SetPlatformData(platformData));

            Measure("Initialize event queue", () =>
            {
                try { client.InitializeEventQueue(); }
                catch (DevCycleException ex) { VerboseException(ex); }
            });

            var customData = new Dictionary<string, object>
            {
                { "appVersion", "1.2.3" },
                { "platform", "C#" },
                { "feature", "FFI Integration" }
            };
            Measure("Set client custom data", () =>
            {
                try { client.SetClientCustomData(customData); }
                catch (DevCycleException ex) { VerboseException(ex); }
            });

            var user = Measure("Create user", () => new DevCycleUser
            {
                UserId = "user-123",
                Email = "user@example.com",
                Name = "Test User",
                Country = "US",
                CustomData = new Dictionary<string, object>
                {
                    { "plan", "premium" },
                    { "age", 25 }
                }
            });

            var bucketedConfig = Measure("Generate bucketed config", () => client.GenerateBucketedConfig(user));
            GC.KeepAlive(bucketedConfig);

            Info("All steps completed successfully.");
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

    // Argument parsing for profiling detail level
    private static void ParseArgs(string[] args)
    {
        foreach (var a in args)
        {
            if (a == "--profile-detailed" || a == "-pd") DetailedProfile = true;
        }
        if (Environment.GetEnvironmentVariable("DVC_PROFILE_DETAILED") == "1") DetailedProfile = true;
    }

    static void CheckNativeLibrary()
    {
        var currentDir = Directory.GetCurrentDirectory();
        var possibleNames = new[]
        {
            "libdevcycle_bucketing_rs.dylib",
            "libdevcycle_bucketing_rs.so",
            "devcycle_bucketing_rs.dll"
        };

        bool found = false;
        foreach (var name in possibleNames)
        {
            var path = Path.Combine(currentDir, name);
            if (File.Exists(path)) { found = true; break; }
        }

        if (!found)
        {
            Warn("Native library not found next to executable.");
            Info("Build with: cargo build --release --features ffi then copy the produced library here.");
        }
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
        Info("Note: Times are wall-clock and include any blocking inside steps; detailed mode adds allocation & GC deltas which can introduce slight overhead.");
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
}
