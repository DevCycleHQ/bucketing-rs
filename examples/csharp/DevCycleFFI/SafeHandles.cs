using System;
using System.Runtime.InteropServices;

namespace DevCycleFFI
{
    // Base pattern: wrap native pointers returned by FFI and ensure the correct free_* function is invoked.
    // Each handle is invalid when IntPtr.Zero.

    internal sealed class CUserHandle : SafeHandle
    {
        public CUserHandle() : base(IntPtr.Zero, ownsHandle: true) { }
        public CUserHandle(IntPtr existing) : this() { SetHandle(existing); }
        public override bool IsInvalid => handle == IntPtr.Zero;
        protected override bool ReleaseHandle()
        {
            if (!IsInvalid) NativeMethods.devcycle_free_user(handle);
            return true;
        }
    }

    internal sealed class CPopulatedUserHandle : SafeHandle
    {
        public CPopulatedUserHandle() : base(IntPtr.Zero, ownsHandle: true) { }
        public CPopulatedUserHandle(IntPtr existing) : this() { SetHandle(existing); }
        public override bool IsInvalid => handle == IntPtr.Zero;
        protected override bool ReleaseHandle()
        {
            if (!IsInvalid) NativeMethods.devcycle_free_populated_user(handle);
            return true;
        }
    }

    internal sealed class CBucketedUserConfigHandle : SafeHandle
    {
        public CBucketedUserConfigHandle() : base(IntPtr.Zero, ownsHandle: true) { }
        public CBucketedUserConfigHandle(IntPtr existing) : this() { SetHandle(existing); }
        public override bool IsInvalid => handle == IntPtr.Zero;
        protected override bool ReleaseHandle()
        {
            if (!IsInvalid) NativeMethods.devcycle_free_bucketed_config(handle);
            return true;
        }
    }

    internal sealed class CVariableForUserResultHandle : SafeHandle
    {
        public CVariableForUserResultHandle() : base(IntPtr.Zero, ownsHandle: true) { }
        public CVariableForUserResultHandle(IntPtr existing) : this() { SetHandle(existing); }
        public override bool IsInvalid => handle == IntPtr.Zero;
        protected override bool ReleaseHandle()
        {
            if (!IsInvalid) NativeMethods.devcycle_free_variable_result(handle);
            return true;
        }
    }

    internal sealed class CEventQueueOptionsHandle : SafeHandle
    {
        public CEventQueueOptionsHandle() : base(IntPtr.Zero, ownsHandle: true) { }
        public CEventQueueOptionsHandle(IntPtr existing) : this() { SetHandle(existing); }
        public override bool IsInvalid => handle == IntPtr.Zero;
        protected override bool ReleaseHandle()
        {
            if (!IsInvalid) NativeMethods.devcycle_free_event_queue_options(handle);
            return true;
        }
    }
}

