#pragma once

#include <QByteArray>
#include <QLabel>
#include <QProgressBar>
#include <QPushButton>
#include <QWidget>

#include "credential_types.h"

class CredentialRowWidget : public QWidget {
    Q_OBJECT

public:
    explicit CredentialRowWidget(QWidget* parent = nullptr);

    void setCredential(const Credential& credential);
    void updateCountdown(qint64 nowMs);
    const Credential& credential() const { return credential_; }

signals:
    void copyRequested(const QByteArray& id, const QString& code);
    void refreshRequested(const QByteArray& id);
    void deleteRequested(const QByteArray& id);

private:
    void updateUi();

    Credential credential_;

    QLabel* nameLabel = nullptr;
    QLabel* codeLabel = nullptr;
    QLabel* typeLabel = nullptr;
    QProgressBar* progress = nullptr;
    QPushButton* copyButton = nullptr;
    QPushButton* refreshButton = nullptr;
    QPushButton* deleteButton = nullptr;
};
