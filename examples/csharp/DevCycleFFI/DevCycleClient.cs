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
    private static (NativeMethods.DevCycleFFIErrorCode code, string message) GetLastErrorDetails()
    {
        var code = NativeMethods.devcycle_get_last_error_code();
        var msgPtr = NativeMethods.devcycle_get_last_error();
        string msg = "";
        if (msgPtr != IntPtr.Zero)
        {
            try { msg = Marshal.PtrToStringAnsi(msgPtr) ?? ""; }
            finally { NativeMethods.devcycle_free_string(msgPtr); }
        }
        return (code, string.IsNullOrWhiteSpace(msg) ? "No detailed error available" : msg);
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
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to initialize event queue (code {(int)code}): {msg}");
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
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to set config (code {(int)code}): {msg}");
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
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to set client custom data (code {(int)code}): {msg}");
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
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to set platform data (code {(int)code}): {msg}");
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
                initializeEventQueue ? IntPtr.Zero : IntPtr.Zero,
                customDataPtr == IntPtr.Zero ? IntPtr.Zero : customDataPtr,
                platformDataPtr);

            LogDebug($"Native function returned: {result}");

            if (result != 0)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to initialize SDK key (code {(int)code}): {msg}");
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
    /// Queue a manual event for the provided user (raw user JSON passed). Requires event queue initialized.
    /// </summary>
    public void QueueEvent(DevCycleUser user, string eventType, string? customType = null, string? target = null, double value = 0.0, Dictionary<string, object>? metaData = null)
    {
        if (user == null) throw new ArgumentNullException(nameof(user));
        if (string.IsNullOrWhiteSpace(eventType)) throw new ArgumentNullException(nameof(eventType));
        var userJson = System.Text.Json.JsonSerializer.Serialize(user);
        var metaJson = metaData != null ? System.Text.Json.JsonSerializer.Serialize(metaData) : null;
        IntPtr sdkKeyPtr = IntPtr.Zero; IntPtr userPtr = IntPtr.Zero; IntPtr eventTypePtr = IntPtr.Zero; IntPtr customTypePtr = IntPtr.Zero; IntPtr targetPtr = IntPtr.Zero; IntPtr metaPtr = IntPtr.Zero;
        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            userPtr = Marshal.StringToHGlobalAnsi(userJson);
            eventTypePtr = Marshal.StringToHGlobalAnsi(eventType);
            if (!string.IsNullOrEmpty(customType)) customTypePtr = Marshal.StringToHGlobalAnsi(customType);
            if (!string.IsNullOrEmpty(target)) targetPtr = Marshal.StringToHGlobalAnsi(target);
            if (metaJson != null) metaPtr = Marshal.StringToHGlobalAnsi(metaJson);
            var rc = NativeMethods.devcycle_queue_event(sdkKeyPtr, userPtr, eventTypePtr, customTypePtr, targetPtr, value, metaPtr);
            if (rc != 0)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to queue event (code {(int)code}): {msg}");
            }
            LogDebug($"Queued event '{eventType}' for user {user.UserId}");
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero) Marshal.FreeHGlobal(sdkKeyPtr);
            if (userPtr != IntPtr.Zero) Marshal.FreeHGlobal(userPtr);
            if (eventTypePtr != IntPtr.Zero) Marshal.FreeHGlobal(eventTypePtr);
            if (customTypePtr != IntPtr.Zero) Marshal.FreeHGlobal(customTypePtr);
            if (targetPtr != IntPtr.Zero) Marshal.FreeHGlobal(targetPtr);
            if (metaPtr != IntPtr.Zero) Marshal.FreeHGlobal(metaPtr);
        }
    }

    /// <summary>
    /// Generate bucketed config for the currently set user (preferred). Falls back to ephemeral if no user set.
    /// </summary>
    public string GenerateBucketedConfig(DevCycleUser user)
    {
        if (user == null) throw new ArgumentNullException(nameof(user));
        // Use the FFI function which takes a CUser directly; Rust side handles internal custom/platform data.
        IntPtr sdkKeyPtr = IntPtr.Zero; IntPtr userJsonPtr = IntPtr.Zero; IntPtr cUserPtr = IntPtr.Zero; IntPtr bucketedPtr = IntPtr.Zero; IntPtr jsonPtr = IntPtr.Zero;
        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            var userJson = System.Text.Json.JsonSerializer.Serialize(user);
            userJsonPtr = Marshal.StringToHGlobalAnsi(userJson);
            cUserPtr = NativeMethods.devcycle_user_from_json(userJsonPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to create user (code {(int)code}): {msg}");
            }
            bucketedPtr = NativeMethods.devcycle_generate_bucketed_config_from_user(sdkKeyPtr, cUserPtr);
            if (bucketedPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to generate bucketed config (code {(int)code}): {msg}");
            }
            jsonPtr = NativeMethods.devcycle_bucketed_config_to_json(bucketedPtr);
            if (jsonPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                return $"Config generated but JSON serialization failed (code {(int)code}): {msg}";
            }
            var raw = Marshal.PtrToStringAnsi(jsonPtr) ?? string.Empty;
            try { var doc = System.Text.Json.JsonDocument.Parse(raw); return System.Text.Json.JsonSerializer.Serialize(doc.RootElement, new System.Text.Json.JsonSerializerOptions { WriteIndented = true }); }
            catch { return raw; }
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero) Marshal.FreeHGlobal(sdkKeyPtr);
            if (userJsonPtr != IntPtr.Zero) Marshal.FreeHGlobal(userJsonPtr);
            if (jsonPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(jsonPtr);
            if (bucketedPtr != IntPtr.Zero) NativeMethods.devcycle_free_bucketed_config(bucketedPtr);
            if (cUserPtr != IntPtr.Zero) NativeMethods.devcycle_free_user(cUserPtr);
        }
    }

    /// <summary>
    /// Generate bucketed config for the currently set user (preferred). Falls back to ephemeral if no user set.
    /// </summary>
    public string GenerateBucketedConfigRaw(DevCycleUser user)
    {
        if (user == null) throw new ArgumentNullException(nameof(user));
        IntPtr sdkKeyPtr = IntPtr.Zero; IntPtr userJsonPtr = IntPtr.Zero; IntPtr cUserPtr = IntPtr.Zero; IntPtr bucketedPtr = IntPtr.Zero; IntPtr jsonPtr = IntPtr.Zero;
        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            var userJson = System.Text.Json.JsonSerializer.Serialize(user);
            userJsonPtr = Marshal.StringToHGlobalAnsi(userJson);
            cUserPtr = NativeMethods.devcycle_user_from_json(userJsonPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                return $"error:create_user:{(int)code}:{msg}";
            }
            bucketedPtr = NativeMethods.devcycle_generate_bucketed_config_from_user(sdkKeyPtr, cUserPtr);
            if (bucketedPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                return $"error:bucket_config:{(int)code}:{msg}";
            }
            jsonPtr = NativeMethods.devcycle_bucketed_config_to_json(bucketedPtr);
            if (jsonPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                return $"error:serialize_config:{(int)code}:{msg}";
            }
            return Marshal.PtrToStringAnsi(jsonPtr) ?? string.Empty;
        }
        finally
        {
            if (sdkKeyPtr != IntPtr.Zero) Marshal.FreeHGlobal(sdkKeyPtr);
            if (userJsonPtr != IntPtr.Zero) Marshal.FreeHGlobal(userJsonPtr);
            if (jsonPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(jsonPtr);
            if (bucketedPtr != IntPtr.Zero) NativeMethods.devcycle_free_bucketed_config(bucketedPtr);
            if (cUserPtr != IntPtr.Zero) NativeMethods.devcycle_free_user(cUserPtr);
        }
    }

    /// <summary>
    /// Full variable evaluation returning metadata and error details if any.
    /// </summary>
    public VariableFullResult GetVariableForUser(DevCycleUser user, string variableKey, string variableType)
    {
        if (user == null) throw new ArgumentNullException(nameof(user));
        if (string.IsNullOrWhiteSpace(variableKey)) throw new ArgumentNullException(nameof(variableKey));
        if (string.IsNullOrWhiteSpace(variableType)) throw new ArgumentNullException(nameof(variableType));
        string normType = NormalizeVariableType(variableType);
        LogDebug($"Variable evaluation start: key='{variableKey}', requestedType='{variableType}', normalizedType='{normType}'");
        IntPtr sdkKeyPtr = IntPtr.Zero; IntPtr userJsonPtr = IntPtr.Zero; IntPtr cUserPtr = IntPtr.Zero; IntPtr populatedPtr = IntPtr.Zero; IntPtr keyPtr = IntPtr.Zero; IntPtr typePtr = IntPtr.Zero; IntPtr resultPtr = IntPtr.Zero;
        IntPtr valPtr = IntPtr.Zero; IntPtr typeResPtr = IntPtr.Zero; IntPtr featurePtr = IntPtr.Zero; IntPtr variationPtr = IntPtr.Zero; IntPtr evalPtr = IntPtr.Zero; IntPtr errPtr = IntPtr.Zero; IntPtr fullPtr = IntPtr.Zero;
        var variableResult = new VariableFullResult();
        try
        {
            sdkKeyPtr = Marshal.StringToHGlobalAnsi(_sdkKey);
            keyPtr = Marshal.StringToHGlobalAnsi(variableKey);
            typePtr = Marshal.StringToHGlobalAnsi(normType);
            var userJson = System.Text.Json.JsonSerializer.Serialize(user);
            userJsonPtr = Marshal.StringToHGlobalAnsi(userJson);
            cUserPtr = NativeMethods.devcycle_user_from_json(userJsonPtr);
            if (cUserPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to create user (code {(int)code}): {msg}");
            }
            populatedPtr = NativeMethods.devcycle_populate_user(sdkKeyPtr, cUserPtr);
            if (populatedPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                throw new DevCycleException($"Failed to populate user (code {(int)code}): {msg}");
            }
            resultPtr = NativeMethods.devcycle_variable_for_user(sdkKeyPtr, populatedPtr, keyPtr, typePtr);
            if (resultPtr == IntPtr.Zero)
            {
                var (code, msg) = GetLastErrorDetails();
                LogDebug($"Native variable_for_user returned null. code={(int)code} message='{msg}'");
                throw new DevCycleException($"Variable evaluation failed (code {(int)code}): {msg}");
            }
            fullPtr = NativeMethods.devcycle_variable_result_to_full_json(resultPtr);
            if (fullPtr != IntPtr.Zero)
            {
                variableResult.FullJson = Marshal.PtrToStringAnsi(fullPtr) ?? string.Empty;
            }
            valPtr = NativeMethods.devcycle_variable_result_to_json(resultPtr);
            variableResult.IsError = NativeMethods.devcycle_variable_result_is_error(resultPtr) == 1;
            typeResPtr = NativeMethods.devcycle_variable_result_get_type(resultPtr);
            featurePtr = NativeMethods.devcycle_variable_result_get_feature_id(resultPtr);
            variationPtr = NativeMethods.devcycle_variable_result_get_variation_id(resultPtr);
            evalPtr = NativeMethods.devcycle_variable_result_get_evaluation_reason(resultPtr);
            if (variableResult.IsError) errPtr = NativeMethods.devcycle_variable_result_get_error(resultPtr);
            variableResult.RawValueJson = valPtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(valPtr) ?? "null" : "null";
            variableResult.VariableType = typeResPtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(typeResPtr) ?? "unknown" : "unknown";
            variableResult.FeatureId = featurePtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(featurePtr) ?? string.Empty : string.Empty;
            variableResult.VariationId = variationPtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(variationPtr) ?? string.Empty : string.Empty;
            variableResult.EvaluationReason = evalPtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(evalPtr) ?? string.Empty : string.Empty;
            if (variableResult.IsError) variableResult.ErrorMessage = errPtr != IntPtr.Zero ? Marshal.PtrToStringAnsi(errPtr) ?? string.Empty : string.Empty;
            LogDebug($"Variable evaluation complete. isError={variableResult.IsError} featureId='{variableResult.FeatureId}' variationId='{variableResult.VariationId}' reason='{variableResult.EvaluationReason}' valueJson='{variableResult.RawValueJson}' error='{variableResult.ErrorMessage}'");
            return variableResult;
        }
        finally
        {
            if (valPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(valPtr);
            if (typeResPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(typeResPtr);
            if (featurePtr != IntPtr.Zero) NativeMethods.devcycle_free_string(featurePtr);
            if (variationPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(variationPtr);
            if (evalPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(evalPtr);
            if (errPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(errPtr);
            if (fullPtr != IntPtr.Zero) NativeMethods.devcycle_free_string(fullPtr);
            if (resultPtr != IntPtr.Zero) NativeMethods.devcycle_free_variable_result(resultPtr);
            if (populatedPtr != IntPtr.Zero) NativeMethods.devcycle_free_populated_user(populatedPtr);
            if (cUserPtr != IntPtr.Zero) NativeMethods.devcycle_free_user(cUserPtr);
            if (sdkKeyPtr != IntPtr.Zero) Marshal.FreeHGlobal(sdkKeyPtr);
            if (userJsonPtr != IntPtr.Zero) Marshal.FreeHGlobal(userJsonPtr);
            if (keyPtr != IntPtr.Zero) Marshal.FreeHGlobal(keyPtr);
            if (typePtr != IntPtr.Zero) Marshal.FreeHGlobal(typePtr);
        }
    }

    private static string NormalizeVariableType(string variableType)
    {
        if (string.IsNullOrWhiteSpace(variableType)) return variableType;
        var t = variableType.Trim().ToLowerInvariant();
        return t switch
        {
            "string" => "String",
            "number" => "Number",
            "int" => "Number",
            "integer" => "Number",
            "float" => "Number",
            "double" => "Number",
            "bool" => "Boolean",
            "boolean" => "Boolean",
            "json" => "JSON",
            _ => variableType // leave as-is for custom types
        };
    }

    private string _lastUserId = string.Empty;

    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    protected virtual void Dispose(bool disposing)
    {
        if (!_disposed)
        {
            // No persistent native user resources to free now.
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

/// <summary>
/// Represents the result of a variable evaluation, including metadata and error details
/// </summary>
public class VariableFullResult
{
    public string RawValueJson { get; set; } = string.Empty; // JSON-encoded value
    public string VariableType { get; set; } = string.Empty;
    public string FeatureId { get; set; } = string.Empty;
    public string VariationId { get; set; } = string.Empty;
    public string EvaluationReason { get; set; } = string.Empty;
    public bool IsError { get; set; }
    public string ErrorMessage { get; set; } = string.Empty;
    public string FullJson { get; set; } = string.Empty; // optional full JSON structure
    public override string ToString() => IsError ? $"ERROR({ErrorMessage})" : RawValueJson;
}
