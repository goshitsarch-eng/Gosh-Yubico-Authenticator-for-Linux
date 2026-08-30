// SPDX-License-Identifier: GPL-3.0-or-later
//
// Light/dark/system color-scheme switching.

#pragma once

namespace thememanager {

/// Apply a theme mode: 0 = follow system, 1 = force light, 2 = force dark.
///
/// On a full KDE stack this activates the matching Breeze color scheme via
/// KColorSchemeManager, so the app behaves exactly like other KDE
/// applications. Without KColorScheme it falls back to Qt's color-scheme
/// hint (Qt >= 6.8) or a built-in Breeze-like palette.
void applyTheme(int mode);

} // namespace thememanager
