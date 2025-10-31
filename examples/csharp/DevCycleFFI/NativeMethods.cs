using System.Runtime.InteropServices;

namespace DevCycleFFI;

/// <summary>
/// Native P/Invoke declarations for the DevCycle Rust FFI library
/// </summary>
internal static class NativeMethods
{
    // Library name - adjust based on platform
    private const string LibName = "devcycle_bucketing_rs";

    // Opaque pointer types
    internal struct CBucketedUserConfig { }
    internal struct CUser { }
    internal struct CPopulatedUser { }
    internal struct CEventQueueOptions { }
    internal struct CVariableForUserResult { }

    /// <summary>
    /// Initialize event queue
    /// </summary>
    /// <returns>0 on success, non-zero on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int devcycle_init_event_queue(
        IntPtr sdk_key,
        IntPtr options);

    /// <summary>
    /// Set config from JSON string
    /// </summary>
    /// <returns>0 on success, non-zero on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int devcycle_set_config(
        IntPtr sdk_key,
        IntPtr config_json);

    /// <summary>
    /// Set client custom data from JSON string
    /// </summary>
    /// <returns>0 on success, non-zero on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int devcycle_set_client_custom_data(
        IntPtr sdk_key,
        IntPtr custom_data_json);

    /// <summary>
    /// Set platform data from JSON string
    /// </summary>
    /// <returns>0 on success, non-zero on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int devcycle_set_platform_data(
        IntPtr sdk_key,
        IntPtr platform_data_json);

    /// <summary>
    /// Initialize SDK key with all required data in one call
    /// </summary>
    /// <returns>0 on success, non-zero on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int devcycle_init_sdk_key(
        IntPtr sdk_key,
        IntPtr config_json,
        IntPtr event_queue_options,
        IntPtr client_custom_data_json,
        IntPtr platform_data_json);

    /// <summary>
    /// Generate bucketed config from populated user
    /// </summary>
    /// <returns>Pointer to CBucketedUserConfig on success, null on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_generate_bucketed_config(
        IntPtr sdk_key,
        IntPtr user,
        IntPtr client_custom_data_json);

    /// <summary>
    /// Generate bucketed config from user
    /// </summary>
    /// <returns>Pointer to CBucketedUserConfig on success, null on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_generate_bucketed_config_from_user(
        IntPtr sdk_key,
        IntPtr user);

    /// <summary>
    /// Get variable value for a user
    /// </summary>
    /// <returns>Pointer to CVariableForUserResult on success, null on error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_variable_for_user(
        IntPtr sdk_key,
        IntPtr user,
        IntPtr variable_key,
        IntPtr variable_type);

    /// <summary>
    /// Get JSON representation of variable for user result
    /// </summary>
    /// <returns>C string that must be freed with devcycle_free_string</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_variable_result_to_json(IntPtr result);

    /// <summary>
    /// Get variable type from result
    /// </summary>
    /// <returns>C string that must be freed with devcycle_free_string</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_variable_result_get_type(IntPtr result);

    /// <summary>
    /// Get JSON representation of bucketed config
    /// </summary>
    /// <returns>C string that must be freed with devcycle_free_string</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_bucketed_config_to_json(IntPtr config);

    /// <summary>
    /// Get the last error message from FFI operations
    /// </summary>
    /// <returns>C string that must be freed with devcycle_free_string, or null if no error</returns>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_get_last_error();

    /// <summary>
    /// Create user from JSON string
    /// </summary>
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr devcycle_user_from_json(IntPtr json);

    // Memory management functions
    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_string(IntPtr s);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_bucketed_config(IntPtr config);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_user(IntPtr user);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_populated_user(IntPtr user);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_event_queue_options(IntPtr options);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void devcycle_free_variable_result(IntPtr result);
}
