// SPDX-License-Identifier: GPL-3.0-or-later

#include "version.h"

#include <QApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QUrl>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    QCoreApplication::setOrganizationName(QStringLiteral("goshapps"));
    QCoreApplication::setOrganizationDomain(QStringLiteral("goshapps.com"));
    QCoreApplication::setApplicationName(QStringLiteral("gosh-authenticator"));
    QCoreApplication::setApplicationVersion(QStringLiteral(APP_VERSION_STRING));
    QGuiApplication::setApplicationDisplayName(QStringLiteral(APP_DISPLAY_NAME));
    QGuiApplication::setDesktopFileName(QStringLiteral(APP_ID));
    QGuiApplication::setWindowIcon(QIcon::fromTheme(QStringLiteral(APP_ID)));

    // Use the KDE QQC2 style for a native look; users can still override
    // through QT_QUICK_CONTROLS_STYLE.
    if (qEnvironmentVariableIsEmpty("QT_QUICK_CONTROLS_STYLE")) {
        QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));
    }

    QQmlApplicationEngine engine;
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app,
                     []() { QCoreApplication::exit(1); },
                     Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral(
        "qrc:/qt/qml/com/goshapps/yubicoauthenticator/src/qml/Main.qml")));
    if (engine.rootObjects().isEmpty()) {
        return 1;
    }

    return app.exec();
}
