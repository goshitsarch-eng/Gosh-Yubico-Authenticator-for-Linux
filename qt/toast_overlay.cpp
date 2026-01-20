#include "toast_overlay.h"

#include <QHBoxLayout>
#include <QVBoxLayout>

ToastOverlay::ToastOverlay(QWidget* parent)
    : QWidget(parent) {
    setAttribute(Qt::WA_TransparentForMouseEvents, true);
    setAttribute(Qt::WA_NoSystemBackground, true);

    label = new QLabel(this);
    label->setStyleSheet(
        "QLabel { background: rgba(30, 30, 30, 210); color: white; padding: 8px 12px; "
        "border-radius: 6px; }");
    label->setVisible(false);

    auto* hbox = new QHBoxLayout();
    hbox->addStretch(1);
    hbox->addWidget(label);
    hbox->addStretch(1);

    auto* layout = new QVBoxLayout(this);
    layout->addStretch(1);
    layout->addLayout(hbox);
    layout->setContentsMargins(12, 12, 12, 12);
    setLayout(layout);

    hideTimer = new QTimer(this);
    hideTimer->setSingleShot(true);
    connect(hideTimer, &QTimer::timeout, this, [this]() { label->setVisible(false); });
}

void ToastOverlay::showMessage(const QString& message, int durationMs) {
    if (message.trimmed().isEmpty()) {
        return;
    }
    label->setText(message);
    label->setVisible(true);
    hideTimer->start(durationMs);
}

void ToastOverlay::resizeEvent(QResizeEvent* event) {
    QWidget::resizeEvent(event);
}
