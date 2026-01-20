#pragma once

#include <QDialog>
#include <QLineEdit>

class PasswordDialog : public QDialog {
    Q_OBJECT

public:
    explicit PasswordDialog(QWidget* parent = nullptr);

    QString password() const;

private:
    QLineEdit* passwordEdit = nullptr;
};
