#include "credential_row_widget.h"

#include <QDateTime>
#include <QHBoxLayout>
#include <QVBoxLayout>

CredentialRowWidget::CredentialRowWidget(QWidget* parent)
    : QWidget(parent) {
    nameLabel = new QLabel(this);
    nameLabel->setTextInteractionFlags(Qt::TextSelectableByMouse);

    typeLabel = new QLabel(this);
    typeLabel->setStyleSheet("color: #666;");

    codeLabel = new QLabel(this);
    QFont codeFont = codeLabel->font();
    codeFont.setPointSize(codeFont.pointSize() + 2);
    codeFont.setBold(true);
    codeLabel->setFont(codeFont);
    codeLabel->setTextInteractionFlags(Qt::TextSelectableByMouse);

    progress = new QProgressBar(this);
    progress->setRange(0, 100);
    progress->setTextVisible(true);

    copyButton = new QPushButton(tr("Copy"), this);
    refreshButton = new QPushButton(tr("Refresh"), this);
    deleteButton = new QPushButton(tr("Delete"), this);

    connect(copyButton, &QPushButton::clicked, this, [this]() {
        emit copyRequested(credential_.id, credential_.code);
    });
    connect(refreshButton, &QPushButton::clicked, this, [this]() {
        emit refreshRequested(credential_.id);
    });
    connect(deleteButton, &QPushButton::clicked, this, [this]() {
        emit deleteRequested(credential_.id);
    });

    auto* leftLayout = new QVBoxLayout();
    leftLayout->addWidget(nameLabel);
    leftLayout->addWidget(codeLabel);
    leftLayout->addWidget(progress);

    auto* rightLayout = new QVBoxLayout();
    rightLayout->addWidget(typeLabel, 0, Qt::AlignRight);
    rightLayout->addStretch(1);
    rightLayout->addWidget(copyButton);
    rightLayout->addWidget(refreshButton);
    rightLayout->addWidget(deleteButton);

    auto* layout = new QHBoxLayout(this);
    layout->addLayout(leftLayout, 1);
    layout->addLayout(rightLayout);
    layout->setContentsMargins(8, 8, 8, 8);
    layout->setSpacing(12);
    setLayout(layout);

    updateUi();
}

void CredentialRowWidget::setCredential(const Credential& credential) {
    credential_ = credential;
    updateUi();
}

void CredentialRowWidget::updateCountdown(qint64 nowMs) {
    if (!credential_.isTotp()) {
        progress->setVisible(false);
        return;
    }

    const uint32_t period = credential_.period > 0 ? credential_.period : 30;
    const qint64 periodMs = static_cast<qint64>(period) * 1000;
    const qint64 elapsed = nowMs % periodMs;
    const qint64 remainingMs = periodMs - elapsed;
    const int remainingSeconds = static_cast<int>((remainingMs + 999) / 1000);
    const double ratio = static_cast<double>(remainingMs) / static_cast<double>(periodMs);
    const int value = static_cast<int>(ratio * 100.0);

    progress->setVisible(true);
    progress->setValue(value);
    progress->setFormat(tr("%1s").arg(remainingSeconds));
}

void CredentialRowWidget::updateUi() {
    nameLabel->setText(credential_.displayName());

    if (credential_.code.isEmpty()) {
        codeLabel->setText(credential_.touchRequired ? tr("Touch required") : tr("-"));
    } else {
        codeLabel->setText(credential_.code);
    }

    typeLabel->setText(credential_.isTotp() ? tr("TOTP") : tr("HOTP"));
    refreshButton->setVisible(credential_.isHotp() || credential_.touchRequired);
}
