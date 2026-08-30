// SPDX-License-Identifier: GPL-3.0-or-later

#include "thememanager.h"

#include <QGuiApplication>

#if defined(HAVE_KCOLORSCHEME)
#include <KColorSchemeManager>
#include <QModelIndex>
#else
#include <QPalette>
#include <QStyleHints>
#endif

namespace thememanager {

#if defined(HAVE_KCOLORSCHEME)

void applyTheme(int mode)
{
    KColorSchemeManager *manager = KColorSchemeManager::instance();
    switch (mode) {
    case 1: {
        QModelIndex index = manager->indexForScheme(QStringLiteral("Breeze Light"));
        if (!index.isValid()) {
            index = manager->indexForScheme(QStringLiteral("Breeze"));
        }
        manager->activateScheme(index);
        break;
    }
    case 2:
        manager->activateScheme(manager->indexForScheme(QStringLiteral("Breeze Dark")));
        break;
    default:
        // Invalid index re-activates the system color scheme.
        manager->activateScheme(QModelIndex());
        break;
    }
}

#else // !HAVE_KCOLORSCHEME

namespace {

QPalette buildPalette(bool dark)
{
    // Breeze-like colors so forced light/dark still looks at home next to
    // KDE applications when KColorScheme is unavailable at build time.
    QPalette palette;
    if (dark) {
        const QColor window(0x2a, 0x2e, 0x32);
        const QColor base(0x1b, 0x1e, 0x20);
        const QColor alternate(0x23, 0x26, 0x29);
        const QColor text(0xfc, 0xfc, 0xfc);
        const QColor button(0x31, 0x36, 0x3b);
        const QColor highlight(0x3d, 0xae, 0xe9);
        const QColor disabled(0x6e, 0x71, 0x73);
        palette.setColor(QPalette::Window, window);
        palette.setColor(QPalette::WindowText, text);
        palette.setColor(QPalette::Base, base);
        palette.setColor(QPalette::AlternateBase, alternate);
        palette.setColor(QPalette::ToolTipBase, window);
        palette.setColor(QPalette::ToolTipText, text);
        palette.setColor(QPalette::Text, text);
        palette.setColor(QPalette::Button, button);
        palette.setColor(QPalette::ButtonText, text);
        palette.setColor(QPalette::Link, highlight);
        palette.setColor(QPalette::Highlight, highlight);
        palette.setColor(QPalette::HighlightedText, QColor(0xfc, 0xfc, 0xfc));
        palette.setColor(QPalette::PlaceholderText, disabled);
        palette.setColor(QPalette::Disabled, QPalette::Text, disabled);
        palette.setColor(QPalette::Disabled, QPalette::WindowText, disabled);
        palette.setColor(QPalette::Disabled, QPalette::ButtonText, disabled);
    } else {
        const QColor window(0xef, 0xf0, 0xf1);
        const QColor base(0xff, 0xff, 0xff);
        const QColor alternate(0xf7, 0xf7, 0xf7);
        const QColor text(0x23, 0x26, 0x29);
        const QColor button(0xfc, 0xfc, 0xfc);
        const QColor highlight(0x3d, 0xae, 0xe9);
        const QColor disabled(0xa0, 0xa2, 0xa4);
        palette.setColor(QPalette::Window, window);
        palette.setColor(QPalette::WindowText, text);
        palette.setColor(QPalette::Base, base);
        palette.setColor(QPalette::AlternateBase, alternate);
        palette.setColor(QPalette::ToolTipBase, base);
        palette.setColor(QPalette::ToolTipText, text);
        palette.setColor(QPalette::Text, text);
        palette.setColor(QPalette::Button, button);
        palette.setColor(QPalette::ButtonText, text);
        palette.setColor(QPalette::Link, highlight);
        palette.setColor(QPalette::Highlight, highlight);
        palette.setColor(QPalette::HighlightedText, QColor(0xff, 0xff, 0xff));
        palette.setColor(QPalette::PlaceholderText, disabled);
        palette.setColor(QPalette::Disabled, QPalette::Text, disabled);
        palette.setColor(QPalette::Disabled, QPalette::WindowText, disabled);
        palette.setColor(QPalette::Disabled, QPalette::ButtonText, disabled);
    }
    return palette;
}

} // namespace

void applyTheme(int mode)
{
#if QT_VERSION >= QT_VERSION_CHECK(6, 8, 0)
    switch (mode) {
    case 1:
        QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Light);
        return;
    case 2:
        QGuiApplication::styleHints()->setColorScheme(Qt::ColorScheme::Dark);
        return;
    default:
        QGuiApplication::styleHints()->unsetColorScheme();
        return;
    }
#else
    switch (mode) {
    case 1:
        QGuiApplication::setPalette(buildPalette(false));
        break;
    case 2:
        QGuiApplication::setPalette(buildPalette(true));
        break;
    default:
        QGuiApplication::setPalette(QPalette());
        break;
    }
#endif
}

#endif // HAVE_KCOLORSCHEME

} // namespace thememanager
