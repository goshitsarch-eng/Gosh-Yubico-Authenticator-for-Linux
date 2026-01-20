#include "mainwindow.h"

#include <QApplication>
#include <QClipboard>
#include <QDateTime>
#include <QHBoxLayout>
#include <QLabel>
#include <QMenu>
#include <QMessageBox>
#include <QToolButton>
#include <QVBoxLayout>

#include "add_credential_dialog.h"
#include "password_dialog.h"

#ifndef VERSION_STRING
#define VERSION_STRING "0.1.0"
#endif

MainWindow::MainWindow(QWidget* parent)
    : QMainWindow(parent) {
    setupUi();
    setupConnections();

    countdownTimer.setInterval(100);
    connect(&countdownTimer, &QTimer::timeout, this, &MainWindow::updateCountdown);
    countdownTimer.start();

    tryConnect();
}

MainWindow::~MainWindow() = default;

void MainWindow::resizeEvent(QResizeEvent* event) {
    QMainWindow::resizeEvent(event);
    if (toast && central) {
        toast->setGeometry(central->rect());
    }
}

void MainWindow::setupUi() {
    setWindowTitle(tr("Gosh Authenticator"));
    resize(720, 520);

    central = new QWidget(this);
    auto* layout = new QVBoxLayout(central);

    refreshButton = new QPushButton(tr("Refresh"), this);
    addButton = new QPushButton(tr("Add"), this);
    addButton->setEnabled(false);

    auto* menuButton = new QToolButton(this);
    menuButton->setText(tr("Menu"));
    menuButton->setPopupMode(QToolButton::InstantPopup);
    auto* menu = new QMenu(menuButton);
    auto* aboutAction = menu->addAction(tr("About"));
    auto* quitAction = menu->addAction(tr("Quit"));
    menuButton->setMenu(menu);

    connect(aboutAction, &QAction::triggered, this, [this]() {
        QMessageBox::about(this,
                           tr("About Gosh Authenticator"),
                           tr("Manage OATH credentials on YubiKey devices.\nVersion %1")
                               .arg(QStringLiteral(VERSION_STRING)));
    });
    connect(quitAction, &QAction::triggered, qApp, &QApplication::quit);

    auto* header = new QWidget(this);
    auto* headerLayout = new QHBoxLayout(header);
    headerLayout->addWidget(refreshButton);
    headerLayout->addWidget(addButton);
    headerLayout->addStretch(1);
    headerLayout->addWidget(menuButton);
    headerLayout->setContentsMargins(0, 0, 0, 0);

    stack = new QStackedWidget(this);

    loadingPage = new QWidget(this);
    auto* loadingLayout = new QVBoxLayout(loadingPage);
    loadingLabel = new QLabel(tr("Connecting to YubiKey..."), loadingPage);
    loadingLabel->setAlignment(Qt::AlignCenter);
    loadingLayout->addStretch(1);
    loadingLayout->addWidget(loadingLabel);
    loadingLayout->addStretch(1);

    noDevicePage = new QWidget(this);
    auto* noDeviceLayout = new QVBoxLayout(noDevicePage);
    auto* noDeviceLabel = new QLabel(tr("No YubiKey connected."), noDevicePage);
    noDeviceLabel->setAlignment(Qt::AlignCenter);
    auto* connectButton = new QPushButton(tr("Connect"), noDevicePage);
    connect(connectButton, &QPushButton::clicked, this, &MainWindow::tryConnect);
    noDeviceLayout->addStretch(1);
    noDeviceLayout->addWidget(noDeviceLabel);
    noDeviceLayout->addWidget(connectButton, 0, Qt::AlignCenter);
    noDeviceLayout->addStretch(1);

    noCredentialsPage = new QWidget(this);
    auto* noCredentialsLayout = new QVBoxLayout(noCredentialsPage);
    auto* noCredentialsLabel = new QLabel(tr("No credentials found."), noCredentialsPage);
    noCredentialsLabel->setAlignment(Qt::AlignCenter);
    addFirstButton = new QPushButton(tr("Add credential"), noCredentialsPage);
    noCredentialsLayout->addStretch(1);
    noCredentialsLayout->addWidget(noCredentialsLabel);
    noCredentialsLayout->addWidget(addFirstButton, 0, Qt::AlignCenter);
    noCredentialsLayout->addStretch(1);

    credentialsPage = new QWidget(this);
    auto* credentialsLayout = new QVBoxLayout(credentialsPage);
    listWidget = new QListWidget(credentialsPage);
    listWidget->setSelectionMode(QAbstractItemView::NoSelection);
    listWidget->setSpacing(6);
    credentialsLayout->addWidget(listWidget);

    touchPage = new QWidget(this);
    auto* touchLayout = new QVBoxLayout(touchPage);
    auto* touchLabel = new QLabel(tr("Touch your YubiKey to continue."), touchPage);
    touchLabel->setAlignment(Qt::AlignCenter);
    touchLayout->addStretch(1);
    touchLayout->addWidget(touchLabel);
    touchLayout->addStretch(1);

    stack->addWidget(loadingPage);
    stack->addWidget(noDevicePage);
    stack->addWidget(noCredentialsPage);
    stack->addWidget(credentialsPage);
    stack->addWidget(touchPage);

    layout->addWidget(header);
    layout->addWidget(stack, 1);
    central->setLayout(layout);
    setCentralWidget(central);

    toast = new ToastOverlay(central);
    toast->setGeometry(central->rect());
    toast->raise();

    client = new GoshClientWrapper(this);
}

void MainWindow::setupConnections() {
    connect(refreshButton, &QPushButton::clicked, this, [this]() {
        if (connected) {
            refresh();
        } else {
            tryConnect();
        }
    });
    connect(addButton, &QPushButton::clicked, this, &MainWindow::showAddDialog);
    connect(addFirstButton, &QPushButton::clicked, this, &MainWindow::showAddDialog);

    connect(client, &GoshClientWrapper::connected, this, [this](const QString&, const QByteArray&) {
        connected = true;
        addButton->setEnabled(true);
    });
    connect(client, &GoshClientWrapper::disconnected, this, [this]() {
        connected = false;
        addButton->setEnabled(false);
        listWidget->clear();
        showNoDevice();
    });
    connect(client, &GoshClientWrapper::authenticationRequired, this, [this]() {
        showPasswordDialog();
    });
    connect(client, &GoshClientWrapper::authenticationSuccessful, this, [this]() {
        showToast(tr("Unlocked successfully"));
    });
    connect(client, &GoshClientWrapper::authenticationFailed, this, [this](const QString& msg) {
        showToast(tr("Authentication failed: %1").arg(msg));
        showPasswordDialog();
    });
    connect(client, &GoshClientWrapper::credentialsUpdated, this, [this](const QVector<Credential>& creds) {
        applyCredentials(creds);
    });
    connect(client, &GoshClientWrapper::credentialCalculated, this, [this](const QByteArray& id,
                                                                          const QString& code,
                                                                          uint8_t digits) {
        Q_UNUSED(digits);
        updateRowCode(id, code, digits);
    });
    connect(client, &GoshClientWrapper::touchRequired, this, [this](const QByteArray&) {
        showTouchRequired();
        showToast(tr("Touch your YubiKey"));
    });
    connect(client, &GoshClientWrapper::credentialAdded, this, [this](const Credential& cred) {
        showToast(tr("Added %1").arg(cred.displayName()));
    });
    connect(client, &GoshClientWrapper::credentialDeleted, this, [this](const QByteArray&) {
        showToast(tr("Credential deleted"));
    });
    connect(client, &GoshClientWrapper::error, this, [this](const QString& msg) {
        const QString lower = msg.toLower();
        if (lower.contains("resource manager") || lower.contains("not running")) {
            showToast(tr("PC/SC service not running. Start pcscd."));
        } else if (lower.contains("no yubikey") || lower.contains("no device")) {
            // No toast
        } else {
            showToast(tr("Error: %1").arg(msg));
        }
        showNoDevice();
    });
}

void MainWindow::tryConnect() {
    if (!connected) {
        showLoading(tr("Connecting to YubiKey..."));
        client->connectDevice();
    }
}

void MainWindow::refresh() {
    if (connected) {
        client->refresh();
    }
}

void MainWindow::applyCredentials(const QVector<Credential>& credentials) {
    listWidget->clear();
    if (credentials.isEmpty()) {
        showNoCredentials();
        return;
    }

    for (const auto& credential : credentials) {
        auto* item = new QListWidgetItem(listWidget);
        auto* row = new CredentialRowWidget(listWidget);
        row->setCredential(credential);
        item->setSizeHint(row->sizeHint());

        connect(row, &CredentialRowWidget::copyRequested, this, [this, row](const QByteArray& id,
                                                                           const QString& code) {
            if (code.trimmed().isEmpty()) {
                return;
            }
            QClipboard* clipboard = QApplication::clipboard();
            clipboard->setText(code.simplified().remove(' '));
            showToast(tr("Copied code for %1").arg(row->credential().displayName()));
        });
        connect(row, &CredentialRowWidget::refreshRequested, this, [this](const QByteArray& id) {
            client->calculate(id);
        });
        connect(row, &CredentialRowWidget::deleteRequested, this, [this, row](const QByteArray& id) {
            confirmDelete(id, row->credential().displayName());
        });

        listWidget->addItem(item);
        listWidget->setItemWidget(item, row);
    }

    showCredentials();
}

void MainWindow::updateCountdown() {
    const qint64 nowMs = QDateTime::currentMSecsSinceEpoch();
    const qint64 periodMs = 30000;
    const qint64 counter = nowMs / periodMs;
    if (counter != lastPeriodCounter) {
        lastPeriodCounter = counter;
        refresh();
    }

    for (int i = 0; i < listWidget->count(); ++i) {
        auto* item = listWidget->item(i);
        if (auto* row = qobject_cast<CredentialRowWidget*>(listWidget->itemWidget(item))) {
            row->updateCountdown(nowMs);
        }
    }
}

void MainWindow::showToast(const QString& message) {
    if (toast) {
        toast->showMessage(message);
    }
}

void MainWindow::showLoading(const QString& message) {
    loadingLabel->setText(message);
    stack->setCurrentWidget(loadingPage);
}

void MainWindow::showNoDevice() {
    stack->setCurrentWidget(noDevicePage);
}

void MainWindow::showNoCredentials() {
    stack->setCurrentWidget(noCredentialsPage);
}

void MainWindow::showCredentials() {
    stack->setCurrentWidget(credentialsPage);
}

void MainWindow::showTouchRequired() {
    stack->setCurrentWidget(touchPage);
}

void MainWindow::showAddDialog() {
    AddCredentialDialog dialog(this);
    if (dialog.exec() != QDialog::Accepted) {
        return;
    }
    CredentialInput input;
    QString error;
    if (!dialog.collectInput(&input, &error)) {
        showToast(error);
        return;
    }

    if (!client->addCredential(input.issuer,
                               input.account,
                               input.secret,
                               input.oathType,
                               input.algorithm,
                               input.digits,
                               input.requireTouch,
                               input.hasInitialCounter,
                               input.initialCounter)) {
        const QString err = client->takeLastError();
        if (!err.isEmpty()) {
            showToast(err);
        }
    }
}

void MainWindow::showPasswordDialog() {
    PasswordDialog dialog(this);
    if (dialog.exec() == QDialog::Accepted) {
        client->authenticate(dialog.password());
    } else {
        showNoDevice();
    }
}

void MainWindow::confirmDelete(const QByteArray& id, const QString& name) {
    const QString text = tr("Are you sure you want to delete \"%1\"? This cannot be undone.")
                             .arg(name);
    if (QMessageBox::question(this,
                              tr("Delete Credential?"),
                              text,
                              QMessageBox::Yes | QMessageBox::No,
                              QMessageBox::No)
        == QMessageBox::Yes) {
        client->deleteCredential(id);
    }
}

void MainWindow::updateRowCode(const QByteArray& id, const QString& code, uint8_t digits) {
    Q_UNUSED(digits);
    for (int i = 0; i < listWidget->count(); ++i) {
        auto* item = listWidget->item(i);
        if (auto* row = qobject_cast<CredentialRowWidget*>(listWidget->itemWidget(item))) {
            if (row->credential().id == id) {
                Credential updated = row->credential();
                updated.code = code;
                updated.touchRequired = false;
                row->setCredential(updated);
                showCredentials();
                break;
            }
        }
    }
}
