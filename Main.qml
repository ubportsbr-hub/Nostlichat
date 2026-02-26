import QtQuick 2.9
import QtQuick.Window 2.2
import QtQuick.Controls 2.2
import QtQuick.Layouts 1.3
import QtQuick.Dialogs 1.0
import NostliBackend 1.0

Window {
    id: window
    visible: true
    width: 400
    height: 700
    title: "Nostlichat"
    
    readonly property color bgColor: backend.dark_mode ? "#1A1A1A" : "#F0F2F5"
    readonly property color textColor: backend.dark_mode ? "#FFFFFF" : "#000000"
    readonly property color barColor: backend.dark_mode ? "#2D2D2D" : "#FFFFFF"
    readonly property color myBubbleColor: backend.dark_mode ? "#056162" : "#DCF8C6"
    readonly property color otherBubbleColor: backend.dark_mode ? "#262D31" : "#FFFFFF"

    color: bgColor

    // --- State Management ---
    property bool showingSettings: false

    // --- DIALOGS ---
    FileDialog {
        id: fileDialog
        title: "Select Profile Picture"
        nameFilters: ["Images (*.jpg *.png *.webp)"]
        onAccepted: { backend.user_avatar = fileDialog.fileUrl.toString() }
    }

    FileDialog {
        id: photoPicker
        title: "Send a Photo"
        nameFilters: ["Images (*.png *.jpg *.jpeg)"]
        onAccepted: backend.send_image(photoPicker.fileUrl.toString())
    }

    Dialog {
        id: phoneHelperDialog
        title: "Phone Number"
        standardButtons: Dialog.Close
        x: (parent.width - width) / 2; y: (parent.height - height) / 2
        ColumnLayout {
            spacing: 15; width: 250
            TextField { id: phoneDisplay; readOnly: true; Layout.fillWidth: true; horizontalAlignment: Text.AlignHCenter; font.bold: true }
            Text { text: "The phone number must include the country code.\nExample: +5511999999999"; color: "#666"; font.pixelSize: 11; Layout.fillWidth: true; wrapMode: Text.Wrap }
            Button { text: "Copy Number"; Layout.fillWidth: true; onClicked: { phoneDisplay.selectAll(); phoneDisplay.copy() } }
        }
    }

    StackLayout {
        anchors.fill: parent
        // 0: Setup & Login | 1: Profile Settings | 2: Chat
        currentIndex: !backend.logged_in ? 0 : (showingSettings ? 1 : 2)
        
        // --- PAGE 0: ACCOUNT SETUP & LOGIN ---
        ScrollView {
            clip: true
            ColumnLayout {
                width: window.width; spacing: 15; Layout.margins: 25
                
                Text { 
                    text: "Account Setup"; 
                    font.pixelSize: 22; font.bold: true; color: "#0078D4"; 
                    Layout.alignment: Qt.AlignHCenter 
                }

                // Profile Picture Setup
                Rectangle {
                    width: 100; height: 100; radius: 50; color: "#DDD"; Layout.alignment: Qt.AlignHCenter; clip: true
                    Image { anchors.fill: parent; source: backend.user_avatar; fillMode: Image.PreserveAspectCrop; visible: backend.user_avatar !== "" }
                    Text { text: "Set\nPhoto"; anchors.centerIn: parent; horizontalAlignment: Text.AlignHCenter; font.pixelSize: 12; visible: backend.user_avatar === "" }
                    MouseArea { anchors.fill: parent; onClicked: fileDialog.open() }
                }

                Label { text: "Display Name"; font.bold: true; color: textColor }
                TextField { 
                    id: loginName; text: backend.user_name; Layout.fillWidth: true; placeholderText: "How others see you"
                    onEditingFinished: backend.user_name = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { loginName.forceActiveFocus(); mouse.accepted = false } }
                }

                Label { text: "Gmail Address (Required)"; font.bold: true; color: textColor }
                TextField { 
                    id: loginEmail; text: backend.user_email; Layout.fillWidth: true; placeholderText: "yourname@gmail.com"
                    inputMethodHints: Qt.ImhEmailCharactersOnly
                    onEditingFinished: backend.user_email = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { loginEmail.forceActiveFocus(); mouse.accepted = false } }
                }

                Label { text: "Phone Number"; font.bold: true; color: textColor }
                TextField { 
                    id: loginPhone; text: backend.user_phone; Layout.fillWidth: true; placeholderText: "+country..."
                    inputMethodHints: Qt.ImhDialableCharactersOnly
                    onEditingFinished: backend.user_phone = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { loginPhone.forceActiveFocus(); mouse.accepted = false } }
                }

                Label { text: "Bio / Description"; font.bold: true; color: textColor }
                TextArea { 
                    id: loginBio; text: backend.user_desc; Layout.fillWidth: true; Layout.preferredHeight: 60; placeholderText: "A bit about you..."
                    onEditingFinished: backend.user_desc = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { loginBio.forceActiveFocus(); mouse.accepted = false } }
                }

                Rectangle { Layout.fillWidth: true; height: 1; color: "#DDD"; Layout.topMargin: 10 }

                Text { text: "Connect to Google:"; font.bold: true; Layout.alignment: Qt.AlignHCenter }
                
                Button { 
                    text: "1. Get Login Link"; Layout.fillWidth: true; highlighted: true
                    onClicked: {
                        backend.user_email = loginEmail.text // Garante sincronia antes de abrir o link
                        backend.start_google_login()
                    }
                }

                TextField { 
                    id: loginField; placeholderText: "Paste callback URL here..."; 
                    Layout.fillWidth: true 
                }

                Button { 
                    text: "2. Confirm & Enter"; Layout.fillWidth: true; 
                    enabled: loginField.text.length > 0 && loginEmail.text !== ""
                    onClicked: {
                        backend.user_email = loginEmail.text
                        backend.handle_callback(loginField.text)
                    }
                }
            }
        }

        // --- PAGE 1: PROFILE SETTINGS (Post-Login) ---
        ScrollView {
            clip: true
            ColumnLayout {
                width: window.width; spacing: 15; Layout.margins: 25
                Text { text: "My Profile"; font.pixelSize: 22; font.bold: true; color: textColor }
                Rectangle {
                    width: 100; height: 100; radius: 50; color: "#DDD"; Layout.alignment: Qt.AlignHCenter; clip: true
                    Image { anchors.fill: parent; source: backend.user_avatar; fillMode: Image.PreserveAspectCrop; visible: backend.user_avatar !== "" }
                    Text { text: "Change\nPhoto"; anchors.centerIn: parent; horizontalAlignment: Text.AlignHCenter; font.pixelSize: 12; visible: backend.user_avatar === "" }
                    MouseArea { anchors.fill: parent; onClicked: fileDialog.open() }
                }
                Label { text: "Display Name"; font.bold: true; color: textColor }
                TextField { 
                    id: editName; text: backend.user_name; Layout.fillWidth: true
                    onEditingFinished: backend.user_name = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { editName.forceActiveFocus(); mouse.accepted = false } }
                }
                
                Label { text: "Gmail (Active Session)"; font.bold: true; color: textColor }
                TextField { text: backend.user_email; readOnly: true; Layout.fillWidth: true; color: "gray" }
                
                Label { text: "Phone Number"; font.bold: true; color: textColor }
                TextField { 
                    id: editPhone; text: backend.user_phone; Layout.fillWidth: true
                    onEditingFinished: backend.user_phone = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { editPhone.forceActiveFocus(); mouse.accepted = false } }
                }
                Label { text: "Bio"; font.bold: true; color: textColor }
                TextArea { 
                    id: editBio; text: backend.user_desc; Layout.fillWidth: true; Layout.preferredHeight: 60
                    onEditingFinished: backend.user_desc = text
                    MouseArea { anchors.fill: parent; propagateComposedEvents: true; onClicked: { editBio.forceActiveFocus(); mouse.accepted = false } }
                }
                Button { text: "Back to Chats"; Layout.fillWidth: true; highlighted: true; onClicked: showingSettings = false }
            }
        }

        // --- PAGE 2: CHAT (Mantida como original) ---
        ColumnLayout {
            spacing: 0
            Rectangle {
                Layout.fillWidth: true; height: 60; color: barColor
                RowLayout { 
                    anchors.fill: parent; anchors.margins: 10
                    Button { text: "☰"; flat: true; onClicked: drawer.open() }
                    Text { text: backend.active_room === "General" ? "Chats" : backend.active_room; font.bold: true; color: textColor; Layout.fillWidth: true }
                    Button { text: "📞"; visible: backend.active_room !== "General"; onClicked: { phoneDisplay.text = backend.get_current_phone(); phoneHelperDialog.open() } }
                }
            }
            ListView {
                id: chatView; Layout.fillWidth: true; Layout.fillHeight: true; clip: true; model: backend.messages; spacing: 8; Layout.margins: 10
                onCountChanged: chatView.positionViewAtEnd()
                delegate: RowLayout {
                    width: chatView.width - 20
                    Layout.alignment: modelData.startsWith("Me:") ? Qt.AlignRight : Qt.AlignLeft
                    Rectangle {
                        radius: 12; color: modelData.startsWith("Me:") ? myBubbleColor : otherBubbleColor
                        width: Math.min(msgT.implicitWidth + 24, chatView.width * 0.8); height: msgT.implicitHeight + 16
                        Text { id: msgT; text: modelData; anchors.centerIn: parent; width: parent.width - 16; wrapMode: Text.Wrap; color: textColor }
                    }
                }
            }
            Rectangle { 
                Layout.fillWidth: true; height: 70; color: barColor
                RowLayout {
                    anchors.fill: parent; anchors.margins: 10
                    Button { text: "📷"; flat: true; onClicked: photoPicker.open() }
                    TextField { id: msgIn; Layout.fillWidth: true; placeholderText: "Message..."; onAccepted: { if(text != "") { backend.send_message(text); text = "" } } }
                    Button { text: "➤"; onClicked: { if(msgIn.text != "") { backend.send_message(msgIn.text); msgIn.text = "" } } }
                }
            }
        }
    }

    Drawer { 
        id: drawer; width: 280; height: window.height 
        ColumnLayout {
            anchors.fill: parent; spacing: 5; anchors.margins: 10
            Text { text: "Nostlichat"; font.bold: true; font.pixelSize: 18; Layout.bottomMargin: 10 }
            ItemDelegate { text: "👤 My Profile"; Layout.fillWidth: true; onClicked: { showingSettings = true; drawer.close() } }
            Rectangle { Layout.fillWidth: true; height: 1; color: "#DDD" }
            Label { text: "Contacts"; font.pixelSize: 12; color: "#777" }
            ListView {
                Layout.fillWidth: true; Layout.fillHeight: true; clip: true; model: backend.contact_list
                delegate: ItemDelegate { width: parent.width; text: modelData; onClicked: { backend.active_room = modelData; showingSettings = false; drawer.close() } }
            }
            ItemDelegate { text: "➕ New Contact"; Layout.fillWidth: true; onClicked: { contactDialog.open(); drawer.close() } }
            SwitchDelegate { text: "Dark Mode"; Layout.fillWidth: true; checked: backend.dark_mode; onToggled: backend.dark_mode = checked }
            Button { text: "Logout"; Layout.fillWidth: true; onClicked: { backend.logout(); drawer.close() } }
        }
    }

    Dialog { 
        id: contactDialog; title: "New Contact"; standardButtons: Dialog.Save | Dialog.Cancel
        ColumnLayout { spacing: 10; width: 250
            TextField { id: cN; placeholderText: "Name"; Layout.fillWidth: true }
            TextField { id: cE; placeholderText: "Email (Gmail)"; Layout.fillWidth: true }
            TextField { id: cP; placeholderText: "Phone Number"; Layout.fillWidth: true }
        }
        onAccepted: { backend.save_contact(cN.text, cE.text, cP.text); cN.text = ""; cE.text = ""; cP.text = "" }
    }
}
