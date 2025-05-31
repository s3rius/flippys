#pragma once

#include <furi.h>
#include <gui/gui.h>
#include <gui/scene_manager.h>
#include <bt/bt_service/bt.h>
#include <bt/bt_service/bt_keys_storage.h>
#include <ble_profile/extra_services/hid_service.h>
#include <extra_profiles/hid_profile.h>
#include <furi_hal_bt.h>
#include <storage/storage.h>
#include <gui/view_dispatcher.h>
#include <gui/modules/submenu.h>
#include <notification/notification.h>

#include "consts.h"
#include <flippys_icons.h>

typedef struct FlippysApp FlippysApp;

struct FlippysApp {
    Gui* gui;
    Bt* bt;
    FuriHalBleProfileBase* hid_profile;
    SceneManager* scene_manager;
    ViewDispatcher* view_dispatcher;
    Submenu* main_menu;
    NotificationApp* notification_app;
};
