using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json.Serialization;

namespace DevCycleFFI;

/// <summary>
/// Managed wrapper for the DevCycle Rust FFI library
/// </summary>
public class DevCycleClient : IDisposable
{
    private readonly string _sdkKey;
    private bool _disposed = false;
    private static bool _debugMode = true; // Enable debug logging

    public DevCycleClient(string sdkKey)
    {
        _sdkKey = sdkKey ?? throw new ArgumentNullException(nameof(sdkKey));
        LogDebug($"DevCycleClient created with SDK key: {sdkKey}");
    }

    /// <summary>
    /// Helper method to get the last error message from the FFI layer
    /// </summary>
    private static string GetLastFFIError()
    {
        var errorPtr = NativeMethods.devcycle_get_last_error();
        if (errorPtr == IntPtr.Zero)
        {
            return "No detailed error available";
        }

        try
        {
            var errorMessage = Marshal.PtrToStringAnsi(errorPtr);
            return errorMessage ?? "Unknown error";
        }
        finally
        {
            NativeMethods.devcycle_free_string(errorPtr);
        }
    }

    /// <summary>
    /// Initialize the event queue for this SDK key
    /// </summary>
    public void InitializeEventQueue()
    {
        LogDebug("InitializeEventQueue called");
        IntPtr sdkKeyPtr = IntPtr.Zero;

        try
        {
            LogDebug($"Converting SDK key to native string: '{_sdkKey}'");
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            LogDebug($"SDK key pointer: 0x{sdkKeyPtr:X}");

            LogDebug("Calling native devcycle_init_event_queue...");
            var result = NativeMethods.devcycle_init_event_queue(sdkKeyPtr, IntPtr.Zero);
            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to initialize event queue (error code {result}): {detailedError}");
            }

            LogDebug("Event queue initialized successfully");
        }
        catch (Exception ex) when (ex is not DevCycleException)
        {
            LogDebug($"Unexpected exception in InitializeEventQueue: {ex.GetType().Name}: {ex.Message}");
            LogDebug($"Stack trace: {ex.StackTrace}");
            throw new DevCycleException($"Failed to initialize event queue: {ex.Message}", ex);
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero)
            {
                LogDebug("Freeing SDK key pointer");
                Marshal.FreeHGlobal(sdkKeyPtr);
            }
        }
    }

    /// <summary>
    /// Set configuration from JSON string
    /// </summary>
    public void SetConfig(string configJson)
    {
        LogDebug("SetConfig called");
        if (string.IsNullOrEmpty(configJson))
            throw new ArgumentNullException(nameof(configJson));

        LogDebug($"Config JSON length: {configJson.Length} characters");
        IntPtr sdkKeyPtr = IntPtr.Zero;
        IntPtr configPtr = IntPtr.Zero;

        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            configPtr = Marshal.StringToHGlobalAnsi(configJson);
            LogDebug($"SDK key pointer: 0x{sdkKeyPtr:X}, Config pointer: 0x{configPtr:X}");

            LogDebug("Calling native devcycle_set_config...");
            var result = NativeMethods.devcycle_set_config(sdkKeyPtr, configPtr);
            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to set config (error code {result}): {detailedError}");
            }

            LogDebug("Config set successfully");
        }
        catch (Exception ex) when (ex is not DevCycleException)
        {
            LogDebug($"Unexpected exception in SetConfig: {ex.GetType().Name}: {ex.Message}");
            throw new DevCycleException($"Failed to set config: {ex.Message}", ex);
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(sdkKeyPtr);
            if (configPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(configPtr);
        }
    }

    /// <summary>
    /// Set client custom data from dictionary
    /// </summary>
    public void SetClientCustomData(Dictionary<string, object> customData)
    {
        LogDebug("SetClientCustomData called");
        if (customData == null)
            throw new ArgumentNullException(nameof(customData));

        var customDataJson = System.Text.Json.JsonSerializer.Serialize(customData);
        LogDebug($"Custom data JSON: {customDataJson}");

        IntPtr sdkKeyPtr = IntPtr.Zero;
        IntPtr customDataPtr = IntPtr.Zero;

        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            customDataPtr = Marshal.StringToHGlobalAnsi(customDataJson);
            LogDebug("Calling native devcycle_set_client_custom_data...");

            var result = NativeMethods.devcycle_set_client_custom_data(sdkKeyPtr, customDataPtr);
            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to set client custom data (error code {result}): {detailedError}");
            }

            LogDebug("Client custom data set successfully");
        }
        catch (Exception ex) when (ex is not DevCycleException)
        {
            LogDebug($"Unexpected exception: {ex.GetType().Name}: {ex.Message}");
            throw new DevCycleException($"Failed to set client custom data: {ex.Message}", ex);
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(sdkKeyPtr);
            if (customDataPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(customDataPtr);
        }
    }

    /// <summary>
    /// Set platform data for this SDK instance
    /// </summary>
    public void SetPlatformData(PlatformData platformData)
    {
        LogDebug("SetPlatformData called");
        if (platformData == null)
            throw new ArgumentNullException(nameof(platformData));

        var platformDataJson = System.Text.Json.JsonSerializer.Serialize(platformData);
        LogDebug($"Platform data JSON: {platformDataJson}");

        IntPtr sdkKeyPtr = IntPtr.Zero;
        IntPtr platformDataPtr = IntPtr.Zero;

        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            platformDataPtr = Marshal.StringToHGlobalAnsi(platformDataJson);
            LogDebug("Calling native devcycle_set_platform_data...");

            var result = NativeMethods.devcycle_set_platform_data(sdkKeyPtr, platformDataPtr);
            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to set platform data (error code {result}): {detailedError}");
            }

            LogDebug("Platform data set successfully");
        }
        catch (Exception ex) when (ex is not DevCycleException)
        {
            LogDebug($"Unexpected exception: {ex.GetType().Name}: {ex.Message}");
            throw new DevCycleException($"Failed to set platform data: {ex.Message}", ex);
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(sdkKeyPtr);
            if (platformDataPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(platformDataPtr);
        }
    }

    /// <summary>
    /// Initialize SDK key with all required data in one call
    /// This is a convenience method that combines SetConfig, SetPlatformData, SetClientCustomData, and InitializeEventQueue
    /// </summary>
    public void InitSdkKey(
        string configJson,
        PlatformData? platformData = null,
        Dictionary<string, object>? customData = null,
        bool initializeEventQueue = false)
    {
        LogDebug("InitSdkKey called");
        if (string.IsNullOrEmpty(configJson))
            throw new ArgumentNullException(nameof(configJson));

        // Use provided platform data or generate it
        var platform = platformData ?? PlatformData.Generate();
        var platformDataJson = System.Text.Json.JsonSerializer.Serialize(platform);

        // Serialize custom data if provided
        var customDataJson = customData != null
            ? System.Text.Json.JsonSerializer.Serialize(customData)
            : null;

        LogDebug($"Config JSON length: {configJson.Length} characters");
        LogDebug($"Platform: {platform.Platform}, Hostname: {platform.Hostname}");
        if (customData != null)
            LogDebug($"Custom data: {customDataJson}");

        IntPtr sdkKeyPtr = IntPtr.Zero;
        IntPtr configPtr = IntPtr.Zero;
        IntPtr platformDataPtr = IntPtr.Zero;
        IntPtr customDataPtr = IntPtr.Zero;

        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            configPtr = Marshal.StringToHGlobalAnsi(configJson);
            platformDataPtr = Marshal.StringToHGlobalAnsi(platformDataJson);

            if (customDataJson != null)
                customDataPtr = Marshal.StringToHGlobalAnsi(customDataJson);

            LogDebug("Calling native devcycle_init_sdk_key...");
            var result = NativeMethods.devcycle_init_sdk_key(
                sdkKeyPtr,
                configPtr,
                initializeEventQueue ? IntPtr.Zero : IntPtr.Zero,  // event queue options (null for default)
                customDataPtr == IntPtr.Zero ? IntPtr.Zero : customDataPtr,
                platformDataPtr);

            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to initialize SDK key (error code {result}): {detailedError}");
            }

            LogDebug("SDK key initialized successfully");
        }
        catch (Exception ex) when (ex is not DevCycleException)
        {
            LogDebug($"Unexpected exception in InitSdkKey: {ex.GetType().Name}: {ex.Message}");
            throw new DevCycleException($"Failed to initialize SDK key: {ex.Message}", ex);
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(sdkKeyPtr);
            if (configPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(configPtr);
            if (platformDataPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(platformDataPtr);
            if (customDataPtr != IntPtr.Zero)
                Marshal.FreeHGlobal(customDataPtr);
        }
    }

    /// <summary>
    /// Generate bucketed config for a user
    /// </summary>
    public string GenerateBucketedConfig(DevCycleUser user)
    {
        if (user == null)
            throw new ArgumentNullException(nameof(user));

        var userJson = System.Text.Json.JsonSerializer.Serialize(user);
        LogDebug($"User JSON being sent: {userJson}");

        var sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
        var userPtr = Marshal.StringToHGlobalAnsi(userJson);
        try
        {
            // First create user object from JSON
            LogDebug("Calling devcycle_user_from_json...");
            var cUserPtr = NativeMethods.devcycle_user_from_json(userPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var detailedError = GetLastFFIError();
                LogDebug($"Detailed error from FFI: {detailedError}");
                throw new DevCycleException($"Failed to create user from JSON: {detailedError}");
            }
            LogDebug($"User created successfully, pointer: 0x{cUserPtr:X}");

            try
            {
                var configPtr = NativeMethods.devcycle_generate_bucketed_config_from_user(
                    sdkKeyPtr, cUserPtr);

                if (configPtr == IntPtr.Zero)
                {
                    var detailedError = GetLastFFIError();
                    LogDebug($"Detailed error from FFI: {detailedError}");
                    throw new DevCycleException($"Failed to generate bucketed config: {detailedError}");
                }

                try
                {
                    // Get the JSON representation of the bucketed config
                    LogDebug("Converting bucketed config to JSON...");
                    var jsonPtr = NativeMethods.devcycle_bucketed_config_to_json(configPtr);

                    if (jsonPtr == IntPtr.Zero)
                    {
                        var detailedError = GetLastFFIError();
                        LogDebug($"Warning: Failed to serialize config to JSON: {detailedError}");
                        return "Config generated successfully (but could not serialize to JSON)";
                    }

                    try
                    {
                        var configJson = Marshal.PtrToStringAnsi(jsonPtr);
                        if (configJson != null)
                        {
                            // Pretty print the JSON
                            try
                            {
                                var jsonDoc = System.Text.Json.JsonDocument.Parse(configJson);
                                var prettyJson = System.Text.Json.JsonSerializer.Serialize(
                                    jsonDoc.RootElement,
                                    new System.Text.Json.JsonSerializerOptions { WriteIndented = true }
                                );
                                Console.WriteLine("\n=== Bucketed Config JSON ===");
                                Console.WriteLine(prettyJson);
                                Console.WriteLine("============================\n");
                                return prettyJson;
                            }
                            catch
                            {
                                // If pretty printing fails, just return the raw JSON
                                Console.WriteLine("\n=== Bucketed Config JSON ===");
                                Console.WriteLine(configJson);
                                Console.WriteLine("============================\n");
                                return configJson;
                            }
                        }
                        return "Config generated successfully";
                    }
                    finally
                    {
                        if (jsonPtr != IntPtr.Zero)
                            NativeMethods.devcycle_free_string(jsonPtr);
                    }
                }
                finally
                {
                    NativeMethods.devcycle_free_bucketed_config(configPtr);
                }
            }
            finally
            {
                NativeMethods.devcycle_free_user(cUserPtr);
            }
        }
        finally
        {
            Marshal.FreeHGlobal(sdkKeyPtr);
            Marshal.FreeHGlobal(userPtr);
        }
    }

    // Silent variant for benchmarking: no console output except debug
    public string GenerateBucketedConfigSilent(DevCycleUser user)
    {
        if (user == null)
            throw new ArgumentNullException(nameof(user));

        var userJson = System.Text.Json.JsonSerializer.Serialize(user);
        LogDebug($"User JSON being sent: {userJson}");

        var sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
        var userPtr = Marshal.StringToHGlobalAnsi(userJson);
        try
        {
            var cUserPtr = NativeMethods.devcycle_user_from_json(userPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var detailedError = GetLastFFIError();
                throw new DevCycleException($"Failed to create user from JSON: {detailedError}");
            }

            try
            {
                var configPtr = NativeMethods.devcycle_generate_bucketed_config_from_user(sdkKeyPtr, cUserPtr);
                if (configPtr == IntPtr.Zero)
                {
                    var detailedError = GetLastFFIError();
                    throw new DevCycleException($"Failed to generate bucketed config: {detailedError}");
                }

                try
                {
                    var jsonPtr = NativeMethods.devcycle_bucketed_config_to_json(configPtr);
                    if (jsonPtr == IntPtr.Zero)
                    {
                        var detailedError = GetLastFFIError();
                        LogDebug($"Warning: Failed to serialize config to JSON: {detailedError}");
                        return string.Empty;
                    }
                    try
                    {
                        var configJson = Marshal.PtrToStringAnsi(jsonPtr);
                        if (configJson != null)
                        {
                            // Attempt pretty-print (cost negligible for benchmarking correctness)
                            try
                            {
                                var jsonDoc = System.Text.Json.JsonDocument.Parse(configJson);
                                return System.Text.Json.JsonSerializer.Serialize(jsonDoc.RootElement);
                            }
                            catch
                            {
                                return configJson;
                            }
                        }
                        return string.Empty;
                    }
                    finally
                    {
                        if (jsonPtr != IntPtr.Zero)
                            NativeMethods.devcycle_free_string(jsonPtr);
                    }
                }
                finally
                {
                    NativeMethods.devcycle_free_bucketed_config(configPtr);
                }
            }
            finally
            {
                NativeMethods.devcycle_free_user(cUserPtr);
            }
        }
        finally
        {
            Marshal.FreeHGlobal(sdkKeyPtr);
            Marshal.FreeHGlobal(userPtr);
        }
    }

    /// <summary>
    /// Get a variable value for a user
    /// </summary>
    public VariableResult GetVariableForUser(DevCycleUser user, string variableKey, string variableType)
    {
        if (user == null)
            throw new ArgumentNullException(nameof(user));
        if (string.IsNullOrEmpty(variableKey))
            throw new ArgumentNullException(nameof(variableKey));
        if (string.IsNullOrEmpty(variableType))
            throw new ArgumentNullException(nameof(variableType));

        var userJson = System.Text.Json.JsonSerializer.Serialize(user);

        var sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
        var userJsonPtr = Marshal.StringToHGlobalAnsi(userJson);
        var variableKeyPtr = Marshal.StringToHGlobalAnsi(variableKey);
        var variableTypePtr = Marshal.StringToHGlobalAnsi(variableType);

        try
        {
            // Create user object from JSON - note: this creates a populated user
            // In a real scenario, you might need a separate function for PopulatedUser
            var cUserPtr = NativeMethods.devcycle_user_from_json(userJsonPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var detailedError = GetLastFFIError();
                throw new DevCycleException($"Failed to create user from JSON: {detailedError}");
            }

            try
            {
                // For this example, we'll assume the user pointer works as PopulatedUser
                // In production, you'd need proper conversion
                var resultPtr = NativeMethods.devcycle_variable_for_user(
                    sdkKeyPtr, cUserPtr, variableKeyPtr, variableTypePtr);

                if (resultPtr == IntPtr.Zero)
                {
                    var detailedError = GetLastFFIError();
                    throw new DevCycleException($"Failed to get variable for user: {detailedError}");
                }

                try
                {
                    // Get the variable value as JSON
                    var valueJsonPtr = NativeMethods.devcycle_variable_result_to_json(resultPtr);
                    var typePtr = NativeMethods.devcycle_variable_result_get_type(resultPtr);

                    try
                    {
                        var valueJson = Marshal.PtrToStringAnsi(valueJsonPtr) ?? "null";
                        var type = Marshal.PtrToStringAnsi(typePtr) ?? "unknown";

                        return new VariableResult
                        {
                            Value = valueJson,
                            Type = type
                        };
                    }
                    finally
                    {
                        if (valueJsonPtr != IntPtr.Zero)
                            NativeMethods.devcycle_free_string(valueJsonPtr);
                        if (typePtr != IntPtr.Zero)
                            NativeMethods.devcycle_free_string(typePtr);
                    }
                }
                finally
                {
                    NativeMethods.devcycle_free_variable_result(resultPtr);
                }
            }
            finally
            {
                NativeMethods.devcycle_free_user(cUserPtr);
            }
        }
        finally
        {
            Marshal.FreeHGlobal(sdkKeyPtr);
            Marshal.FreeHGlobal(userJsonPtr);
            Marshal.FreeHGlobal(variableKeyPtr);
            Marshal.FreeHGlobal(variableTypePtr);
        }
    }

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!_disposed)
        {
            // Clean up any resources if needed
            _disposed = true;
        }
    }

    private static void LogDebug(string message)
    {
        if (_debugMode)
        {
            var timestamp = DateTime.Now.ToString("HH:mm:ss.fff");
            Console.WriteLine($"[DEBUG {timestamp}] {message}");
        }
    }

    public static void SetDebugMode(bool enabled)
    {
        _debugMode = enabled;
    }
}

/// <summary>
/// Represents a DevCycle user
/// </summary>
public class DevCycleUser
{
    [JsonPropertyName("userId")]
    public string UserId { get; set; } = string.Empty;

    [JsonPropertyName("email")]
    public string Email { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("country")]
    public string Country { get; set; } = string.Empty;

    [JsonPropertyName("language")]
    public string Language { get; set; } = string.Empty;

    [JsonPropertyName("appVersion")]
    public string AppVersion { get; set; } = string.Empty;

    [JsonPropertyName("appBuild")]
    public string AppBuild { get; set; } = string.Empty;

    [JsonPropertyName("customData")]
    public Dictionary<string, object> CustomData { get; set; } = new Dictionary<string, object>();

    [JsonPropertyName("privateCustomData")]
    public Dictionary<string, object> PrivateCustomData { get; set; } = new Dictionary<string, object>();

    [JsonPropertyName("deviceModel")]
    public string DeviceModel { get; set; } = string.Empty;

    // Note: last_seen_date is handled automatically by the Rust User struct
    // We don't need to serialize it from C#
}

/// <summary>
/// Represents a variable result
/// </summary>
public class VariableResult
{
    public string Value { get; set; } = string.Empty;
    public string Type { get; set; } = string.Empty;
}

/// <summary>
/// Represents platform data for the SDK
/// </summary>
public class PlatformData
{
    [JsonPropertyName("sdkType")]
    public string SdkType { get; set; } = "server";

    [JsonPropertyName("sdkVersion")]
    public string SdkVersion { get; set; } = string.Empty;

    [JsonPropertyName("platformVersion")]
    public string PlatformVersion { get; set; } = string.Empty;

    [JsonPropertyName("deviceModel")]
    public string DeviceModel { get; set; } = "unknown";

    [JsonPropertyName("platform")]
    public string Platform { get; set; } = "CSharp";

    [JsonPropertyName("hostname")]
    public string Hostname { get; set; } = string.Empty;

    /// <summary>
    /// Creates a PlatformData instance with automatically detected values
    /// </summary>
    public static PlatformData Generate()
    {
        return new PlatformData
        {
            SdkType = "server",
            SdkVersion = "1.0.0", // Should be set to your actual SDK version
            PlatformVersion = Environment.OSVersion.VersionString,
            DeviceModel = "unknown",
            Platform = "CSharp",
            Hostname = Environment.MachineName
        };
    }
}

/// <summary>
/// Exception thrown by DevCycle operations
/// </summary>
public class DevCycleException : Exception
{
    public DevCycleException(string message) : base(message) { }
    public DevCycleException(string message, Exception innerException)
        : base(message, innerException) { }
}
