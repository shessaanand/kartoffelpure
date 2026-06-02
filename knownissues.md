# Known Issues

## URL Normalization

The address bar URL normalization logic still has edge cases.

For example, malformed input such as:

https:youtube.com

may currently be transformed into:

https://https:youtube.com

instead of:

https://youtube.com

This is a known bug and will be addressed in a future patch release.

---

## WebKitGTK Sandbox

Some Linux environments may fail to launch WebKitGTK's sandbox correctly and display errors similar to:

bwrap: setting up uid map: Permission denied
Failed to fully launch dbus-proxy

Temporary development workaround:

WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 cargo run

This workaround should only be used for development and testing.

---

## Performance With Many Tabs

KartoffelPure currently keeps all tab WebViews alive simultaneously.

Opening a large number of tabs may increase memory usage and reduce responsiveness.

Future releases may introduce:

- Tab suspension
- Background tab sleeping
- Memory pressure handling
- Improved resource management

---

## Current Status

Implemented:

- Browser window
- Multi-tab browsing
- Tab overflow handling
- Persistent browsing history
- History search
- History deletion

Planned:

- Bookmarks
- Downloads
- Settings
- Privacy features
- Performance optimizations
