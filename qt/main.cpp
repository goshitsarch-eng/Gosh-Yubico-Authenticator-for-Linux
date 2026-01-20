#include <QApplication>
#include <QIcon>

#include "credential_types.h"
#include "mainwindow.h"

int main(int argc, char* argv[]) {
    QApplication app(argc, argv);
    app.setApplicationName("Gosh Authenticator");
    app.setOrganizationName("Gosh");

    qRegisterMetaType<Credential>("Credential");
    qRegisterMetaType<QVector<Credential>>("QVector<Credential>");

    QIcon appIcon = QIcon::fromTheme("com.github.gosh.authenticator");
    if (appIcon.isNull()) {
        appIcon = QIcon(":/icons/app-icon.svg");
    }
    if (!appIcon.isNull()) {
        app.setWindowIcon(appIcon);
    }

    MainWindow window;
    window.show();
    return app.exec();
}
