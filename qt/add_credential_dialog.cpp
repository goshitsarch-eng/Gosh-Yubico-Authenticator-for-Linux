#include "add_credential_dialog.h"

#include <QDialogButtonBox>
#include <QFormLayout>
#include <QVBoxLayout>

AddCredentialDialog::AddCredentialDialog(QWidget* parent)
    : QDialog(parent) {
    setWindowTitle(tr("Add Credential"));

    issuerEdit = new QLineEdit(this);
    accountEdit = new QLineEdit(this);
    secretEdit = new QLineEdit(this);
    secretEdit->setEchoMode(QLineEdit::Password);

    typeCombo = new QComboBox(this);
    typeCombo->addItem(tr("TOTP"), 0x20);
    typeCombo->addItem(tr("HOTP"), 0x10);

    algorithmCombo = new QComboBox(this);
    algorithmCombo->addItem(tr("SHA1"), 0x01);
    algorithmCombo->addItem(tr("SHA256"), 0x02);
    algorithmCombo->addItem(tr("SHA512"), 0x03);

    digitsCombo = new QComboBox(this);
    digitsCombo->addItem(QStringLiteral("6"), 6);
    digitsCombo->addItem(QStringLiteral("7"), 7);
    digitsCombo->addItem(QStringLiteral("8"), 8);

    touchCheck = new QCheckBox(tr("Require touch"), this);
    counterCheck = new QCheckBox(tr("Set initial counter (HOTP)"), this);
    counterSpin = new QSpinBox(this);
    counterSpin->setRange(0, 99999999);
    counterSpin->setEnabled(false);

    connect(counterCheck, &QCheckBox::toggled, this, [this](bool checked) {
        counterSpin->setEnabled(checked);
    });
    connect(typeCombo, &QComboBox::currentIndexChanged, this, [this]() {
        updateCounterVisibility();
    });

    auto* form = new QFormLayout();
    form->addRow(tr("Issuer"), issuerEdit);
    form->addRow(tr("Account"), accountEdit);
    form->addRow(tr("Secret"), secretEdit);
    form->addRow(tr("Type"), typeCombo);
    form->addRow(tr("Algorithm"), algorithmCombo);
    form->addRow(tr("Digits"), digitsCombo);
    form->addRow(QString(), touchCheck);
    form->addRow(QString(), counterCheck);
    form->addRow(tr("Initial counter"), counterSpin);

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel,
                                         Qt::Horizontal,
                                         this);
    connect(buttons, &QDialogButtonBox::accepted, this, &QDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &QDialog::reject);

    auto* layout = new QVBoxLayout(this);
    layout->addLayout(form);
    layout->addWidget(buttons);
    setLayout(layout);

    updateCounterVisibility();
}

bool AddCredentialDialog::collectInput(CredentialInput* out, QString* error) const {
    if (!out) {
        return false;
    }
    const QString account = accountEdit->text().trimmed();
    if (account.isEmpty()) {
        if (error) {
            *error = tr("Account is required.");
        }
        return false;
    }
    const QString secret = secretEdit->text().trimmed();
    if (secret.isEmpty()) {
        if (error) {
            *error = tr("Secret is required.");
        }
        return false;
    }

    out->issuer = issuerEdit->text().trimmed();
    out->account = account;
    out->secret = secret;
    out->oathType = static_cast<uint8_t>(typeCombo->currentData().toUInt());
    out->algorithm = static_cast<uint8_t>(algorithmCombo->currentData().toUInt());
    out->digits = static_cast<uint8_t>(digitsCombo->currentData().toUInt());
    out->requireTouch = touchCheck->isChecked();
    const bool isHotp = out->oathType == 0x10;
    out->hasInitialCounter = isHotp && counterCheck->isChecked();
    out->initialCounter = static_cast<uint32_t>(counterSpin->value());
    return true;
}

void AddCredentialDialog::updateCounterVisibility() {
    const uint8_t oathType = static_cast<uint8_t>(typeCombo->currentData().toUInt());
    const bool isHotp = (oathType == 0x10);
    counterCheck->setVisible(isHotp);
    counterSpin->setVisible(isHotp);
}
