#pragma once

#include <QLabel>
#include <QTimer>
#include <QWidget>

class ToastOverlay : public QWidget {
    Q_OBJECT

public:
    explicit ToastOverlay(QWidget* parent = nullptr);
    void showMessage(const QString& message, int durationMs = 3000);

protected:
    void resizeEvent(QResizeEvent* event) override;

private:
    QLabel* label = nullptr;
    QTimer* hideTimer = nullptr;
};
