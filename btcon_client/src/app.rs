use flipperzero::notification::NotificationApp;

use crate::bt::BluetoothApp;

pub struct BtconnApp {
    // Apps
    pub bt: BluetoothApp,
    pub notifications: NotificationApp,
}
