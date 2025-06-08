use flipperzero::notification::NotificationApp;

use crate::bt::BluetoothApp;

pub struct BtconnApp {
    // Apps
    pub bt: BluetoothApp,
    pub notifications: NotificationApp,
}

impl BtconnApp {
    pub fn new() -> Self {
        let bt = BluetoothApp::open();
        let notifications = NotificationApp::open();
        Self {
            bt: bt,
            notifications: notifications,
        }
    }
}
