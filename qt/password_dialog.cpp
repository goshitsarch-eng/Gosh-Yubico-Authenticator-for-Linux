#include "password_dialog.h"

#include <QDialogButtonBox>
#include <QFormLayout>
#include <QVBoxLayout>

PasswordDialog::PasswordDialog(QWidget* parent)
    : QDialog(parent) {
    setWindowTitle(tr("Unlock YubiKey"));

    passwordEdit = new QLineEdit(this);
    passwordEdit->setEchoMode(QLineEdit::Password);

    auto* form = new QFormLayout();
    form->addRow(tr("Password"), passwordEdit);

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel,
                                         Qt::Horizontal,
                                         this);
    connect(buttons, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &QDialog::reject);

    auto* layout = new QVBoxLayout(this);
    layout->addLayout(form);
    layout->addWidget(buttons);
    setLayout(layout);
}

QString PasswordDialog::password() const {
    return passwordEdit->text();
}
