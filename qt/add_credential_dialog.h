#pragma once

#include <cstdint>

#include <QCheckBox>
#include <QComboBox>
#include <QDialog>
#include <QLineEdit>
#include <QSpinBox>

struct CredentialInput {
    QString issuer;
    QString account;
    QString secret;
    uint8_t oathType;
    uint8_t algorithm;
    uint8_t digits;
    bool requireTouch;
    bool hasInitialCounter;
    uint32_t initialCounter;
};

class AddCredentialDialog : public QDialog {
    Q_OBJECT

public:
    explicit AddCredentialDialog(QWidget* parent = nullptr);

    bool collectInput(CredentialInput* out, QString* error) const;

private:
    void updateCounterVisibility();

    QLineEdit* issuerEdit = nullptr;
    QLineEdit* accountEdit = nullptr;
    QLineEdit* secretEdit = nullptr;
    QComboBox* typeCombo = nullptr;
    QComboBox* algorithmCombo = nullptr;
    QComboBox* digitsCombo = nullptr;
    QCheckBox* touchCheck = nullptr;
    QCheckBox* counterCheck = nullptr;
    QSpinBox* counterSpin = nullptr;
};
