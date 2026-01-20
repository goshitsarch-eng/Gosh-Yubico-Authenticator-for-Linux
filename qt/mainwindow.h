#pragma once

#include <QListWidget>
#include <QMainWindow>
#include <QStackedWidget>
#include <QTimer>
#include <QLabel>

#include "credential_row_widget.h"
#include "gosh_client.h"
#include "toast_overlay.h"

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow() override;

protected:
    void resizeEvent(QResizeEvent* event) override;

private:
    void setupUi();
    void setupConnections();
    void tryConnect();
    void refresh();
    void applyCredentials(const QVector<Credential>& credentials);
    void updateCountdown();
    void showToast(const QString& message);
    void showLoading(const QString& message);
    void showNoDevice();
    void showNoCredentials();
    void showCredentials();
    void showTouchRequired();
    void showAddDialog();
    void showPasswordDialog();
    void confirmDelete(const QByteArray& id, const QString& name);
    void updateRowCode(const QByteArray& id, const QString& code, uint8_t digits);

    QWidget* central = nullptr;
    QStackedWidget* stack = nullptr;
    QWidget* loadingPage = nullptr;
    QWidget* noDevicePage = nullptr;
    QWidget* noCredentialsPage = nullptr;
    QWidget* credentialsPage = nullptr;
    QWidget* touchPage = nullptr;
    QLabel* loadingLabel = nullptr;
    QListWidget* listWidget = nullptr;
    QPushButton* refreshButton = nullptr;
    QPushButton* addButton = nullptr;
    QPushButton* addFirstButton = nullptr;
    ToastOverlay* toast = nullptr;

    GoshClientWrapper* client = nullptr;
    QTimer countdownTimer;
    qint64 lastPeriodCounter = -1;
    bool connected = false;
};
