# Window Resize Rendering Issue

## Summary

On Windows with high-DPI displays, resizing the window causes rendering issues where:
1. **Black space appears** in newly exposed window regions
2. **Content (strokes and UI) shifts to the corner** instead of maintaining correct position
3. The issue is immediately fixed by clicking anywhere on the canvas or moving the window

## Reproduction Steps

1. Launch the application
2. Resize the window larger (e.g., maximize)
3. Observe black space in newly exposed regions OR content shifting to top-left corner
4. Click on canvas or move window - issue is immediately fixed

## Behavior Details

| Action | Result |
|--------|--------|
| Maximize window | Black space appears in new window regions |
| Restore window | Issue may persist |
| Slow resize | May occasionally self-correct |
| Click on canvas | Issue immediately fixed |
| Move window | Issue immediately fixed |
| Hover | Does NOT fix issue |
| Minimize/Restore | Random - usually doesn't fix |

## Environment

- **Platform:** Windows
- **GPU:** Intel (Vulkan backend)
- **DPI Scale:** 175% (observed)
- **wgpu version:** 28.0.0
- **winit version:** 0.30.x

## Root Cause

Likely a combination of:
1. **Windows Desktop Window Manager (DWM) compositor timing issues** - The OS isn't properly syncing WGPU surface textures during rapid resize events
2. **Surface/swapchain state synchronization** - WGPU surface configuration may need specific handling during window size changes

This is a **known issue** in wgpu:
- [wgpu Issue #7836](https://github.com/gfx-rs/wgpu/issues/7836) - "Resizing windows with no decorations does not immediately update window contents"
- [wgpu Issue #3756](https://github.com/gfx-rs/wgpu/issues/3756) - Similar resize-related issues

Both were closed as "not planned" for wgpu to fix.

## Workarounds Attempted

- Viewport setting to match texture dimensions
- Uniform buffer consistency (using actual texture size)
- Surface reconfiguration checks on every frame
- `ControlFlow::Poll` for continuous rendering
- `PresentMode::Immediate` instead of `Fifo`
- Multiple `request_redraw()` calls after resize
- `pre_present_notify()` before presenting
- Surface suboptimal flag handling
- Windows `InvalidateRect`/`UpdateWindow` API calls
- Clamping surface dimensions to GPU texture limits
- Logical-to-physical window size conversion for saved preferences

None of these fully resolved the issue.

## Impact

- **Cosmetic issue only** - Does not affect drawing functionality
- New strokes render correctly once drawn
- Existing content remains correct
- Clicking/moving window immediately corrects the display

## Suggested Next Steps

1. File a detailed bug report with wgpu/winit if this blocks users
2. Investigate winit's window resize handling specifically
3. Try alternative window backends if available
4. Consider using a higher-level game engine (bevy, macroquad) with more robust resize handling
5. Explore creating an intermediate render target to handle resize more gracefully
